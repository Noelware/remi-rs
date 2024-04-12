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
use azure_core::request_options::Prefix;
use azure_storage_blobs::prelude::ContainerClient;
use bytes::Bytes;
use futures_util::StreamExt;
use remi::{Blob, File, ListBlobsRequest, UploadRequest};
use std::{ops::Deref, path::Path, time::SystemTime};

#[derive(Debug, Clone)]
pub struct StorageService {
    container: ContainerClient,

    #[allow(unused)]
    config: StorageConfig,
}

impl StorageService {
    /// Creates a new [`StorageService`] with a provided [`StorageConfig`].
    pub fn new(config: StorageConfig) -> StorageService {
        Self {
            container: config.clone().into(),
            config,
        }
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
    const NAME: &'static str = "remi:azure";

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "remi.azure.init",
            skip_all,
            fields(
                remi.service = "azure"
            )
        )
    )]
    async fn init(&self) -> Result<(), Self::Error> {
        if self.container.exists().await? {
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        ::tracing::info!(
            remi.service = "azure",
            container = self.config.container,
            "creating blob container as it doesn't exist"
        );

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
            remi.service = "azure",
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

        let client = self.container.blob_client(path.to_str().ok_or_else(|| {
            azure_core::Error::new(azure_core::error::ErrorKind::Other, "failed to convert path to UTF-8")
        })?);

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
            remi.service = "azure",
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

        let client = self.container.blob_client(path.to_str().ok_or_else(|| {
            azure_core::Error::new(azure_core::error::ErrorKind::Other, "failed to convert path to UTF-8")
        })?);

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

        let options = request.unwrap_or_default();
        let mut blobs = self.container.list_blobs();

        if let Some(prefix) = options.prefix {
            blobs = blobs.prefix(Prefix::from(prefix.clone()));
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
            remi.service = "azure",
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

        let client = self.container.blob_client(path.to_str().ok_or_else(|| {
            azure_core::Error::new(azure_core::error::ErrorKind::Other, "failed to convert path to UTF-8")
        })?);

        // file doesn't exist, skip right away
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
            remi.service = "azure",
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

        let client = self.container.blob_client(path.to_str().ok_or_else(|| {
            azure_core::Error::new(azure_core::error::ErrorKind::Other, "failed to convert path to UTF-8")
        })?);

        client.exists().await
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
            remi.service = "azure",
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

        let client = self.container.blob_client(path.to_str().ok_or_else(|| {
            azure_core::Error::new(azure_core::error::ErrorKind::Other, "failed to convert path to UTF-8")
        })?);

        if client.exists().await? {
            #[cfg(feature = "tracing")]
            ::tracing::warn!(
                remi.service = "azure",
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

        blob.await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use crate::{is_docker_enabled, StorageConfig};
    use azure_storage::CloudLocation;
    use testcontainers::{clients::Cli, GenericImage};

    fn get_azurite_container() -> GenericImage {
        GenericImage::new("mcr.microsoft.com/azure-storage/azurite", "3.29.0").with_exposed_port(10000)
    }

    #[test]
    fn check_if_azurite_container_can_run() {
        if !is_docker_enabled() {
            eprintln!("[remi-azure] `docker` is missing, cannot run test");
            return;
        }

        // in CI (GitHub Actions), it will pull the `azurite` image as a Windows container and Microsoft
        // doesn't ship Windows containers of the `azurite` image, so we just ignore the test alltogether
        //
        // TODO(@auguwu): how to fix this? :3
        //
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // running 2 tests
        // test service::tests::dbg_config ... ignored
        // 2024-01-15T01:40:26.004251Z DEBUG testcontainers::clients::cli: Executing command: "docker" "run" "--expose=10000" "-P" "-d" "mcr.microsoft.com/azure-storage/azurite:3.29.0" "azurite-blob" "--blobHost" "0.0.0.0"
        // 2024-01-15T01:40:33.899191Z ERROR testcontainers::clients::cli: Failed to start container.
        // Container stdout:
        // Container stderr: Unable to find image 'mcr.microsoft.com/azure-storage/azurite:3.29.0' locally
        // 3.29.0: Pulling from azure-storage/azurite
        // docker: no matching manifest for windows/amd64 10.0.20348 in the manifest list entries.
        // See 'docker run --help'.
        //
        // test service::tests::check_if_azurite_container_can_run ... FAILED
        //
        // failures:
        //
        // ---- service::tests::check_if_azurite_container_can_run stdout ----
        // thread 'service::tests::check_if_azurite_container_can_run' panicked at C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\testcontainers-0.15.0\src\clients\cli.rs:51:13:
        // Failed to start container, check log for details
        // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
        //
        // failures:
        //     service::tests::check_if_azurite_container_can_run
        //
        // test result: FAILED. 0 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 7.90s
        //
        // error: test failed, to rerun pass `-p remi-azure --lib`
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        if is_docker_enabled() && cfg!(windows) {
            eprintln!("[remi-azure] `docker` is enabled but it might not be configured with using Linux containers, cannot run test");
            return;
        }

        // testcontainers uses 'log' to output information, so we use tracing_subscriber
        // with `tracing_log` to output to stdout (useful for debugging)
        crate::setup_log_pipeline();

        let cli = Cli::default();
        let container = cli.run((
            get_azurite_container(),
            vec![
                String::from("azurite-blob"),
                String::from("--blobHost"),
                String::from("0.0.0.0"),
            ],
        ));

        eprintln!(
            "[remi-azure] container with id {} is running image {:?}",
            container.id(),
            container.image()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn dbg_config() {
        if !is_docker_enabled() {
            eprintln!("[remi-azure] `docker` is missing, cannot run test");
            return;
        }

        let cli = Cli::default();
        let container = cli.run((
            get_azurite_container(),
            vec![
                String::from("azurite-blob"),
                String::from("--blobHost"),
                String::from("0.0.0.0"),
            ],
        ));

        let config = StorageConfig {
            location: CloudLocation::Emulator {
                address: container.get_bridge_ip_address().to_string(),
                port: container.get_host_port_ipv4(10000),
            },

            container: "test-container".into(),
            credentials: crate::Credential::AccessKey {
                account: "devstoreaccount1".into(),
                access_key: "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw=="
                    .into(),
            },
        };

        dbg!(config);
    }
}
