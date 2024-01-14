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
use azure_storage_blobs::prelude::ContainerClient;
use bytes::Bytes;
use remi::{Blob, ListBlobsRequest, UploadRequest};
use std::{io, path::Path};

#[derive(Debug, Clone)]
pub struct StorageService(ContainerClient);

impl StorageService {
    /// Creates a new [`StorageService`] with a provided [`StorageConfig`].
    pub fn new(config: StorageConfig) -> StorageService {
        Self::with_client(config)
    }

    /// Creates a new [`StorageService`] with a preconfigured [`ContainerClient`].
    pub fn with_client<C: Into<ContainerClient>>(client: C) -> StorageService {
        StorageService(client.into())
    }
}

#[async_trait]
impl remi::StorageService for StorageService {
    const NAME: &'static str = "remi:azure";

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
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Bytes>> {
        unimplemented!()
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
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Blob>> {
        unimplemented!()
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
        _request: Option<ListBlobsRequest>,
    ) -> io::Result<Vec<Blob>> {
        unimplemented!()
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
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<()> {
        unimplemented!()
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
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<bool> {
        unimplemented!()
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
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> io::Result<()> {
        unimplemented!()
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
