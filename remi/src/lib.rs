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

use std::path::Path;

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

    /// The name of the storage service.
    const NAME: &'static str;

    /// Optionally initialize this [`StorageService`] if it requires initialization,
    /// like creating a directory if it doesn't exist.
    ///
    /// * since 0.1.0
    async fn init(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Opens a file in the specified `path` and returns the contents as [`Bytes`] if it existed, otherwise
    /// `None` will be returned to indicate that file doesn't exist.
    ///
    /// * since 0.1.0
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Bytes>, Self::Error>;

    /// Open a file in the given `path` and returns a [`Blob`] structure if the path existed, otherwise
    /// `None` will be returned to indiciate that a file doesn't exist.
    ///
    /// * since 0.1.0
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> Result<Option<Blob>, Self::Error>;

    /// Iterate over a list of files from a storage service and returns a [`Vec`] of [`Blob`]s.
    ///
    /// * since 0.1.0
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>, Self::Error>;

    /// Deletes a file in a specified `path`. At the moment, `()` is returned but `bool` might be
    /// returned to indicate if it actually deleted itself or not.
    ///
    /// * since 0.1.0
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> Result<(), Self::Error>;

    /// Checks the existence of the file by the specified path.
    ///
    /// * since: 0.1.0
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> Result<bool, Self::Error>;

    /// Does a file upload where it writes the byte array as one call and does not do chunking. Use the [`StorageService::multipart_upload`]
    /// method to upload chunks by a specific size.
    ///
    /// * since: 0.1.0
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> Result<(), Self::Error>;

    // /// Does a multipart upload, where it uploads chunks of data bit by bit. By default, this will in an
    // /// unimplemented state and some storage services don't support chunk uploading.
    // ///
    // /// * since
    // async fn multipart_upload<P: AsRef<Path> + Send>(&self, _path: P) -> Result<(), Self::Error> {
    //     unimplemented!()
    // }

    /// Attempt to find a blob from a [`Blob`] where it returns the first blob that was found. A default
    /// implementation is given which just queries all blobs via [`StorageService::blobs`] and uses the
    /// [`find`][Iterator::find] method.
    ///
    /// * since: 0.6.0
    async fn find<P: AsRef<Path> + Send, F: FnMut(&Blob) -> bool + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
        finder: F,
    ) -> Result<Option<Blob>, Self::Error> {
        self.blobs(path, options)
            .await
            .map(|blobs| blobs.into_iter().find(finder))
    }
}
