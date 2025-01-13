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

//! # 🐻‍❄️🧶 `remi_fs`
//! This crate is an official implementation of [`remi::StorageService`] that uses
//! the local filesystem for operations.
//!
//! [`remi::StorageService`]: https://docs.rs/remi/*/remi/trait.StorageService.html
//!
//! ## Example
//! ```rust,no_run
//! // Cargo.toml:
//! //
//! // [dependencies]
//! // remi = "^0"
//! // remi-fs = "^0"
//! // tokio = { version = "^1", features = ["full"] }
//!
//! use remi_fs::{StorageService, StorageConfig};
//! use remi::{StorageService as _, UploadRequest};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize a `StorageService` that uses your local filesystem for storing files.
//!     let storage = StorageService::new("./data");
//!
//!     // Next, we will run the `init` function which will create
//!     // the ./data directory if it doesn't exist already.
//!     storage.init().await.unwrap();
//!
//!     // We define a `UploadRequest`, which will set the content type to `text/plain` and set the
//!     // contents of `weow.txt` to `weow fluff`.
//!     let upload = UploadRequest::default()
//!         .with_content_type(Some("text/plain"))
//!         .with_data("weow fluff");
//!
//!     // Let's upload it!
//!     storage.upload("./weow.txt", upload).await.unwrap();
//!
//!     // Let's check if it exists! This `assert!` will panic if it failed
//!     // to upload.
//!     assert!(storage.exists("./weow.txt").await.unwrap());
//! }
//! ```
//!
//! ## Crate Features
//! | Crate Features    | Description                                                                            | Enabled by default?  |
//! | :---------------- | :------------------------------------------------------------------------------------- | -------------------- |
//! | `unstable`        | Tap into unstable features from `remi_fs` and the `remi` crate.                        | No.                  |
//! | [`serde_yaml_ng`] | Allows to detect YAML documents with the [`serde_yaml_ng`] crate.                      | No.                  |
//! | [`serde_json`]    | Uses the [`serde_json`] crate to detect JSON documents and return `application/json`   | No.                  |
//! | [`file-format`]   | Uses the [`file-format`] crate to find media types on any external datatype.           | Yes.                 |
//! | [`tracing`]       | Enables the use of [`tracing::instrument`] and emit events for actions by the crate.   | No.                  |
//! | [`infer`]         | Uses the [`infer`] crate to infer external datatypes and map them to their media type. | Yes.                 |
//! | [`serde`]         | Enables the use of **serde** in `StorageConfig`                                        | No.                  |
//! | [`log`]           | Emits log records for actions by the crate                                             | No.                  |
//!
//! [`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
//! [`serde_yaml_ng`]: https://crates.io/crates/serde_yaml_ng
//! [`file-format`]: https://crates.io/crates/file-format
//! [`serde_json`]: https://crates.io/crates/serde_json
//! [`tracing`]: https://crates.io/crates/tracing
//! [`infer`]: https://crates.io/crates/infer
//! [`serde`]: https://serde.rs
//! [`log`]: https://crates.io/crates/log

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc(html_favicon_url = "https://cdn.floofy.dev/images/trans.png")]
#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]

mod config;
mod content_type;
mod service;

pub use config::*;
pub use content_type::*;
pub use service::*;
