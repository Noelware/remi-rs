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

/// Represents the request options for querying blobs from a storage
/// service.
#[derive(Debug, Clone, Default)]
pub struct ListBlobsRequest {
    /// Whether if the response should include directory blobs or not. If this set
    /// to false, then it will only include file blobs in the given directory
    /// where the request is being processed.
    pub include_dirs: bool,

    /// A list of extensions to filter for. By default, this will
    /// include all file extensions if no entries exist.
    pub extensions: Vec<String>,

    /// List of file names to exclude from the returned entry. This can
    /// exclude directories with the `dir:` prefix.
    pub excluded: Vec<String>,

    /// Optional prefix to set when querying for blobs.
    pub prefix: Option<String>,
}

impl ListBlobsRequest {
    /// Appends a slice of strings to exclude from.
    pub fn exclude(&mut self, items: &[&str]) -> &mut Self {
        self.excluded
            .append(&mut items.iter().map(|val| val.to_string()).collect::<Vec<_>>());

        self
    }

    /// Sets a prefix to this request.
    pub fn prefix(&mut self, prefix: Option<String>) -> &mut Self {
        self.prefix = prefix;
        self
    }

    /// Appends a list of extensions that can be use to filter files from
    /// in the given directory that items were found.
    pub fn extensions(&mut self, exts: &[&str]) -> &mut Self {
        self.extensions.append(
            &mut exts
                .iter()
                .filter(|val| val.starts_with('.'))
                .map(|val| val.to_string())
                .collect::<Vec<_>>(),
        );

        self
    }

    /// Whether if the response should include directory blobs or not. If this set
    /// to false, then it will only include file blobs in the given directory
    /// where the request is being processed.
    pub fn include_dirs(&mut self, yes: bool) -> &mut Self {
        self.include_dirs = yes;
        self
    }

    /// Checks if the given item is excluded or not.
    ///
    /// ## Example
    /// ```
    /// # use remi_core::ListBlobsRequest;
    /// #
    /// let mut req = ListBlobsRequest::default();
    /// let _ = req.exclude(&["hello.txt"]);
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
    /// ```
    /// # use remi_core::ListBlobsRequest;
    /// #
    /// let mut req = ListBlobsRequest::default();
    /// let _ = req.extensions(&[".txt"]);
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

    /// [`Bytes`] container of the given data to send to the service
    /// or to write to local disk (with `remi_fs`).
    pub data: Bytes,
}

impl UploadRequest {
    /// Overrides the content type when the request is sent.
    ///
    /// ## Example
    /// ```
    /// # use remi_core::UploadRequest;
    /// #
    /// let mut req = UploadRequest::default();
    /// assert!(req.content_type.is_none());
    ///
    /// let _ = req.content_type(Some("application/json; charset=utf-8".into()));
    /// assert!(req.content_type.is_some());
    /// assert_eq!(req.content_type.unwrap().as_str(), "application/json; charset=utf-8");
    /// ```
    pub fn content_type(&mut self, content_type: Option<String>) -> &mut Self {
        self.content_type = content_type;
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
    /// let _ = req.data(Bytes::from_static(&[0x12, 0x13]));
    /// assert!(!req.data.is_empty());
    /// ```
    pub fn data(&mut self, container: Bytes) -> &mut Self {
        self.data = container;
        self
    }
}
