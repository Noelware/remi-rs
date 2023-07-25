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

use crate::config::S3StorageConfig;
use async_trait::async_trait;
use aws_config::AppName;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::{config::Credentials, primitives::ByteStream, types::Object, Client, Config};
use bytes::{Bytes, BytesMut};
use log::*;
use remi_core::{Blob, DirectoryBlob, FileBlob, ListBlobsRequest, StorageService, UploadRequest};
use std::{borrow::Cow, io::Result, path::Path};
use tokio::io::{AsyncReadExt, BufReader};

#[derive(Debug, Clone)]
pub struct S3StorageService {
    config: S3StorageConfig,
    client: Client,
}

macro_rules! to_io_error {
    ($x:expr) => {
        ::std::io::Error::new(std::io::ErrorKind::Other, format!("{}", $x))
    };
}

impl S3StorageService {
    /// Creates a new [`S3StorageService`] from the configuration object given.
    pub fn new(config: S3StorageConfig) -> S3StorageService {
        let mut sdk_config = Config::builder();
        sdk_config.set_credentials_provider(Some(SharedCredentialsProvider::new(
            Credentials::new(
                config.access_key_id(),
                config.secret_access_key(),
                None,
                None,
                "remi-rs",
            ),
        )));

        sdk_config.set_endpoint_url(Some(config.endpoint()));
        sdk_config.set_app_name(Some(
            AppName::new(Cow::Owned(config.app_name().unwrap_or("remi-rs".into()))).unwrap(),
        ));

        if config.enforce_path_access_style() {
            sdk_config.set_force_path_style(Some(true));
        }

        let sdk_config = sdk_config.region(config.region()).build();
        S3StorageService {
            config,
            client: Client::from_conf(sdk_config),
        }
    }

    /// Overwrites the [configuration][S3StorageConfig] and reconfigures the AWS SDK
    /// to use the new configuration. You will need to call [`StorageService::init`] once
    /// more to assure that buckets are created safely.
    pub fn overwrite_config(&mut self, config: S3StorageConfig) {
        self.config = config;
        self.configure();
    }

    pub(crate) fn configure(&mut self) {
        info!(
            "setting up AWS SDK client with endpoint {} for app name [{:?}]",
            self.config.endpoint(),
            self.config.app_name()
        );

        let mut sdk_config = Config::builder();
        sdk_config.set_credentials_provider(Some(SharedCredentialsProvider::new(
            Credentials::new(
                self.config.access_key_id(),
                self.config.secret_access_key(),
                None,
                None,
                "remi-rs",
            ),
        )));

        sdk_config.set_endpoint_url(Some(self.config.endpoint()));
        sdk_config.set_app_name(Some(
            AppName::new(Cow::Owned(
                self.config.app_name().unwrap_or("remi-rs".into()),
            ))
            .unwrap(),
        ));

        if self.config.enforce_path_access_style() {
            sdk_config.set_force_path_style(Some(true));
        }

        let sdk_config = sdk_config.region(self.config.region()).build();
        self.client = Client::from_conf(sdk_config);
    }

    pub(crate) fn resolve_path<P: AsRef<Path>>(&self, path: P) -> String {
        let config = self.config.clone();
        match config.prefix() {
            Some(p) => format!("{p}/{}", path.as_ref().display()),
            None => path.as_ref().to_string_lossy().clone().to_string(),
        }
    }

    async fn s3_obj_to_blob(&self, entry: &Object) -> Result<Option<Blob>> {
        if entry.key().is_none() {
            return Ok(None);
        }

        let key = entry.key().unwrap();
        if key.ends_with('/') {
            let dir_blob = DirectoryBlob::new(None, "s3".into(), key.to_owned());
            return Ok(Some(Blob::Directory(dir_blob)));
        }

        self.blob(entry.key().unwrap()).await
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    fn name(self) -> &'static str {
        "remi:s3"
    }

    async fn init(&self) -> Result<()> {
        info!("Ensuring bucket [{}] exists...", self.config.bucket());

        // Check if the bucket exists
        let bucket_req = self
            .client
            .list_buckets()
            .send()
            .await
            .map_err(|x| to_io_error!(x))?;

        let buckets = bucket_req.buckets().unwrap_or_default();
        let has_bucket = buckets.iter().any(|x| {
            let name = x.name();
            if name.is_none() {
                return false;
            }

            name.unwrap() == self.config.bucket()
        });

        if !has_bucket {
            let bucket = self.config.bucket();
            warn!("Bucket [{bucket}] doesn't exist, creating!");

            self.client
                .create_bucket()
                .bucket(bucket.clone())
                .acl(self.config.default_bucket_acl())
                .send()
                .await
                .map_err(|x| to_io_error!(x))?;
        } else {
            info!("Bucket [{}] exists!", self.config.bucket());
        }

        Ok(())
    }

    async fn open(&self, path: impl AsRef<Path> + Send) -> Result<Option<Bytes>> {
        let normalized = self.resolve_path(path.as_ref());
        trace!("opening file {normalized}...");

        let obj = self
            .client
            .get_object()
            .bucket(self.config.bucket())
            .key(normalized)
            .send()
            .await;

        match obj {
            Ok(obj) => {
                let mut bytes = BytesMut::new();
                let stream = obj.body;

                let mut reader = BufReader::new(stream.into_async_read());
                reader
                    .read_exact(&mut bytes)
                    .await
                    .map_err(|x| to_io_error!(x))?;

                Ok(Some(bytes.into()))
            }

            Err(e) => {
                let error = e.into_service_error();
                if error.is_no_such_key() {
                    return Ok(None);
                }

                Err(to_io_error!(error))
            }
        }
    }

