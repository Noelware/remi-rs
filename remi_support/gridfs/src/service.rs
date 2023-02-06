// 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
// Copyright (c) 2022-2023 Noelware <team@noelware.org>
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

use std::{
    io::{Error, Result},
    path::Path,
};

use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use log::*;
use mongodb::{
    bson::{doc, Document},
    options::{DeleteOptions, FindOneOptions, FindOptions},
    Collection, Database,
};
use remi_core::{
    blob::{Blob, FileBlob},
    builders::{ListBlobsRequest, UploadRequest},
    StorageService,
};

use crate::GridfsStorageConfig;

fn to_io_error(error: mongodb::error::Error) -> Error {
    Error::new(
        std::io::ErrorKind::Other,
        format!("mongodb: {}", error.kind.as_ref()),
    )
}

#[derive(Debug, Clone)]
pub struct GridfsStorageService {
    database: Database,
    config: GridfsStorageConfig,
}

impl GridfsStorageService {
    /// Creates a new [`GridfsStorageService`] with the MongoDB database and configuration options to configure this.
    /// It calls the [`GridfsStorageService::with_bucket`] function internally to get a instance of this service.
    pub fn new(database: &Database, options: GridfsStorageConfig) -> GridfsStorageService {
        //let bucket = GridFSBucket::new(database.clone(), Some(options.to_gridfs_options()));
        GridfsStorageService {
            database: database.clone(),
            config: options,
        }
    }

    fn files_collection(&self) -> Collection<Document> {
        self.database
            .collection::<Document>(format!("{}.files", self.config.bucket_name()).as_str())
    }

    fn chunks_collection(&self) -> Collection<Document> {
        self.database
            .collection::<Document>(format!("{}.chunks", self.config.bucket_name()).as_str())
    }
}

#[async_trait]
impl StorageService for GridfsStorageService {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>> {
        let path = path.as_ref().to_string_lossy().into_owned();
        info!("opening file in path [{path}]");

        let files = self.files_collection();
        let chunks = self.chunks_collection();

        let mut find_one_opts = FindOneOptions::default();
        let mut find_options = FindOptions::default();

        if let Some(concern) = self.config.read_concern() {
            find_one_opts.read_concern = Some(concern.clone());
            find_options.read_concern = Some(concern);
        }

        if let Some(pref) = self.config.read_preference() {
            find_one_opts.selection_criteria = Some(
                mongodb::options::SelectionCriteria::ReadPreference(pref.clone()),
            );
            find_options.selection_criteria =
                Some(mongodb::options::SelectionCriteria::ReadPreference(pref));
        }

        // First, we need to find the file. Which will be in "bucket_name.files",
        // so we need to query it by the filename! Which, will give us the
        // object ID for that file.
        let file = files
            .find_one(
                doc! {
                    "filename": path
                },
                None,
            )
            .await
            .map_err(to_io_error)?;

        if file.is_none() {
            return Ok(None);
        }

        let file = file.unwrap();
        let oid = file.get_object_id("_id").unwrap();

        // Now, we need to get the chunk that this file (might) have.
        let mut stream = chunks
            .find(
                doc! {
                    "files_id": oid
                },
                Some(find_options),
            )
            .await
            .map_err(to_io_error)?;

        let mut bytes = BytesMut::new();
        while stream.advance().await.map_err(to_io_error)? {
            let doc = stream.current();
            let raw = doc.as_bytes();

            bytes.put(raw);
        }

        Ok(Some(bytes.into()))
    }

    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>> {
        let path = path.as_ref().to_string_lossy().into_owned();
        let bytes = self.open(path.clone()).await?;
        if bytes.is_none() {
            return Ok(None);
        }

        info!("getting file metadata for file [{path}]");
        let files = self.files_collection();
        let mut find_one_opts = FindOneOptions::default();

        if let Some(concern) = self.config.read_concern() {
            find_one_opts.read_concern = Some(concern);
        }

        if let Some(pref) = self.config.read_preference() {
            find_one_opts.selection_criteria =
                Some(mongodb::options::SelectionCriteria::ReadPreference(pref));
        }

        // First, we need to find the file. Which will be in "bucket_name.files",
        // so we need to query it by the filename! Which, will give us the
        // object ID for that file.
        let doc = files
            .find_one(
                doc! {
                    "filename": path
                },
                None,
            )
            .await
            .map_err(to_io_error)?
            .unwrap(); // SAFETY: it's already validated in #open, so we should be fine.

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
            bytes.unwrap(),
            filename.into(),
            length as usize,
        ))))
    }

    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        _path: Option<P>,
        _options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        Ok(vec![])
    }

    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<()> {
        let path = path.as_ref().to_string_lossy().into_owned();
        warn!("deleting document [{path}]");

        // First, we need to find the file by the path.
        let files = self.files_collection();
        let mut find_one_opts = FindOneOptions::default();

        if let Some(concern) = self.config.read_concern() {
            find_one_opts.read_concern = Some(concern);
        }

        if let Some(pref) = self.config.read_preference() {
            find_one_opts.selection_criteria =
                Some(mongodb::options::SelectionCriteria::ReadPreference(pref));
        }

        let file = files
            .find_one(
                doc! {
                    "filename": path.clone()
                },
                None,
            )
            .await
            .map_err(to_io_error)?;

        if file.is_none() {
            warn!("file [{path}] didn't exist, not doing anything");
            return Ok(());
        }

        let doc = file.unwrap();
        let oid = doc.get_object_id("_id").unwrap();

        // First, we will delete all the chunks
        let mut delete_opts = DeleteOptions::default();
        if let Some(concern) = self.config.write_concern() {
            delete_opts.write_concern = Some(concern);
        }

        files
            .delete_many(
                doc! {
                    "files_id": oid
                },
                Some(delete_opts),
            )
            .await
            .map_err(to_io_error)?;

        Ok(())
    }

    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool> {
        let path = path.as_ref().to_string_lossy().into_owned();
        match self.open(path).await {
            Ok(Some(_)) => Ok(true),
            _ => Ok(false),
        }
    }

    async fn upload<P: AsRef<Path> + Send>(&self, _path: P, _options: UploadRequest) -> Result<()> {
        Ok(())
    }
}
