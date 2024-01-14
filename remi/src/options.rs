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

use bytes::Bytes;
use std::collections::{HashMap, HashSet};

/// Represents the request options for querying blobs from a storage service.
#[derive(Debug, Clone, Default)]
pub struct ListBlobsRequest {
    /// Whether if the response should include directory blobs or not. If this set
    /// to false, then it will only include file blobs in the given directory
    /// where the request is being processed.
    pub include_dirs: bool,

    /// A list of extensions to filter for. By default, this will
    /// include all file extensions if no entries exist.
    pub extensions: HashSet<String>,

    /// List of file names to exclude from the returned entry. This can
    /// exclude directories with the `dir:` prefix.
    pub excluded: HashSet<String>,

    /// Optional prefix to set when querying for blobs.
    pub prefix: Option<String>,
}

impl ListBlobsRequest {
    /// Appends a slice of strings to exclude from.
    pub fn exclude<'a, I: Iterator<Item = &'a str>>(mut self, items: I) -> Self {
        self.excluded.extend(items.map(String::from));
        self
    }

    /// Sets a prefix to this request.
    pub fn with_prefix<I: Into<String>>(mut self, prefix: Option<I>) -> Self {
        self.prefix = prefix.map(Into::into);
        self
    }

    /// Appends a list of extensions that can be use to filter files from
    /// in the given directory that items were found.
    pub fn with_extensions<'a, I: Iterator<Item = &'a str>>(mut self, exts: I) -> Self {
        self.extensions
            .extend(exts.filter(|x| x.starts_with('.')).map(String::from));
        self
    }

    /// Whether if the response should include directory blobs or not. If this set
    /// to false, then it will only include file blobs in the given directory
    /// where the request is being processed.
    pub fn with_include_dirs(&mut self, yes: bool) -> &mut Self {
        self.include_dirs = yes;
        self
    }

    /// Checks if the given item is excluded or not.
    ///
    /// ## Example
    /// ```rust,ignore
    /// # use remi::ListBlobsRequest;
    /// #
    /// let mut req = ListBlobsRequest::default();
    /// let _ = req.clone().exclude(&["hello.txt"]);
    ///
    /// assert!(!req.is_excluded("world.txt"));
    /// assert!(req.is_excluded("hello.txt"));
    /// ```
    pub fn is_excluded<I: AsRef<str>>(&self, item: I) -> bool {
        self.excluded.contains(&item.as_ref().to_string())
    }

    /// Checks if an extension is allowed. If the configured extensions
    /// to return is empty, then this will always return `true`. Otherwise,
    /// it will try to check if it exists or not.
    ///
    /// ## Example
    /// ```rust,ignore
    /// # use remi::ListBlobsRequest;
    /// #
    /// let mut req = ListBlobsRequest::default();
    /// let _ = req.clone().extensions(&[".txt"]);
    ///
    /// assert!(!req.is_ext_allowed(".json"));
    /// assert!(req.is_ext_allowed(".txt"));
    ///
    /// let req = ListBlobsRequest::default();
    /// assert!(req.is_ext_allowed(".json"));
    /// ```
    pub fn is_ext_allowed<I: AsRef<str>>(&self, ext: I) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        self.extensions.contains(&ext.as_ref().to_string())
    }
}

/// Represents a request object that allows users who interact with the storage service
/// API to create objects with a [`Bytes`] container.
#[derive(Debug, Clone, Default)]
pub struct UploadRequest {
    /// Returns the content-type to use. By default, the storage service
    /// you use will try to determine it automatically if it can.
    pub content_type: Option<String>,

    /// Extra metadata to insert. Metadata can be queried when blobs
    /// are queried.
    ///
    /// - Filesystem: This will not do anything.
    /// - Azure: This will not do anything.
    /// - GCS: This will not do anything.
    /// - S3: This will insert it into the object's metadata
    pub metadata: HashMap<String, String>,

    /// [`Bytes`] container of the given data to send to the service
    /// or to write to local disk (with `remi_fs`).
    pub data: Bytes,
}

impl UploadRequest {
    /// Overrides the content type when the request is sent.
    ///
    /// ## Example
    /// ```
    /// # use remi::UploadRequest;
    /// #
    /// let mut req = UploadRequest::default();
    /// assert!(req.content_type.is_none());
    ///
    /// let _ = req.with_content_type(Some("application/json; charset=utf-8"));
    /// assert!(req.content_type.is_some());
    /// assert_eq!(req.content_type.unwrap().as_str(), "application/json; charset=utf-8");
    /// ```
    pub fn with_content_type<I: Into<String>>(mut self, content_type: Option<I>) -> Self {
        self.content_type = content_type.map(Into::into);
        self
    }

    /// Appends new metadata to this request.
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Overrides the data container for this request to a new container provided.
    ///
    /// ## Example
    /// ```
    /// # use remi_core::UploadRequest;
    /// # use bytes::Bytes;
    /// #
    /// let mut req = UploadRequest::default();
    /// assert!(req.data.is_empty());
    ///
    /// let _ = req.with_data(Bytes::from_static(&[0x12, 0x13]));
    /// assert!(!req.data.is_empty());
    /// ```
    pub fn with_data<I: Into<Bytes>>(mut self, container: I) -> Self {
        self.data = container.into();
        self
    }
}
