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
    options::{GridFsFindOptions, GridFsUploadOptions},
    Client, Database, GridFsBucket,
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
    let created_at = doc.get_datetime("uploadDate").map_err(value_access_err_to_error)?;
    let content_type = match doc.get_str("contentType") {
        Ok(res) => Some(res),
        Err(e) => match e.kind {
            ValueAccessErrorKind::NotPresent => None,
            _ => return Err(value_access_err_to_error(e)),
        },
    };

    Ok(File {
        last_modified_at: None,
        content_type: content_type.map(String::from),
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
pub struct StorageService {
    config: Option<StorageConfig>,
    bucket: GridFsBucket,
}

impl StorageService {
    /// Creates a new [`StorageService`] which uses the [`StorageConfig`] as a way to create
    /// the inner [`GridFsBucket`].
    pub fn new(db: Database, config: StorageConfig) -> StorageService {
        let bucket = db.gridfs_bucket(Some(config.clone().into()));
        StorageService {
            config: Some(config),
            bucket,
        }
    }

    /// Return a new [`StorageService`] from a constructed [`Client`].
    pub fn from_client(client: &Client, config: StorageConfig) -> StorageService {
        Self::new(
            client.database(&config.clone().database.unwrap_or(String::from("mydb"))),
            config,
        )
    }

    /// Creates a MongoDB client from a connection string and creates a new [`StorageService`] interface.
    pub async fn from_conn_string<C: AsRef<str>>(
        conn_string: C,
        config: StorageConfig,
    ) -> Result<StorageService, mongodb::error::Error> {
        let client = Client::with_uri_str(conn_string).await?;
        Ok(Self::from_client(&client, config))
    }

    /// Uses a preconfigured [`GridFsBucket`] as the underlying bucket.
    pub fn with_bucket(bucket: GridFsBucket) -> StorageService {
        StorageService { config: None, bucket }
    }

    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> Result<String, mongodb::error::Error> {
        let path = path.as_ref().to_str().ok_or_else(|| {
            <mongodb::error::Error as From<io::Error>>::from(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected valid utf-8 string",
            ))
        })?;

        // trim `./` and `~/` since S3 doesn't accept ./ or ~/ as valid paths
        let path = path.trim_start_matches("~/").trim_start_matches("./");

        Ok(path.to_owned())
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
        let path = self.resolve_path(path)?;

        #[cfg(feature = "tracing")]
        ::tracing::info!(remi.service = "gridfs", file = %path, "opening file");

        #[cfg(feature = "log")]
        ::log::info!("opening file [{}]", path);

        let mut cursor = self
            .bucket
            .find(doc! { "filename": &path }, GridFsFindOptions::default())
            .await?;

        let advanced = cursor.advance().await?;
        if !advanced {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(
                remi.service = "gridfs",
                file = %path,
                "file doesn't exist in GridFS"
            );

            #[cfg(feature = "log")]
            ::log::warn!("file [{}] doesn't exist in GridFS", path);

            return Ok(None);
        }

        let doc = cursor.current();
        let stream = self
            .bucket
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
        let path = self.resolve_path(path)?;
        let Some(bytes) = self.open(&path).await? else {
            return Ok(None);
        };

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            remi.service = "gridfs",
            file = %path,
            "getting file metadata for file"
        );

        #[cfg(feature = "log")]
        ::log::info!("getting file metadata for file [{}]", path);

        let mut cursor = self
            .bucket
            .find(
                doc! {
                    "filename": &path,
                },
                GridFsFindOptions::default(),
            )
            .await?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await?;
        if !has_advanced {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(remi.service = "gridfs", file = %path, "file doesn't exist");

            #[cfg(feature = "log")]
            ::log::warn!("file [{}] doesn't exist", path);

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

        let mut cursor = self.bucket.find(doc!(), GridFsFindOptions::default()).await?;
        let mut blobs = vec![];
        while cursor.advance().await? {
            let doc = cursor.current();
            let stream = self
                .bucket
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
        let path = self.resolve_path(path)?;

        #[cfg(feature = "tracing")]
        ::tracing::info!(remi.service = "gridfs", file = %path, "deleting file");

        #[cfg(feature = "log")]
        ::log::info!("deleting file [{}]", path);

        let mut cursor = self
            .bucket
            .find(
                doc! {
                    "filename": &path,
                },
                GridFsFindOptions::default(),
            )
            .await?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await?;
        if !has_advanced {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(remi.service = "gridfs", file = %path, "file doesn't exist");

            #[cfg(feature = "log")]
            ::log::warn!("file [{}] doesn't exist", path);

            return Ok(());
        }

        let doc = cursor.current();
        let oid = doc.get_object_id("_id").map_err(value_access_err_to_error)?;

        self.bucket.delete(Bson::ObjectId(oid)).await
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
        let path = self.resolve_path(path)?;

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            remi.service = "gridfs",
            file = %path,
            "uploading file to GridFS..."
        );

        #[cfg(feature = "log")]
        ::log::info!("uploading file [{}] to GridFS", path);

        let opts = GridFsUploadOptions::builder()
            .chunk_size_bytes(Some(
                self.config.clone().unwrap_or_default().chunk_size.unwrap_or(255 * 1024),
            ))
            .metadata(match options.metadata.is_empty() {
                true => None,
                false => Some(
                    options
                        .metadata
                        .into_iter()
                        .map(|(k, v)| (k, Bson::String(v)))
                        .collect(),
                ),
            })
            .build();

        let mut stream = self.bucket.open_upload_stream(path, Some(opts));
        stream.write_all(&options.data[..]).await?;
        stream.close().await.map_err(From::from)

        // TODO(@auguwu): add metadata to document that was created and the given content type
        // if one was supplied.
    }
}
