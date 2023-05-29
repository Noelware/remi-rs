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

//! # 🐻‍❄️🧶 Amazon S3 support for remi-rs
//! **remi-s3** implements the [remi-core](https://github.com/Noelware/remi-rs/tree/master/remi_core) crate that interacts
//! with any Amazon S3 compatible server.
//!
//! > Note: This crate is also re-exported in the main [remi](https://crates.io/crates/remi) crate with the `s3` feature.
//!
//! ## Features
//! ### serde (disabled by default)
//! Enabels the use of **serde** for the [`S3StorageConfig`] struct.
//!
//! ## Usage
//! ```toml
//! [dependencies]
//! # Using the main `remi` crate. This is recommended for most cases, but not recommended
//! # if you want to enable some crate features.
//! remi = { version = "0.2", default-features = false, features = ["s3"] }
//!
//! # Using the crate itself, which you can enable any feature you wish. You will need to write
//! # the following examples as `remi_s3` instead of `remi::s3`.
//! remi-s3 = "0.2"
//! ```
//!
//! ```no_run
//! use remi::core::StorageService;
//! use remi::s3::S3StorageService;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let s3 = S3StorageService::from_aws_env();
//! s3.init().await?;
//! # }
//! ```

mod config;
mod service;

pub use config::{S3StorageConfig, S3StorageConfigBuilder, S3StorageConfigBuilderError};
pub use service::S3StorageService;
