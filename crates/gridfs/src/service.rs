// 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
// Copyright (c) 2022-2024 Noelware, LLC. <team@noelware.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::StorageConfig;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures_util::{AsyncWriteExt, StreamExt};
use mongodb::{
    bson::{doc, raw::ValueAccessErrorKind, Bson, RawDocument},
    options::GridFsFindOptions,
    Database, GridFsBucket,
};
use remi::{Blob, File, ListBlobsRequest, UploadRequest};
use std::{io, path::Path};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};

fn value_access_err_to_error(error: mongodb::bson::raw::ValueAccessError) -> mongodb::error::Error {
    match error.kind {
        ValueAccessErrorKind::NotPresent => {
            mongodb::error::Error::custom(format!("key [{}] was not found", error.key()))
        }

        ValueAccessErrorKind::UnexpectedType { expected, actual, .. } => mongodb::error::Error::custom(format!(
            "expected BSON type '{expected:?}', actual type for key [{}] is '{actual:?}'",
            error.key()
        )),

        ValueAccessErrorKind::InvalidBson(err) => err.into(),
        _ => unimplemented!(
            "`ValueAccessErrorKind` was unhandled, please report it: https://github.com/Noelware/remi-rs/issues/new"
        ),
    }
}

fn document_to_blob(bytes: Bytes, doc: &RawDocument) -> Result<File, mongodb::error::Error> {
    let filename = doc.get_str("filename").map_err(value_access_err_to_error)?;
    let length = doc.get_i64("length").map_err(value_access_err_to_error)?;
    let content_type = doc.get_str("contentType").map_err(value_access_err_to_error)?;
    let created_at = doc.get_datetime("uploadDate").map_err(value_access_err_to_error)?;

    Ok(File {
        last_modified_at: None,
        content_type: Some(content_type.to_owned()),
        created_at: if created_at.timestamp_millis() < 0 {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(remi.service = "gridfs", %filename, "`created_at` timestamp was negative");

            #[cfg(feature = "log")]
            ::log::warn!("`created_at` for file {filename} was negative");

            None
        } else {
            Some(
                u128::try_from(created_at.timestamp_millis())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            )
        },

        is_symlink: false,
        data: bytes,
        name: filename.to_owned(),
        path: format!("gridfs://{filename}"),
        size: if length < 0 {
            0
        } else {
            length
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        },
    })
}

#[deprecated(
    since = "0.5.0",
    note = "`GridfsStorageService` has been renamed to `StorageService`, this will be removed in v0.7.0"
)]
pub type GridfsStorageService = StorageService;

#[derive(Debug, Clone)]
pub struct StorageService(GridFsBucket);

impl StorageService {
    /// Creates a new [`StorageService`] which uses the [`StorageConfig`] as a way to create
    /// the inner [`GridFsBucket`].
    pub fn new(db: &Database, config: StorageConfig) -> StorageService {
        let bucket = db.gridfs_bucket(Some(config.into()));
        StorageService::with_bucket(bucket)
    }

    /// Uses a preconfigured [`GridFsBucket`] as the underlying bucket.
    pub fn with_bucket(bucket: GridFsBucket) -> StorageService {
        StorageService(bucket)
    }
}

