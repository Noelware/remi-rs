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

//! # 🐻‍❄️🧶 `remi_s3`
//! The **remi_s3** crate is an official implementation of the [`remi::StorageService`]
//! trait with Amazon S3 using the official AWS crate [`aws_sdk_s3`].
//!
//! [`remi::StorageSerive`]: https://docs.rs/remi/*/remi/trait.StorageService.html
//! [`aws_sdk_s3`]: https://docs.rs/aws-sdk-s3
//!
//! ## Example
//! ```rust,no_run
//! // Cargo.toml:
//! //
//! // [dependencies]
//! // remi = "^0"
//! // remi-s3 = { version = "^0", features = ["export-crates"] }
//! // tokio = { version = "^1", features = ["full"] }
//!
//! use remi_s3::{StorageService, StorageConfig, aws::s3};
//! use remi::{StorageService as _, UploadRequest};
//!
//! #[tokio::main]
//! async fn main() {
//! }
//! ```
//!
//! ## Crate Features
//! | Crate Features  | Description                                                                          | Enabled by default? |
//! | :-------------- | :----------------------------------------------------------------------------------- | ------------------- |
//! | `export-crates` | Exports all the used AWS crates as a module called `aws`                             | No.                 |
//! | `unstable`      | Tap into unstable features from `remi_gridfs` and the `remi` crate.                  | No.                 |
//! | [`tracing`]     | Enables the use of [`tracing::instrument`] and emit events for actions by the crate. | No.                 |
//! | [`serde`]       | Enables the use of **serde** in `StorageConfig`                                      | No.                 |
//! | [`log`]         | Emits log records for actions by the crate                                           | No.                 |
//!
//! [`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
//! [`tracing`]: https://crates.io/crates/tracing
//! [`serde`]: https://serde.rs
//! [`log`]: https://crates.io/crates/log

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc(html_favicon_url = "https://cdn.floofy.dev/images/trans.png")]
#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]

mod config;
mod error;
mod service;

pub use config::*;
pub use error::*;
pub use service::*;

/// Exports the [`aws_sdk_s3`], [`aws_credential_types`], and [`aws_config`] crate without
/// specifying the dependencies yourself.
#[cfg(feature = "export-crates")]
#[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "export-crates")))]
pub mod aws {
    pub use aws_config as config;
    pub use aws_credential_types as credential_types;
    pub use aws_sdk_s3 as s3;
}
