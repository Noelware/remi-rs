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

use bytes::Bytes;
#[cfg(feature = "log")]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "tracing")]
use tracing::{debug, error, info, trace, warn};

use crate::S3StorageConfig;
use async_trait::async_trait;
use aws_sdk_s3::{
    types::{BucketCannedAcl, Object},
    Client, Config,
};
use remi::{Blob, Directory, ListBlobsRequest, StorageService, UploadRequest};
use std::{io, path::Path};

macro_rules! to_io_error {
    ($x:expr) => {
        ::std::io::Error::new(::std::io::ErrorKind::Other, $x)
    };
}

/// Represents an implementation of [`StorageService`] for Amazon Simple Storage Service.
#[derive(Debug, Clone)]
pub struct S3StorageService {
    client: Client,
    config: S3StorageConfig,
}

impl S3StorageService {
    /// Creates a [`S3StorageService`] with a given storage service configuration.
    pub fn new(config: S3StorageConfig) -> S3StorageService {
        let client = Client::from_conf(config.clone().into());
        S3StorageService { client, config }
    }

    /// Creates a new [`S3StorageService`] with a implementator of [`Config`] that can
    /// represent the AWS SDK S3 configuration that you want.
    pub fn with_sdk_conf<I: Into<Config>>(config: I) -> S3StorageService {
        let client = Client::from_conf(config.into());
        S3StorageService {
            client,
            config: S3StorageConfig::default(),
        }
    }

    /// Overwrites a [`S3StorageConfig`] instance on this service without modifying the
    /// actual SDK client. This is useful if you used the [`S3StorageService::with_sdk_conf`]
    /// method. If you wish to modify the SDK client with a [`S3StorageConfig`],
    /// then use the [`S3StorageConfig::new`] method instead.
    pub fn with_config(self, config: S3StorageConfig) -> S3StorageService {
        S3StorageService {
            client: self.client,
            config,
        }
    }

    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> String {
        match self.config.prefix {
            Some(ref prefix) => format!("{prefix}/{}", path.as_ref().display()),
            None => path.as_ref().display().to_string(),
        }
    }

