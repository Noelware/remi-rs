// 🐻‍❄️🧶 remi-rs: Asynchronous Rust crate to handle communication between applications and object storage providers
// Copyright (c) 2022-2025 Noelware, LLC. <team@noelware.org>
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
use azure_core::request_options::Metadata;
use azure_storage::{ErrorKind, ResultExt};
use azure_storage_blobs::prelude::ContainerClient;
use bytes::Bytes;
use futures_util::StreamExt;
use remi::{Blob, File, ListBlobsRequest, UploadRequest};
use std::{borrow::Cow, ops::Deref, path::Path, time::SystemTime};

#[derive(Debug, Clone)]
pub struct StorageService {
    container: ContainerClient,

    #[allow(unused)]
    config: StorageConfig,
}

impl StorageService {
    /// Creates a new [`StorageService`] with a provided [`StorageConfig`].
    pub fn new(config: StorageConfig) -> Result<StorageService, azure_core::Error> {
        Ok(Self {
            container: config.clone().try_into()?,
            config,
        })
    }

    /// Creates a new [`StorageService`] with an existing [`ContainerClient`].
    pub fn with_container_client(container: ContainerClient) -> StorageService {
        Self {
            container,
            config: StorageConfig::dummy(),
        }
    }

    fn sanitize_path<P: AsRef<Path> + Send>(&self, path: P) -> azure_core::Result<String> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| azure_core::Error::new(ErrorKind::Other, "was not valid utf-8"))
            .with_context(ErrorKind::Other, || "failed to convert path into a string")?;

        let path = path.trim_start_matches("./").trim_start_matches("~/");
        Ok(path.into())
    }
}

