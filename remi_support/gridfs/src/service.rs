// 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
// Copyright (c) 2022-2023 Noelware, LLC. <team@noelware.org>
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

use std::{io::Result, path::Path};

use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use log::*;
use mongodb::{bson::doc, Database};
use mongodb_gridfs::{options::GridFSFindOptions, GridFSBucket, GridFSError};
use remi_core::{Blob, FileBlob, ListBlobsRequest, StorageService, UploadRequest};
use tokio_stream::StreamExt;

use crate::GridfsStorageConfig;

fn to_io_error(error: mongodb::error::Error) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("mongodb: {}", error.kind.as_ref()),
    )
}

#[derive(Debug, Clone)]
pub struct GridfsStorageService {
    bucket: GridFSBucket,
}

impl GridfsStorageService {
    /// Creates a new [`GridfsStorageService`] with the MongoDB database and configuration options to configure this.
    /// It calls the [`GridfsStorageService::with_bucket`] function internally to get a instance of this service.
    pub fn new(database: &Database, options: GridfsStorageConfig) -> GridfsStorageService {
        let bucket = GridFSBucket::new(database.clone(), Some(options.to_gridfs_options()));
        GridfsStorageService { bucket }
    }
}

#[async_trait]
impl StorageService for GridfsStorageService {
    fn name(self) -> &'static str {
        "remi:gridfs"
    }

    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn open(&self, path: impl AsRef<Path> + Send) -> Result<Option<Bytes>> {
        let path = path.as_ref().to_string_lossy().into_owned();
        info!("opening file in path [{path}]");

        let mut cursor = self
            .bucket
            .find(
                doc! { "filename": path.clone() },
                GridFSFindOptions::default(),
            )
            .await
            .map_err(to_io_error)?;

        let advanced = cursor.advance().await.map_err(to_io_error)?;
        if !advanced {
            return Ok(None);
        }

        let oid = cursor.current().get_object_id("_id").unwrap();
        let result = self.bucket.open_download_stream(oid).await;

        if let Err(e) = result {
            match e {
                GridFSError::MongoError(error) => return Err(to_io_error(error)),
                GridFSError::FileNotFound() => return Ok(None),
            }
        }

        let mut stream = result.unwrap();
        let mut bytes = BytesMut::new();
        while let Some(raw) = stream.next().await {
            bytes.put(raw.as_ref());
        }

        Ok(Some(bytes.into()))
    }

    async fn blob(&self, path: impl AsRef<Path> + Send) -> Result<Option<Blob>> {
        let path = path.as_ref().to_string_lossy().into_owned();
        let bytes = self.open(path.clone()).await?;
        if bytes.is_none() {
            return Ok(None);
        }

        let bytes = bytes.unwrap();

        info!("getting file metadata for file [{path}]");
        let mut cursor = self
            .bucket
            .find(
                doc! { "filename": path.clone() },
                GridFSFindOptions::default(),
            )
            .await
            .map_err(to_io_error)?;

        let advanced = cursor.advance().await.map_err(to_io_error)?;
        if !advanced {
            return Ok(None);
        }

        let doc = cursor.current();
        let filename = doc.get_str("filename").unwrap();
        let length = doc.get_i64("length").unwrap();
        let content_type = doc.get_str("contentType").unwrap();
        //let created_at = doc.get_datetime("uploadDate").unwrap();

        Ok(Some(Blob::File(FileBlob::new(
            None,
            Some(content_type.to_owned()),
            None,
            false,
            "gridfs".into(),
            bytes,
            filename.into(),
            length as usize,
        ))))
    }

    async fn blobs(
        &self,
        _path: Option<impl AsRef<Path> + Send>,
        _options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        error!(
            "blobs(Path, ListBlobsRequest) is not supported at this time, returning empty vec..."
        );

        Ok(vec![])
    }

    async fn delete(&self, path: impl AsRef<Path> + Send) -> Result<()> {
        let path = path.as_ref().to_string_lossy().into_owned();
        warn!("deleting document in path [{path}]");

        let mut cursor = self
            .bucket
            .find(
                doc! { "filename": path.clone() },
                GridFSFindOptions::default(),
            )
            .await
            .map_err(to_io_error)?;

        let advanced = cursor.advance().await.map_err(to_io_error)?;
        if !advanced {
            warn!("file [{path}] doesn't even exist, skipping");
            return Ok(());
        }

        let doc = cursor.current();
        let oid = doc.get_object_id("_id").unwrap();

        match self.bucket.delete(oid).await {
            Ok(()) => Ok(()),
            Err(e) => match e {
                GridFSError::FileNotFound() => {
                    warn!("file [{path}] doesn't even exist, skipping");
                    Ok(())
                }

                GridFSError::MongoError(e) => Err(to_io_error(e)),
            },
        }
    }

    async fn exists(&self, path: impl AsRef<Path> + Send) -> Result<bool> {
        let path = path.as_ref().to_string_lossy().into_owned();
        match self.open(path).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn upload(&self, _path: impl AsRef<Path> + Send, _options: UploadRequest) -> Result<()> {
        warn!("#upload(Path, UploadRequest) is not supported at this time.");
        Ok(())
    }
}