    async fn s3_obj_to_blob(&self, entry: &Object) -> Result<Option<Blob>, io::Error> {
        match entry.key() {
            Some(key) if key.ends_with('/') => Ok(Some(Blob::Directory(Directory {
                created_at: None,
                name: key.to_owned(),
                path: format!("s3://{key}"),
            }))),

            Some(key) => self.blob(key).await,
            None => Ok(None),
        }
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    const NAME: &'static str = "remi:s3";

    #[cfg_attr(feature = "tracing", ::tracing::instrument(name = "remi.s3.init", skip_all, remi.service = "s3", bucket = self.config.bucket))]
    async fn init(&self) -> io::Result<()> {
        #[cfg(feature = "log")]
        info!("ensuring that bucket [{}] exists!", self.config.bucket);

        #[cfg(feature = "tracing")]
        info!(
            remi.service = "s3",
            bucket = self.config.bucket,
            "ensuring that bucket exists"
        );

        let output = self.client.list_buckets().send().await.map_err(|x| to_io_error!(x))?;

        if !output.buckets().iter().any(|x| match x.name() {
            Some(name) => name == self.config.bucket,
            None => false,
        }) {
            #[cfg(feature = "log")]
            info!(
                "creating bucket [{}] due to no bucket existing on this AWS account",
                self.config.bucket
            );

            #[cfg(feature = "tracing")]
            info!(
                remi.service = "s3",
                bucket = self.config.bucket,
                "creating bucket due to the bucket not existing on this AWS account"
            );

            #[allow(unused)]
            self.client
                .create_bucket()
                .bucket(&self.config.bucket)
                .acl(
                    self.config
                        .default_bucket_acl
                        .clone()
                        .unwrap_or(BucketCannedAcl::Private),
                )
                .send()
                .await
                .map(|output| {
                    #[cfg(feature = "log")]
                    info!("bucket [{}] was created successfully", self.config.bucket);

                    #[cfg(feature = "log")]
                    trace!("{output:?}");

                    #[cfg(feature = "tracing")]
                    info!(
                        remi.service = "s3",
                        bucket = self.config.bucket,
                        "bucket was created successfully"
                    );

                    #[cfg(feature = "tracing")]
                    trace!(remi.service = "s3", bucket = self.config.bucket, "{output:?}");
                })
                .map_err(|x| to_io_error!(x))?;
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(name = "remi.s3.open", skip(self), remi.service = "s3", path = tracing::field::display(path.display()))
    )]
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Bytes>> {
        Ok(None)
    }

    /*
    async fn open(&self, path: impl AsRef<Path> + Send) -> Result<Option<Bytes>> {
        let normalized = self.resolve_path(path.as_ref());

        #[cfg(feature = "log")]
        trace!("opening file {normalized}...");

        let obj = self
            .client
            .get_object()
            .bucket(self.config.bucket.clone())
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
     */

    /// Returns a [`Blob`] instance of the given file or directory, if it exists.
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Blob>> {
        Ok(None)
    }

    /// Similar to [`blob`](StorageService::blob) but returns a list of blobs that exist
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> io::Result<Vec<Blob>> {
        Ok(vec![])
    }

    /// Deletes a path.
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<()> {
        Ok(())
    }

    /// Checks whether or not if a path exists.
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<bool> {
        Ok(false)
    }

    /// Uploads a path.
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> io::Result<()> {
        Ok(())
    }
}

/*
#[async_trait]
impl StorageService for S3StorageService {
    async fn open(&self, path: impl AsRef<Path> + Send) -> Result<Option<Bytes>> {
        let normalized = self.resolve_path(path.as_ref());

        #[cfg(feature = "log")]
        trace!("opening file {normalized}...");

        let obj = self
            .client
            .get_object()
            .bucket(self.config.bucket.clone())
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

        #[cfg(feature = "log")]
        trace!("opening file [{normalized}]");

        let obj = self
            .client
            .get_object()
            .bucket(self.config.bucket.clone())
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
                .bucket(self.config.bucket.clone())
                .max_keys(1000); // TODO: add this in ListBlobsRequest?

            loop {
                let resp = req.clone().send().await.map_err(|x| to_io_error!(x))?;
                let entries = resp.contents().unwrap_or_default();

                #[cfg(feature = "log")]
                trace!("found {} entries", entries.len());

                for entry in entries {
                    let name = entry.key();
                    if name.is_none() {
                        #[cfg(feature = "log")]
                        trace!("skipping entry due to no name");

                        continue;
                    }

                    let name = name.unwrap();
                    if options.is_excluded(name) {
                        #[cfg(feature = "log")]
                        debug!("S3 object with key [{name}] is being excluded from output");
                    }

                    match self.s3_obj_to_blob(entry).await {
                        Ok(Some(blob)) => {
                            blobs.push(blob);
                        }

                        #[cfg(feature = "log")]
                        Err(e) => {
                            warn!("skipping error [{e}] when listing objects, object [{name:?}] will not be present in the final result");
                            continue;
                        }

                        #[cfg(not(feature = "log"))]
                        Err(_) => continue,
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
            .bucket(self.config.bucket.clone())
            .max_keys(1000) // TODO: add this in ListBlobsRequest?
            .set_prefix(Some(resolved));

        loop {
            let resp = req.clone().send().await.map_err(|x| to_io_error!(x))?;
            let entries = resp.contents().unwrap_or_default();

            #[cfg(feature = "log")]
            trace!("found {} entries", entries.len());

            for entry in entries {
                let name = entry.key();
                if name.is_none() {
                    #[cfg(feature = "log")]
                    trace!("skipping entry due to no name");

                    continue;
                }

                let name = name.unwrap();
                if options.is_excluded(name) {
                    #[cfg(feature = "log")]
                    debug!("S3 object with key [{name}] is being excluded from output");
                }

                match self.s3_obj_to_blob(entry).await {
                    Ok(Some(blob)) => {
                        blobs.push(blob);
                    }

                    #[cfg(feature = "log")]
                    Err(e) => {
                        warn!("skipping error [{e}] when listing objects, object [{name:?}] will not be present in the final result");
                        continue;
                    }

                    #[cfg(not(feature = "log"))]
                    Err(_) => continue,
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
            .bucket(self.config.bucket.clone())
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
            .bucket(self.config.bucket.clone())
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

        #[cfg(feature = "log")]
        trace!("uploading object [{path}] with content type [{content_type:?}]");
        let len = options.data.len();
        let stream: ByteStream = options.data.into();

        self.client
            .put_object()
            .bucket(self.config.bucket.clone())
            .key(path)
            .acl(
                self.config
                    .default_object_acl
                    .clone()
                    .unwrap_or(ObjectCannedAcl::BucketOwnerFullControl),
            )
            .body(stream)
            .content_type(content_type)
            .content_length(len as i64)
            .set_metadata(match options.metadata.is_empty() {
                true => None,
                false => Some(options.metadata.clone()),
            })
            .send()
            .await
            .map(|_| ())
            .map_err(|x| to_io_error!(x))
    }
}
*/
