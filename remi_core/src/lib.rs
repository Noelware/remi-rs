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

#![doc = include_str!("../README.md")]

use std::{io::Result, path::Path};

pub use async_trait::async_trait;
use blob::Blob;
use builders::{ListBlobsRequest, UploadRequest};
use bytes::Bytes;

#[deprecated(
    since = "0.2.0",
    reason = "Use remi_core::Blob instead of remi_core::blob::Blob. Scheduled to be removed in 0.3.0"
)]
pub mod blob;

#[deprecated(
    since = "0.2.0",
    reason = "Builder APIs will now be in global module (remi_core::ListRequestBuilder) instead of builders module. Scheduled to be removed in 0.3.0"
)]
pub mod builders;

pub use blob::*;
pub use builders::*;

/// `StorageService` is the base primitive for implementing a storage backend. This is the main trait
/// you should implement if you're creating your own storage backend with **remi-rs**. Please refer to the
/// crate documentation for an example on how to implement your own.
#[async_trait]
pub trait StorageService {
    async fn init(&self) -> Result<()>;

    /// Opens the file in the given path and returns a Result of the given inner data of the file, if it exists. If the
    /// result was `Ok`, then the inner type will be the [`Bytes`] that the file is contained, or a `Option::None` variant
    /// which can happen if:
    ///
    /// - the path is a directory
    /// - the path doesn't exist.
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>>;

    /// Returns the [`Blob`] if the given path exists on the disk or on the cloud storage provider.
    ///
    /// ```no_run
    /// # use remi::filesystem::FilesystemStorageService;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #   let service = FilesystemStorageService::new("./.data");
    /// #   service.init().await?;
    /// #
    /// service.blobs(None);
    /// // => Ok(vec![Blob {}, Blob{}])
    /// # }
    /// ```
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>>;
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>>;

    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<()>;
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool>;
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> Result<()>;
}