#[async_trait]
impl remi::StorageService for StorageService {
    type Error = mongodb::error::Error;
    const NAME: &'static str = "remi:gridfs";

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.gridfs.open",
            skip_all,
            fields(
                remi.service = "gridfs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>, Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(remi.service = "gridfs", file = %path.display(), "opening file");

        #[cfg(feature = "log")]
        ::log::info!("opening file [{}]", path.display());

        // ensure that the `path` is utf-8 encoded, because I think
        // MongoDB expects strings to be utf-8 encoded?
        let path_str = path
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected utf-8 encoded path string"))?;

        let mut cursor = self
            .0
            .find(doc! { "filename": path_str }, GridFsFindOptions::default())
            .await?;

        let advanced = cursor.advance().await?;
        if !advanced {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(
                remi.service = "gridfs",
                file = %path.display(),
                "file doesn't exist in GridFS"
            );

            #[cfg(feature = "log")]
            ::log::warn!("file [{}] doesn't exist in GridFS", path.display());

            return Ok(None);
        }

        let doc = cursor.current();
        let stream = self
            .0
            .open_download_stream(Bson::ObjectId(
                doc.get_object_id("_id").map_err(value_access_err_to_error)?,
            ))
            .await?;

        let mut bytes = BytesMut::new();
        let mut reader = ReaderStream::new(stream.compat());
        while let Some(raw) = reader.next().await {
            match raw {
                Ok(b) => bytes.extend(b),
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Some(bytes.into()))
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.gridfs.blob",
            skip_all,
            fields(
                remi.service = "gridfs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>, Self::Error> {
        let path = path.as_ref();
        let Some(bytes) = self.open(path).await? else {
            return Ok(None);
        };

        // .unwrap() is safe here since .open() validates if the path is a
        // utf-8 string.
        let path_str = path.to_str().unwrap();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            remi.service = "gridfs",
            file = %path.display(),
            "getting file metadata for file"
        );

        #[cfg(feature = "log")]
        ::log::info!("getting file metadata for file [{}]", path.display());

        let mut cursor = self
            .0
            .find(
                doc! {
                    "filename": path_str,
                },
                GridFsFindOptions::default(),
            )
            .await?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await?;
        if !has_advanced {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(remi.service = "gridfs", file = %path.display(), "file doesn't exist");

            #[cfg(feature = "log")]
            ::log::warn!("file [{}] doesn't exist", path.display());

            return Ok(None);
        }

        let doc = cursor.current();
        document_to_blob(bytes, doc).map(|doc| Some(Blob::File(doc)))
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.gridfs.blobs",
            skip_all,
            fields(
                remi.service = "gridfs"
            )
        )
    )]
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        _request: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>, Self::Error> {
        // TODO(@auguwu): support filtering files, for now we should probably
        // heavily test this
        #[allow(unused)]
        if let Some(path) = path {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(
                remi.service = "gridfs",
                file = %path.as_ref().display(),
                "using blobs() with a given file name is not supported",
            );

            #[cfg(feature = "log")]
            ::log::warn!(
                "using blobs() with a given file name [{}] is not supported",
                path.as_ref().display()
            );

            return Ok(vec![]);
        }

        let mut cursor = self.0.find(doc!(), GridFsFindOptions::default()).await?;

        let mut blobs = vec![];
        while cursor.advance().await? {
            let doc = cursor.current();
            let stream = self
                .0
                .open_download_stream(Bson::ObjectId(
                    doc.get_object_id("_id").map_err(value_access_err_to_error)?,
                ))
                .await?;

            let mut bytes = BytesMut::new();
            let mut reader = ReaderStream::new(stream.compat());
            while let Some(raw) = reader.next().await {
                match raw {
                    Ok(b) => bytes.extend(b),
                    Err(e) => return Err(e.into()),
                }
            }

            match document_to_blob(bytes.into(), doc) {
                Ok(blob) => blobs.push(Blob::File(blob)),

                #[cfg(any(feature = "tracing", feature = "log"))]
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    ::tracing::error!(remi.service = "gridfs", error = %e, "unable to convert to a file");

                    #[cfg(feature = "log")]
                    ::log::error!("unable to convert to a file: {e}");
                }

                #[cfg(not(any(feature = "tracing", feature = "log")))]
                Err(_e) => {}
            }
        }

        Ok(blobs)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.gridfs.delete",
            skip_all,
            fields(
                remi.service = "gridfs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<(), Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(remi.service = "gridfs", file = %path.display(), "deleting file");

        #[cfg(feature = "log")]
        ::log::info!("deleting file [{}]", path.display());

        // ensure that the `path` is utf-8 encoded, because I think
        // MongoDB expects strings to be utf-8 encoded?
        let path_str = path
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected utf-8 encoded path string"))?;

        let mut cursor = self
            .0
            .find(
                doc! {
                    "filename": path_str,
                },
                GridFsFindOptions::default(),
            )
            .await?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await?;
        if !has_advanced {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(remi.service = "gridfs", file = %path.display(), "file doesn't exist");

            #[cfg(feature = "log")]
            ::log::warn!("file [{}] doesn't exist", path.display());

            return Ok(());
        }

        let doc = cursor.current();
        let oid = doc.get_object_id("_id").map_err(value_access_err_to_error)?;

        self.0.delete(Bson::ObjectId(oid)).await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.gridfs.exists",
            skip_all,
            fields(
                remi.service = "gridfs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool, Self::Error> {
        match self.open(path).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.gridfs.blob",
            skip_all,
            fields(
                remi.service = "gridfs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> Result<(), Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            remi.service = "gridfs",
            file = %path.display(),
            "uploading file to GridFS..."
        );

        #[cfg(feature = "log")]
        ::log::info!("uploading file [{}] to GridFS", path.display());

        // ensure that the `path` is utf-8 encoded, because I think
        // MongoDB expects strings to be utf-8 encoded?
        let path_str = path
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected utf-8 encoded path string"))?;

        let mut stream = self.0.open_upload_stream(path_str, None);
        stream.write_all(&options.data[..]).await?;
        stream.close().await.map_err(From::from)

        // TODO(@auguwu): add metadata to document that was created and the given content type
        // if one was supplied.
    }
}