impl Deref for StorageService {
    type Target = ContainerClient;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

#[async_trait]
impl remi::StorageService for StorageService {
    type Error = azure_core::Error;

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("remi:azure")
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.init",
            skip_all,
            fields(
                remi.service = "azure",
                container = self.config.container,
            )
        )
    )]
    async fn init(&self) -> Result<(), Self::Error> {
        if self.container.exists().await? {
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        ::tracing::info!("creating blob container as it doesn't exist");

        #[cfg(feature = "log")]
        ::log::info!(
            "creating blob container [{}] as it doesn't exist",
            self.config.container
        );

        self.container.create().await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.open",
            skip_all,
            fields(
                remi.service = "azure",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>, Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            container = self.config.container,
            path = %path.display(),
            "opening blob in container"
        );

        #[cfg(feature = "log")]
        ::log::info!(
            "opening blob [{}] in container [{}]",
            path.display(),
            self.config.container
        );

        let client = self.container.blob_client(self.sanitize_path(path)?);
        if !client.exists().await? {
            return Ok(None);
        }

        client.get_content().await.map(|content| Some(From::from(content)))
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.blob",
            skip_all,
            fields(
                remi.service = "azure",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>, Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            container = self.config.container,
            path = %path.display(),
            "opening blob in container"
        );

        #[cfg(feature = "log")]
        ::log::info!(
            "opening blob [{}] in container [{}]",
            path.display(),
            self.config.container
        );

        let client = self.container.blob_client(self.sanitize_path(path)?);
        if !client.exists().await? {
            return Ok(None);
        }

        let props = client.get_properties().await?;
        let data = Bytes::from(client.get_content().await?);

        Ok(Some(Blob::File(File {
            last_modified_at: {
                let last_modified: SystemTime = props.blob.properties.last_modified.into();
                Some(
                    last_modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime overflow?!")
                        .as_millis(),
                )
            },
            metadata: props.blob.metadata.unwrap_or_default(),
            content_type: Some(props.blob.properties.content_type),
            created_at: {
                let created_at: SystemTime = props.blob.properties.creation_time.into();
                Some(
                    created_at
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime overflow?!")
                        .as_millis(),
                )
            },
            is_symlink: false,
            data,
            path: format!("azure://{}", props.blob.name),
            name: props.blob.name,
            size: props.blob.properties.content_length.try_into().map_err(|e| {
                azure_core::Error::new(
                    azure_core::error::ErrorKind::Other,
                    format!("expected content length to fit into `usize`: {e}"),
                )
            })?,
        })))
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.blobs",
            skip_all,
            fields(
                remi.service = "azure"
            )
        )
    )]
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        request: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>, Self::Error> {
        let options = request.unwrap_or_default();
        let mut blobs = self.container.list_blobs();
        match path {
            Some(path) => {
                let path = self.sanitize_path(path)?;
                blobs = blobs.prefix(path);
            }

            None => {
                if let Some(prefix) = options.prefix {
                    blobs = blobs.prefix(prefix);
                }
            }
        }

        let mut stream = blobs.into_stream();
        let mut blobs = vec![];
        while let Some(value) = stream.next().await {
            let data = value?;
            for blob in data.blobs.blobs() {
                blobs.push(Blob::File(File {
                    last_modified_at: {
                        let last_modified: SystemTime = blob.properties.last_modified.into();
                        Some(
                            last_modified
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .expect("SystemTime overflow?!")
                                .as_millis(),
                        )
                    },
                    metadata: blob.metadata.clone().unwrap_or_default(),
                    content_type: Some(blob.properties.content_type.clone()),
                    created_at: {
                        let created_at: SystemTime = blob.properties.creation_time.into();
                        Some(
                            created_at
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .expect("SystemTime overflow?!")
                                .as_millis(),
                        )
                    },
                    is_symlink: false,
                    data: self.open(&blob.name).await?.unwrap(),
                    path: format!("azure://{}", blob.name),
                    name: blob.name.clone(),
                    size: blob.properties.content_length.try_into().map_err(|e| {
                        azure_core::Error::new(
                            azure_core::error::ErrorKind::Other,
                            format!("expected content length to fit into `usize`: {e}"),
                        )
                    })?,
                }));
            }
        }

        Ok(blobs)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.delete",
            skip_all,
            fields(
                remi.service = "azure",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<(), Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            container = self.config.container,
            path = %path.display(),
            "deleting blob in container"
        );

        #[cfg(feature = "log")]
        ::log::info!(
            "deleting blob [{}] in container [{}]",
            path.display(),
            self.config.container
        );

        let client = self.container.blob_client(self.sanitize_path(path)?);
        if !client.exists().await? {
            return Ok(());
        }

        client.delete().await.map(|_| ())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.exists",
            skip_all,
            fields(
                remi.service = "azure",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool, Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            container = self.config.container,
            path = %path.display(),
            "checking if blob is in container"
        );

        #[cfg(feature = "log")]
        ::log::info!(
            "checking if blob [{}] is in container [{}]",
            path.display(),
            self.config.container
        );

        self.container.blob_client(self.sanitize_path(path)?).exists().await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.blob",
            skip_all,
            fields(
                remi.service = "azure",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> Result<(), Self::Error> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            container = self.config.container,
            path = %path.display(),
            "uploading blob to container"
        );

        #[cfg(feature = "log")]
        ::log::info!(
            "uploading blob [{}] into container [{}]",
            path.display(),
            self.config.container
        );

        let client = self.container.blob_client(self.sanitize_path(path)?);
        if client.exists().await? {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(
                container = self.config.container,
                path = %path.display(),
                "blob with path already exists in container, skipping"
            );

            #[cfg(feature = "log")]
            ::log::info!(
                "blob with path [{}] already exist in container [{}], skipping",
                path.display(),
                self.config.container
            );

            return Ok(());
        }

        let mut blob = client.put_block_blob(options.data);
        if let Some(ct) = options.content_type {
            blob = blob.content_type(ct);
        }

        let mut metadata = Metadata::new();
        for (key, value) in options.metadata.clone() {
            metadata.insert(key.as_str(), remi::Bytes::from(value));
        }

        blob.metadata(metadata).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::{StorageConfig, StorageService};

    #[test]
    fn sanitize_paths() {
        let storage = StorageService::new(StorageConfig::dummy()).unwrap();

        assert_eq!(storage.sanitize_path("./weow.txt").unwrap(), String::from("weow.txt"));
        assert_eq!(storage.sanitize_path("~/weow.txt").unwrap(), String::from("weow.txt"));
        assert_eq!(storage.sanitize_path("weow.txt").unwrap(), String::from("weow.txt"));
        assert_eq!(
            storage.sanitize_path("~/weow/fluff/mooo.exe").unwrap(),
            String::from("weow/fluff/mooo.exe")
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{Credential, StorageConfig};
//     use azure_storage::CloudLocation;
//     use bollard::Docker;
//     use remi::{StorageService, UploadRequest};
//     use testcontainers::{runners::AsyncRunner, GenericImage, ImageExt};
//     use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//     const IMAGE: &str = "mcr.microsoft.com/azure-storage/azurite";

//     // renovate: image="microsoft-azure-storage-azurite"
//     const TAG: &str = "3.31.0";

//     fn container() -> GenericImage {
//         GenericImage::new(IMAGE, TAG)
//     }

//     #[test]
//     fn test_sanitize_paths() {
//         let storage = crate::StorageService::new(StorageConfig::dummy()).unwrap();
//         assert_eq!(storage.sanitize_path("./weow.txt").unwrap(), String::from("weow.txt"));
//         assert_eq!(storage.sanitize_path("~/weow.txt").unwrap(), String::from("weow.txt"));
//         assert_eq!(storage.sanitize_path("weow.txt").unwrap(), String::from("weow.txt"));
//         assert_eq!(
//             storage.sanitize_path("~/weow/fluff/mooo.exe").unwrap(),
//             String::from("weow/fluff/mooo.exe")
//         );
//     }

//     macro_rules! build_testcases {
//         (
//             $(
//                 $(#[$meta:meta])*
//                 async fn $name:ident($storage:ident) $code:block
//             )*
//         ) => {
//             $(
//                 #[cfg_attr(target_os = "linux", tokio::test)]
//                 #[cfg_attr(not(target_os = "linux"), ignore = "azurite image can be only used on Linux")]
//                 $(#[$meta])*
//                 async fn $name() {
//                     // if any time we can't probe docker, then we cannot continue
//                     if Docker::connect_with_defaults().is_err() {
//                         eprintln!("[remi-azure] `docker` cannot be probed by default settings; skipping test");
//                         return;
//                     }

//                     let _guard = tracing_subscriber::registry()
//                         .with(tracing_subscriber::fmt::layer())
//                         .set_default();

//                     let req: ::testcontainers::ContainerRequest<GenericImage> = container()
//                         .with_cmd(["azurite-blob", "--blobHost", "0.0.0.0"])
//                         .into();

//                     let container = req.start().await.expect("failed to start container");
//                     let $storage = crate::StorageService::new(StorageConfig {
//                         container: String::from("test-container"),
//                         credentials: Credential::AccessKey {
//                             account: String::from("devstoreaccount1"),
//                             access_key: String::from(
//                                 "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==",
//                             ),
//                         },
//                         location: CloudLocation::Emulator {
//                             address: container.get_host().await.expect("failed to get host ip for container").to_string(),
//                             port: container.get_host_port_ipv4(10000).await.expect("failed to get mapped port `10000`"),
//                         },
//                     }).unwrap();

//                     ($storage).init().await.expect("failed to initialize storage service");

//                     let __ret = $code;
//                     __ret
//                 }
//             )*
//         };
//     }

//     build_testcases! {
//         async fn prepare_azurite_container_usage(storage) {
//         }

//         async fn test_uploading_file(storage) {
//             let contents: remi::Bytes = "{\"wuff\":true}".into();
//             storage.upload("./wuff.json", UploadRequest::default()
//                 .with_content_type(Some("application/json"))
//                 .with_data(contents.clone())
//             ).await.expect("failed to upload");

//             assert!(storage.exists("./wuff.json").await.expect("failed to query ./wuff.json"));
//             assert_eq!(contents, storage.open("./wuff.json").await.expect("failed to open ./wuff.json").expect("it should exist"));
//         }

//         async fn list_blobs(storage) {
//             for i in 0..100 {
//                 let contents: remi::Bytes = format!("{{\"blob\":{i}}}").into();
//                 storage.upload(format!("./wuff.{i}.json"), UploadRequest::default()
//                     .with_content_type(Some("application/json"))
//                     .with_data(contents)
//                 ).await.expect("failed to upload blob");
//             }

//             let blobs = storage.blobs(None::<&str>, None).await.expect("failed to list all blobs");
//             let iter = blobs.iter().filter_map(|x| match x {
//                 remi::Blob::File(file) => Some(file),
//                 _ => None
//             });

//             assert!(iter.clone().all(|x|
//                 x.content_type == Some(String::from("application/json")) &&
//                 !x.is_symlink &&
//                 x.data.starts_with(&[/* b"{" */ 123])
//             ));
//         }

//         async fn query_single_blob(storage) {
//             for i in 0..100 {
//                 let contents: remi::Bytes = format!("{{\"blob\":{i}}}").into();
//                 storage.upload(format!("./wuff.{i}.json"), UploadRequest::default()
//                     .with_content_type(Some("application/json"))
//                     .with_data(contents)
//                 ).await.expect("failed to upload blob");
//             }

//             assert!(storage.blob("./wuff.98.json").await.expect("failed to query single blob").is_some());
//             assert!(storage.blob("./wuff.95.json").await.expect("failed to query single blob").is_some());
//             assert!(storage.blob("~/doesnt/exist").await.expect("failed to query single blob").is_none());
//         }
//     }
// }
