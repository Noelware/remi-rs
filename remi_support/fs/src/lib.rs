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

//! # 🐻‍❄️🧶 remi_fs
//! **remi_fs** is a crate implementation of [StorageService] that interacts with the local filesystem. This crate is also re-exported
//! in the [remi](https://docs.rs/remi) crate with the `fs` crate feature.
//!
//! **remi_fs** supports using async-std and Tokio as the async runtime, which will delegate using `async_std::fs` or
//! `tokio::fs` modules.
//!
//! ## Crate Features
//! ### serde [disabled by default]
//! Enables the use of **serde** for the [`FilesystemStorageConfig`] struct, which will allow you to configure
//! the [`FilesystemStorageService`] struct from anything that can be deserialized from.
//!
//! ### async_std [disabled by default]
//! Enables using **async_std** for filesystem operations, instead of **Tokio**.
//!
//! ## Usage
//! This example will use Tokio, but uses the `remi` crate since the filesystem is the default
//! feature for it.
//!
//! ```toml
//! [dependencies]
//! bytes = "1.4"
//! remi = { version = "0.2" }
//! tokio = { version = "1.28", features = ["fs", "io_util"] }
//! ```
//!
//! ```no_run
//! use remi::filesystem::FilesystemStorageService;
//! use remi::builders::UploadRequest;
//! use bytes::Bytes;
//!
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    let fs = FilesystemStorageService::new("./.data");
//!
//!    // init() will create the directory that was passed above
//!    // if it doesn't exist on the disk
//!    fs.init().await?;
//!
//!    // now, let's open a file that doesn't exist
//!    let file = fs.open("./owo.txt").await?;
//!    // "file" will be None since the file doesn't exist on the disk
//!
//!    // let's create the file
//!    fs.upload("./owo.txt", UploadRequest::builder()
//!        .content(Bytes::new())
//!        .content_type("text/plain")
//!        .build()?
//!    ).await?;
//!
//!    // now, let's read the file again
//!    let file = fs.open("./owo.txt").await?;
//!    // "file" is now Some(Blob::File).
//! }
//! ```

mod config;
mod service;

pub use crate::config::FilesystemStorageConfig;
pub use crate::service::FilesystemStorageService;
