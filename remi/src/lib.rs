// 🐻‍❄️🧶 remi-rs: Asynchronous Rust crate to handle communication between applications and object storage providers
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

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
//! **remi-rs** is a Rust edition of Noelware's Java library [remi](https://github.com/Noelware/remi) that was
//! discontinuted on **December 15th, 2023** and is the primary library that Noelware uses and maintained.
//!
//! **remi-rs** is a easy way to communicate with object storage providers like Microsoft Azure and Amazon S3.
//! It is an abstraction on common methods (like fetching, creating, listing, etc.) called a "storage service"
//! where it implements a set of methods that is commonly used in applications.
//!
//! **Warning** — All code in the repository is VERY EXPERIMENTAL and things can break at anytime &
//! be removed without any notice.
//!
//! ## Projects using `remi-rs`
//! - [📦 **charted-server**](https://github.com/charted-dev/charted)
//! - [🪶 **Hazel**](https://github.com/Noelware/hazel)
//! - [🐾 **ume**](https://github.com/auguwu/floofy.dev)
//!
//! ## Official Crates
//! - [**remi-gridfs**](https://crates.io/crates/remi-gridfs)
//! - [**remi-azure**](https://crates.io/crates/remi-azure)
//! - [**remi-s3**](https://crates.io/crates/remi-s3)
//! - [**remi-fs**](https://crates.io/crates/remi-fs)

use std::{borrow::Cow, path::Path};

// re-export (just in case!~)
#[doc(hidden)]
pub use async_trait::async_trait;

#[doc(hidden)]
pub use bytes::Bytes;

mod blob;
mod metadata;
mod options;

pub use blob::*;
pub use options::*;

/// A storage service is a base primitive of `remi-rs`: it is the way to interact
/// with the storage providers in ways that you would commonly use files: open, deleting,
/// listing, etc.
#[async_trait]
pub trait StorageService: Send + Sync {
    /// Represents a generic error to use for errors that could be emitted
    /// when calling any function.
    type Error;

    /// Returns the name of the storage service.
    ///
    /// * since 0.1.0
    fn name(&self) -> Cow<'static, str>
    where
        Self: Sized;

    /// Optionally initialize this [`StorageService`] if it requires initialization,
    /// like creating a directory if it doesn't exist.
    ///
    /// * since 0.1.0
    async fn init(&self) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        Ok(())
    }

    /// Opens a file in the specified `path` and returns the contents as [`Bytes`] if it existed, otherwise
    /// `None` will be returned to indicate that file doesn't exist.
    ///
    /// * since 0.1.0
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>, Self::Error>
    where
        Self: Sized;

    /// Open a file in the given `path` and returns a [`Blob`] structure if the path existed, otherwise
    /// `None` will be returned to indiciate that a file doesn't exist.
    ///
    /// * since 0.1.0
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>, Self::Error>
    where
        Self: Sized;

    /// Iterate over a list of files from a storage service and returns a [`Vec`] of [`Blob`]s.
    ///
    /// * since 0.1.0
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>, Self::Error>
    where
        Self: Sized;

    /// Deletes a file in a specified `path`. At the moment, `()` is returned but `bool` might be
    /// returned to indicate if it actually deleted itself or not.
    ///
    /// * since 0.1.0
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<(), Self::Error>
    where
        Self: Sized;

    /// Checks the existence of the file by the specified path.
    ///
    /// * since: 0.1.0
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool, Self::Error>
    where
        Self: Sized;

    /// Does a file upload where it writes the byte array as one call and does not do chunking.
    ///
    /// * since: 0.1.0
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> Result<(), Self::Error>
    where
        Self: Sized;

    #[cfg(feature = "unstable")]
    #[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "unstable")))]
    /// Performs any healthchecks to determine the storage service's health.
    async fn healthcheck(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::StorageService;

    const _DYN_STORAGE_SERVICE: Option<&dyn StorageService<Error = ()>> = None;
}
