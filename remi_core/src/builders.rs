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

use bytes::Bytes;
use derive_builder::Builder;

/// Represents the request options for the `blobs` method in [`StorageService`]
#[derive(Debug, Default, Clone, Builder)]
pub struct ListBlobsRequest {
    extensions: Vec<String>,
    excluded: Vec<String>,
    prefix: Option<String>,
}

impl ListBlobsRequest {
    /// Creates a new [`ListBlobsRequestBuilder`].
    pub fn builder() -> ListBlobsRequestBuilder {
        ListBlobsRequestBuilder::default()
    }

    pub fn excluded(&self) -> Vec<String> {
        self.excluded.clone()
    }

    pub fn extensions_allowed(&self) -> Vec<String> {
        self.extensions.clone()
    }

    pub fn prefix(&self) -> Option<String> {
        self.prefix.clone()
    }

    /// Checks if the given string is excluded or not.
    pub fn is_excluded(&self, p: String) -> bool {
        if self.excluded().is_empty() {
            return false;
        }

        self.excluded.contains(&p)
    }

    /// Checks if the `ext` is allowed or not.
    pub fn is_ext_allowed(&self, ext: &str) -> bool {
        if self.extensions_allowed().is_empty() {
            return false;
        }

        self.extensions.contains(&ext.to_string())
    }
}

/// Represents a request object for the [`StorageService::upload`] method. It contains the
/// content type and the inner data as `content` itself.
#[derive(Debug, Builder)]
pub struct UploadRequest {
    content_type: String,
    content: Bytes,
}

impl UploadRequest {
    /// Creates a new [`UploadRequestBuilder`].
    pub fn builder() -> UploadRequestBuilder {
        UploadRequestBuilder::default()
    }

    /// Returns the content type of this file
    pub fn content_type(&self) -> String {
        self.content_type.clone()
    }

    /// Returns the inner [`Bytes`] for this file.
    pub fn data(&self) -> Bytes {
        self.content.clone()
    }
}
