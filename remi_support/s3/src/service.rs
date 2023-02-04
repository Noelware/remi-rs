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

use std::{io::Result, path::Path};

use async_trait::async_trait;

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::{Client, Credentials};
use bytes::{Bytes, BytesMut};
use remi_core::{
    blob::{Blob, FileBlob},
    builders::{ListBlobsRequest, UploadRequest},
    StorageService,
};

use log::*;
use tokio::io::{AsyncReadExt, BufReader};

use crate::config::S3StorageConfig;

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
    /// Creates a [`S3StorageService`] with the credentials in the system environment variables. Use the `new`
    /// function to customize the behaviour of this [storage service][S3StorageService]
    pub async fn from_env() -> S3StorageService {
        let raw = aws_config::from_env().load().await;
        S3StorageService {
            config: S3StorageConfig::builder()
                .bucket("remi".into())
                .build()
                .unwrap(),
            client: Client::new(&raw),
        }
    }

    /// Creates a new [`S3StorageService`] from the configuration object given.
    pub fn new(config: S3StorageConfig) -> S3StorageService {
        let mut sdk_config = SdkConfig::builder();
        sdk_config.set_credentials_provider(Some(SharedCredentialsProvider::new(
            Credentials::new(
                config.access_key_id(),
                config.secret_access_key(),
                None,
                None,
                "remi",
            ),
        )));

        sdk_config.set_region(Some(config.region()));
        sdk_config.set_endpoint_url(Some(config.endpoint()));

        let sdk_config = sdk_config.build();
        S3StorageService {
            config,
            client: Client::new(&sdk_config),
        }
    }

    pub(crate) fn resolve_path<P: AsRef<Path>>(&self, path: P) -> String {
        let config = self.config.clone();
        match config.prefix() {
            Some(p) => format!("{p}/{}", path.as_ref().display()),
            None => path.as_ref().to_string_lossy().clone().to_string(),
        }
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    async fn init(&self) -> Result<()> {
        info!("initializing the s3 storage service...");

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
            warn!("bucket [{bucket}] doesn't exist, creating!");

            self.client
                .create_bucket()
                .bucket(bucket.clone())
                .acl(self.config.default_bucket_acl())
                .send()
                .await
                .map_err(|x| to_io_error!(x))?;
        }

        Ok(())
    }

    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>> {
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

    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>> {
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
                let bytes = self.open(path).await?;

                // shouldn't happen but h
                if bytes.is_none() {
                    return Ok(None);
                }

                let bytes = bytes.unwrap();

                // let last_modified_at = obj.last_modified();

                Ok(Some(Blob::File(FileBlob::new(
                    None,
                    obj.content_type().map(|x| x.to_owned()),
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

    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        _path: Option<P>,
        _options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        Ok(vec![])
    }

    async fn delete<P: AsRef<Path> + Send>(&self, _path: P) -> Result<()> {
        Ok(())
    }

    async fn exists<P: AsRef<Path> + Send>(&self, _path: P) -> Result<bool> {
        Ok(true)
    }

    async fn upload<P: AsRef<Path> + Send>(&self, _path: P, _options: UploadRequest) -> Result<()> {
        Ok(())
    }
}
