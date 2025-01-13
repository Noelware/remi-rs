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

//! # 🐻‍❄️🧶 `remi_gridfs`
//! This crate is an official implementation of [`remi::StorageService`] with the official
//! [`mongodb`] crate by the MongoDB Rust Driver team.
//!
//! [`remi::StorageSerive`]: https://docs.rs/remi/*/remi/trait.StorageService.html
//! [`mongodb`]: https://docs.rs/mongodb
//!
//! ## Example
//! ```rust,no_run
//! // Cargo.toml:
//! //
//! // [dependencies]
//! // remi = "^0"
//! // remi-gridfs = { version = "^0", features = ["export-crates"] }
//! // tokio = { version = "^1", features = ["full"] }
//!
//! use remi_gridfs::{StorageService, StorageConfig, mongodb};
//! use remi::{StorageService as _, UploadRequest};
//!
//! #[tokio::main]
//! async fn main() {
//!     let storage = StorageService::from_conn_string("mongodb://localhost:27017", StorageConfig {
//!         bucket: "my-bucket".into(),
//!
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     // Initialize the container. This will:
//!     //
//!     // * create the `my-bucket` GridFS bucket if it doesn't exist
//!     storage.init().await.unwrap();
//!
//!     // Now we can upload files to GridFS.
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
//! | Crate Features  | Description                                                                          | Enabled by default? |
//! | :-------------- | :----------------------------------------------------------------------------------- | ------------------- |
//! | `export-crates` | Exports all the used MongoDB crates as a module called `mongodb`                     | No.                |
//! | `unstable`      | Tap into unstable features from `remi_gridfs` and the `remi` crate.                  | No.                 |
//! | [`tracing`]     | Enables the use of [`tracing::instrument`] and emit events for actions by the crate. | No.                 |
//! | [`serde`]       | Enables the use of **serde** in `StorageConfig`                                      | No.                 |
//! | [`log`]         | Emits log records for actions by the crate                                           | No.                 | | Crate Features  | Description                                                                          | Enabled by default? |
//!
//! [`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
//! [`tracing`]: https://crates.io/crates/tracing
//! [`serde`]: https://serde.rs
//! [`log`]: https://crates.io/crates/log

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc(html_favicon_url = "https://cdn.floofy.dev/images/trans.png")]
#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]

mod config;
mod service;

pub use config::*;
pub use service::*;

/// Exports the [`mongodb`] crate without specifying the dependency yourself.
#[cfg(feature = "export-crates")]
#[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "export-crates")))]
pub use mongodb;
