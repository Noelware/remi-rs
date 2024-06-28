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
use aws_sdk_s3::{
    primitives::ByteStream,
    types::{BucketCannedAcl, Object, ObjectCannedAcl},
    Client, Config,
};
use bytes::Bytes;
use remi::{Blob, Directory, File, ListBlobsRequest, UploadRequest};
use std::{borrow::Cow, path::Path};

const DEFAULT_CONTENT_TYPE: &str = "application/octet; charset=utf-8";

/// Represents an implementation of [`StorageService`] for Amazon Simple Storage Service.
#[derive(Debug, Clone)]
pub struct StorageService {
    client: Client,
    config: StorageConfig,
}

impl StorageService {
    /// Creates a [`StorageService`] with a given storage service configuration.
    pub fn new(config: StorageConfig) -> StorageService {
        let client = Client::from_conf(From::from(config.clone()));
        StorageService { client, config }
    }

    /// Creates a new [`StorageService`] with a implementator of [`Config`] that can
    /// represent the AWS SDK S3 configuration that you want.
    pub fn with_sdk_conf<I: Into<Config>>(config: I) -> StorageService {
        let client = Client::from_conf(config.into());
        StorageService {
            client,
            config: StorageConfig::default(),
        }
    }

    /// Overwrites a [`S3StorageConfig`] instance on this service without modifying the
    /// actual SDK client. This is useful if you used the [`StorageService::with_sdk_conf`]
    /// method. If you wish to modify the SDK client with a [`S3StorageConfig`],
    /// then use the [`S3StorageConfig::new`] method instead.
    pub fn with_config(self, config: StorageConfig) -> StorageService {
        StorageService {
            client: self.client,
            config,
        }
    }

    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> crate::Result<String> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| crate::error::lib("expected valud a utf-8 string as the path"))?;

        // trim `./` and `~/` since S3 doesn't accept ./ or ~/ as valid paths
        let path = path.trim_start_matches("~/").trim_start_matches("./");
        let prefix = self.config.prefix.clone().unwrap_or_default();
        let prefix = prefix.trim_start_matches("~/").trim_start_matches("./");

        Ok(format!("{prefix}/{path}"))
    }

    async fn s3_obj_to_blob(&self, entry: &Object) -> crate::Result<Option<Blob>> {
        use remi::StorageService;

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
impl remi::StorageService for StorageService {
    // this has to stay `io::Error` since `SdkError` requires too much information
    // and this can narrow down.
    //
    // TODO(@auguwu): this can be a flat error if we could do?
    type Error = crate::Error;

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("remi:s3")
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.init",
            skip_all,
            fields(
                bucket = self.config.bucket,
                remi.service = "s3"
            )
        )
    )]
    async fn init(&self) -> crate::Result<()> {
        #[cfg(feature = "log")]
        log::info!("ensuring that bucket [{}] exists!", self.config.bucket);

        #[cfg(feature = "tracing")]
        tracing::info!(
            remi.service = "s3",
            bucket = self.config.bucket,
            "ensuring that bucket exists"
        );

        let output = self.client.list_buckets().send().await?;
        if !output.buckets().iter().any(|x| match x.name() {
            Some(name) => name == self.config.bucket,
            None => false,
        }) {
            #[cfg(feature = "log")]
            log::info!(
                "creating bucket [{}] due to no bucket existing on this AWS account",
                self.config.bucket
            );

            #[cfg(feature = "tracing")]
            tracing::info!(
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
                    log::info!("bucket [{}] was created successfully", self.config.bucket);

                    #[cfg(feature = "log")]
                    log::trace!("{output:?}");

                    #[cfg(feature = "tracing")]
                    tracing::info!(
                        remi.service = "s3",
                        bucket = self.config.bucket,
                        "bucket was created successfully"
                    );

                    #[cfg(feature = "tracing")]
                    tracing::trace!(remi.service = "s3", bucket = self.config.bucket, "{output:?}");
                })?;
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.blob.open",
            skip(self, path),
            fields(
                remi.service = "s3",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> crate::Result<Option<Bytes>> {
        let normalized = self.resolve_path(path)?;

        #[cfg(feature = "log")]
        log::trace!("opening file [{normalized}]");

        #[cfg(feature = "tracing")]
        tracing::trace!(remi.service = "s3", path = normalized, "opening file");

        let fut = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&normalized)
            .send();

        match fut.await {
            Ok(object) => {
                let stream = object.body;
                let data = stream.collect().await?.into_bytes();

                Ok(Some(data))
            }

            Err(e) => {
                let err = e.into_service_error();
                if err.is_no_such_key() {
                    return Ok(None);
                }

                Err(err.into())
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.blob.get",
            skip(self, path),
            fields(
                remi.service = "s3",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> crate::Result<Option<Blob>> {
        let normalized = self.resolve_path(path)?;

        #[cfg(feature = "log")]
        log::trace!("locating file [{normalized}]");

        #[cfg(feature = "tracing")]
        tracing::trace!(remi.service = "s3", path = normalized, "locating file");

        let fut = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&normalized)
            .send();

        match fut.await {
            Ok(object) => {
                // Get metadata before we read the body
                let content_type = object.content_type().map(|x| x.to_owned());
                let last_modified_at = object
                    .last_modified()
                    .map(|dt| dt.to_millis().expect("cant convert into millis") as u128);

                // Read the entire body of the object itself
                let stream = object.body;
                let data = stream.collect().await?.into_bytes();
                let size = data.len();

                Ok(Some(Blob::File(File {
                    last_modified_at,
                    metadata: object.metadata.clone().unwrap_or_default(),
                    content_type,
                    created_at: None,
                    is_symlink: false,
                    data,
                    name: normalized.clone(),
                    path: format!("s3://{normalized}"),
                    size,
                })))
            }

            Err(e) => {
                let err = e.into_service_error();
                if err.is_no_such_key() {
                    return Ok(None);
                }

                Err(err.into())
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.blob.list",
            skip(self, path),
            fields(
                remi.service = "s3",
                path = ?path.as_ref().map(|path| path.as_ref().display())
            )
        )
    )]
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> crate::Result<Vec<Blob>> {
        let options = options.unwrap_or_default();
        let mut blobs = Vec::new();
        let mut req = match path {
            Some(path) => self
                .client
                .list_objects_v2()
                .bucket(&self.config.bucket)
                .max_keys(1000)
                .prefix(self.resolve_path(path)?),

            None => {
                let mut req = self.client.list_objects_v2().bucket(&self.config.bucket).max_keys(1000);
                if let Some(ref prefix) = self.config.prefix {
                    req = req.prefix(prefix.trim_start_matches("~/").trim_end_matches("./"));
                }

                req
            }
        };

        loop {
            let resp = req.clone().send().await?;
            let entries = resp.contents();

            for entry in entries {
                let Some(name) = entry.key() else {
                    #[cfg(feature = "log")]
                    log::warn!("skipping entry due to no name");

                    #[cfg(feature = "log")]
                    log::trace!("{entry:?}");

                    #[cfg(feature = "tracing")]
                    tracing::warn!(remi.service = "s3", "skipping entry due to no name");

                    #[cfg(feature = "tracing")]
                    tracing::trace!("{entry:?}");

                    continue;
                };

                if options.is_excluded(name) {
                    #[cfg(feature = "log")]
                    log::warn!("excluding entry [{name}] due to options passed in");

                    #[cfg(feature = "log")]
                    log::trace!("{entry:?}");

                    #[cfg(feature = "tracing")]
                    tracing::warn!(remi.service = "s3", name, "skipping entry due to no name");

                    #[cfg(feature = "tracing")]
                    tracing::trace!("{entry:?}");

                    continue;
                }

                // most files include a '.'
                if !name.ends_with('/') && name.contains('.') {
                    let idx = name.chars().position(|x| x == '.');
                    if let Some(idx) = idx {
                        let ext = &name[idx + 1..];
                        if !options.is_ext_allowed(ext) {
                            #[cfg(feature = "log")]
                            log::warn!("excluding entry [{name}] due to extension [{ext}] not being allowed");

                            #[cfg(feature = "log")]
                            log::trace!("{entry:?}");

                            #[cfg(feature = "tracing")]
                            tracing::warn!(
                                remi.service = "s3",
                                name,
                                ext = &ext,
                                "skipping entry due to extension not being allowed"
                            );

                            #[cfg(feature = "tracing")]
                            tracing::trace!("{entry:?}");

                            continue;
                        }
                    }
                }

                match self.s3_obj_to_blob(entry).await {
                    Ok(Some(blob)) => blobs.push(blob),
                    Ok(None) => continue,

                    #[allow(unused)]
                    Err(e) => {
                        #[cfg(feature = "log")]
                        log::warn!("received SDK error when trying to getting blob information: {e}");

                        #[cfg(feature = "log")]
                        log::trace!("{entry:?}");

                        #[cfg(feature = "tracing")]
                        tracing::warn!(
                            remi.service = "s3",
                            name,
                            error = tracing::field::display(e),
                            "received SDK error when trying to getting blob information"
                        );

                        #[cfg(feature = "tracing")]
                        tracing::trace!("{entry:?}");

                        continue;
                    }
                }
            }

            match resp.continuation_token() {
                Some(token) => {
                    req = req.clone().continuation_token(token);
                }

                None => break,
            }
        }

        Ok(blobs)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.blob.delete",
            skip(self, path),
            fields(
                remi.service = "s3",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> crate::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(self.resolve_path(path)?)
            .send()
            .await
            .map(|_| ())
            .map_err(From::from)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.blob.exists",
            skip(self, path),
            fields(
                remi.service = "s3",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> crate::Result<bool> {
        let fut = self
            .client
            .head_object()
            .bucket(&self.config.bucket)
            .key(self.resolve_path(path)?)
            .send();

        match fut.await {
            Ok(res) => {
                if res.delete_marker().is_some() {
                    return Ok(false);
                }

                Ok(true)
            }
            Err(e) => {
                let inner = e.into_service_error();
                if inner.is_not_found() {
                    return Ok(false);
                }

                Err(inner.into())
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.s3.blob.upload",
            skip(self, path, options),
            fields(
                remi.service = "s3",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> crate::Result<()> {
        let normalized = self.resolve_path(path)?;
        let content_type = options.content_type.unwrap_or(DEFAULT_CONTENT_TYPE.into());

        #[cfg(feature = "log")]
        log::trace!("uploading object [{normalized}] with content type [{content_type}]");

        #[cfg(feature = "tracing")]
        tracing::trace!(
            remi.service = "s3",
            path = normalized,
            content_type,
            "uploading object with content type to Amazon S3"
        );

        let len = options.data.len();
        let stream = ByteStream::from(options.data);

        self.client
            .put_object()
            .bucket(self.config.bucket.clone())
            .key(normalized)
            .acl(
                self.config
                    .default_object_acl
                    .clone()
                    .unwrap_or(ObjectCannedAcl::BucketOwnerFullControl),
            )
            .body(stream)
            .content_type(content_type)
            .content_length(len.try_into().expect("unable to convert usize ~> i64"))
            .set_metadata(match options.metadata.is_empty() {
                true => None,
                false => Some(options.metadata.clone()),
            })
            .send()
            .await
            .map(|_| ())
            .map_err(From::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path() {
        let storage = StorageService::new(StorageConfig::default());
        assert_eq!(storage.resolve_path("./weow.txt").unwrap(), String::from("/weow.txt"));
        assert_eq!(storage.resolve_path("~/weow.txt").unwrap(), String::from("/weow.txt"));
        assert_eq!(storage.resolve_path("weow.txt").unwrap(), String::from("/weow.txt"));
        assert_eq!(
            storage.resolve_path("~/weow/fluff/wooo.exe").unwrap(),
            String::from("/weow/fluff/wooo.exe")
        );

        let storage = StorageService::new(StorageConfig {
            prefix: Some(String::from("/wow/epic/sauce")),
            ..Default::default()
        });

        assert_eq!(
            storage.resolve_path("./weow.txt").unwrap(),
            String::from("/wow/epic/sauce/weow.txt")
        );

        assert_eq!(
            storage.resolve_path("~/weow.txt").unwrap(),
            String::from("/wow/epic/sauce/weow.txt")
        );

        assert_eq!(
            storage.resolve_path("weow.txt").unwrap(),
            String::from("/wow/epic/sauce/weow.txt")
        );

        assert_eq!(
            storage.resolve_path("~/weow/fluff/wooo.exe").unwrap(),
            String::from("/wow/epic/sauce/weow/fluff/wooo.exe")
        );
    }
}