    async fn blob(&self, path: impl AsRef<Path> + Send) -> Result<Option<Blob>> {
        let path = path.as_ref();
        let normalized = self.resolve_path(path);
        trace!("opening file {normalized}...");

        let obj = self
            .client
            .get_object()
            .bucket(self.config.bucket())
            .key(normalized.clone())
            .send()
            .await;

        match obj {
            Ok(obj) => {
                let mut bytes = BytesMut::new();
                let content_type = obj.content_type().map(|x| x.to_owned());
                let last_modified_at = obj
                    .last_modified()
                    .map(|dt| dt.to_millis().expect("cant convert into millis") as u128);

                let stream = obj.body.into_async_read();
                let mut reader = BufReader::new(stream);
                reader
                    .read_exact(&mut bytes)
                    .await
                    .map_err(|x| to_io_error!(x))?;

                let bytes: Bytes = bytes.into();
                Ok(Some(Blob::File(FileBlob::new(
                    last_modified_at,
                    content_type,
                    None,
                    false,
                    "s3".into(),
                    bytes.clone(),
                    normalized,
                    bytes.len(),
                ))))
            }

            Err(e) => {
                let error = e.into_service_error();
                if error.is_no_such_key() {
                    return Ok(None);
                }

                Err(to_io_error!(error))
            }
        }
    }

    async fn blobs(
        &self,
        path: Option<impl AsRef<Path> + Send>,
        options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        let options = match options {
            Some(req) => req,
            None => ListBlobsRequest::default(),
        };

        if path.is_none() {
            let mut blobs: Vec<Blob> = Vec::new();
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(self.config.bucket())
                .max_keys(1000); // TODO: add this in ListBlobsRequest?

            loop {
                let resp = req.clone().send().await.map_err(|x| to_io_error!(x))?;
                let entries = resp.contents().unwrap_or_default();
                trace!("found {} entries", entries.len());

                for entry in entries {
                    let name = entry.key();
                    if name.is_none() {
                        trace!("skipping entry due to no name");
                        continue;
                    }

                    let name = name.unwrap();
                    if options.is_excluded(name) {
                        debug!("S3 object with key [{name}] is being excluded from output");
                    }

                    match self.s3_obj_to_blob(entry).await {
                        Ok(Some(blob)) => {
                            blobs.push(blob);
                        }

                        Err(e) => {
                            warn!("skipping error [{e}] when listing objects, object [{name:?}] will not be present in the final result");
                            continue;
                        }

                        _ => continue,
                    }
                }

                if let Some(token) = resp.continuation_token() {
                    req = req.clone().continuation_token(token);
                } else {
                    break;
                }
            }

            return Ok(blobs);
        }

        let path = path.unwrap();
        let resolved = self.resolve_path(path);

        let mut blobs: Vec<Blob> = Vec::new();
        let mut req = self
            .client
            .list_objects_v2()
            .bucket(self.config.bucket())
            .max_keys(1000) // TODO: add this in ListBlobsRequest?
            .set_prefix(Some(resolved));

        loop {
            let resp = req.clone().send().await.map_err(|x| to_io_error!(x))?;
            let entries = resp.contents().unwrap_or_default();
            trace!("found {} entries", entries.len());

            for entry in entries {
                let name = entry.key();
                if name.is_none() {
                    trace!("skipping entry due to no name");
                    continue;
                }

                let name = name.unwrap();
                if options.is_excluded(name) {
                    debug!("S3 object with key [{name}] is being excluded from output");
                }

                match self.s3_obj_to_blob(entry).await {
                    Ok(Some(blob)) => {
                        blobs.push(blob);
                    }

                    Err(e) => {
                        warn!("skipping error [{e}] when listing objects, object [{name:?}] will not be present in the final result");
                        continue;
                    }

                    _ => continue,
                }
            }

            if let Some(token) = resp.continuation_token() {
                req = req.clone().continuation_token(token);
            } else {
                break;
            }
        }

        Ok(blobs)
    }

    async fn delete(&self, path: impl AsRef<Path> + Send) -> Result<()> {
        self.client
            .delete_object()
            .bucket(self.config.bucket())
            .key(self.resolve_path(path))
            .send()
            .await
            .map(|_| ())
            .map_err(|x| to_io_error!(x))
    }

    async fn exists(&self, path: impl AsRef<Path> + Send) -> Result<bool> {
        let res = self
            .client
            .head_object()
            .bucket(self.config.bucket())
            .key(self.resolve_path(path))
            .send()
            .await
            .map(|resp| {
                // If the object has a delete marker, we should return
                // false for this.
                if resp.delete_marker() {
                    return false;
                }

                // assume it is true? since the head_object throws
                // a sdk error if it doesn't exist
                true
            });

        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let inner = e.into_service_error();
                if inner.is_not_found() {
                    return Ok(false);
                }

                return Err(to_io_error!(inner));
            }
        }
    }

    async fn upload(&self, path: impl AsRef<Path> + Send, options: UploadRequest) -> Result<()> {
        let path = self.resolve_path(path);
        let content_type = options
            .content_type
            .unwrap_or("application/octet-stream".into());

        trace!("uploading object [{path}] with content type [{content_type:?}]");
        let len = options.data.len();
        let stream: ByteStream = options.data.into();

        self.client
            .put_object()
            .bucket(self.config.bucket())
            .key(path)
            .acl(self.config.default_object_acl())
            .body(stream)
            .content_type(content_type)
            .content_length(len as i64)
            .send()
            .await
            .map(|_| ())
            .map_err(|x| to_io_error!(x))
    }
}
