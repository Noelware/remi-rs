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

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc = include_str!("../README.md")]

use std::{io, path::Path};

// re-export (just in case!~)
pub use async_trait::async_trait;
pub use bytes::Bytes;

mod blob;
mod options;

pub use blob::*;
pub use options::*;

/// A storage service is a base primitive of `remi-rs`: it is the way to interact
/// with the storage providers in ways that you would commonly use files: open, deleting,
/// listing, etc.
#[async_trait]
pub trait StorageService: Send + Sync {
    /// The name of the storage service.
    const NAME: &'static str;

    /// Returns the name of this [`StorageService`].
    #[deprecated(since = "0.5.0", note = "use Self::NAME instead of the name() function")]
    fn name(&self) -> &'static str {
        Self::NAME
    }

    /// Optionally initialize this [`StorageService`] if it requires initialization,
    /// like creating a directory if it doesn't exist.
    async fn init(&self) -> io::Result<()> {
        Ok(())
    }

    /// Opens a file in a given `path` and returns a Option variant of a given [`Bytes`] container.
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Bytes>>;

    /// Returns a [`Blob`] instance of the given file or directory, if it exists.
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Blob>>;

    /// Similar to [`blob`](StorageService::blob) but returns a list of blobs that exist
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> io::Result<Vec<Blob>>;

    /// Deletes a path.
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<()>;

    /// Checks whether or not if a path exists.
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<bool>;

    /// Uploads a path.
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> io::Result<()>;
}
