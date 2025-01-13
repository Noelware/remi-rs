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

//! # 🐻‍❄️🧶 `remi_azure`
//! This crate is an official implementation of [`remi::StorageService`] for Microsoft's
//! Azure Blob Storage service using the unofficial Azure crates: [`azure_core`], [`azure_storage`],
//! and [`azure_storage_blobs`].
//!
//! [`remi::StorageService`]: https://docs.rs/remi/*/remi/trait.StorageService.html
//! [`azure_storage_blobs`]: https://docs.rs/azure-storage-blobs
//! [`azure_storage`]: https://docs.rs/azure-storage
//! [`azure_core`]: https://docs.rs/azure-core
//!
//! ## Example
//! ```rust,no_run
//! // Cargo.toml:
//! //
//! // [dependencies]
//! // remi = "^0"
//! // remi-azure = "^0"
//! // tokio = { version = "^1", features = ["full"] }
//!
//! use remi_azure::{StorageService, StorageConfig, Credential, CloudLocation};
//! use remi::{StorageService as _, UploadRequest};
//!
//! #[tokio::main]
//! async fn main() {
//!     let storage = StorageService::new(StorageConfig {
//!         credentials: Credential::Anonymous,
//!         container: "my-container".into(),
//!         location: CloudLocation::Public("my-account".into()),
//!     }).unwrap();
//!
//!     // Initialize the container. This will:
//!     //
//!     // * create `my-container` if it doesn't exist
//!     storage.init().await.unwrap();
//!
//!     // Now we can upload files to Azure.
//!
//!     // We define a `UploadRequest`, which will set the content type to `text/plain` and set the
//!     // contents of `weow.txt` to `weow fluff`.
//!     let upload = UploadRequest::default()
//!         .with_content_type(Some("text/plain"))
//!         .with_data("weow fluff");
//!
//!     // Let's upload it!
//!     storage.upload("weow.txt", upload).await.unwrap();
//!
//!     // Let's check if it exists! This `assert!` will panic if it failed
//!     // to upload.
//!     assert!(storage.exists("weow.txt").await.unwrap());
//! }
//! ```
//!
//! ## Crate Features
//! | Crate Features | Description                                                                          | Enabled by default? |
//! | :------------- | :----------------------------------------------------------------------------------- | ------------------- |
//! | `export-azure` | Exports all the used Azure crates as a module called `core`                          | No.                 |
//! | `unstable`     | Tap into unstable features from `remi_azure` and the `remi` crate.                   | No.                 |
//! | [`tracing`]    | Enables the use of [`tracing::instrument`] and emit events for actions by the crate. | No.                 |
//! | [`serde`]      | Enables the use of **serde** in `StorageConfig`                                      | No.                 |
//! | [`log`]        | Emits log records for actions by the crate                                           | No.                 |
//!
//! [`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
//! [`tracing`]: https://crates.io/crates/tracing
//! [`serde`]: https://serde.rs
//! [`log`]: https://crates.io/crates/log

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc(html_favicon_url = "https://cdn.floofy.dev/images/trans.png")]
#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]

#[cfg(feature = "export-azure")]
#[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "export-azure")))]
/// Exports the [`azure_core`], [`azure_storage`], and [`azure_storage_blobs`]
/// crates without defining them as owned dependencies.
///
/// [`azure_storage_blobs`]: https://docs.rs/azure-storage-blobs
/// [`azure_storage`]: https://docs.rs/azure-storage
/// [`azure_core`]: https://docs.rs/azure-core
pub mod core {
    pub use azure_core::*;

    /// Exports the [`azure_storage`] and [`azure_storage_blobs`]
    /// crates without defining them as owned dependencies.
    ///
    /// [`azure_storage_blobs`]: https://docs.rs/azure-storage-blobs
    /// [`azure_storage`]: https://docs.rs/azure-storage
    #[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "export-azure")))]
    pub mod storage {
        pub use azure_storage::*;

        /// Exports the [`azure_storage_blobs`] crate without defining them as owned dependencies.
        ///
        /// [`azure_storage_blobs`]: https://docs.rs/azure-storage-blobs
        pub use azure_storage_blobs as blobs;
    }
}

mod config;
pub use config::*;

mod service;
pub use service::*;
