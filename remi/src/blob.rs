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
use std::fmt::Display;

/// Represents a file or directory from any storage service.
pub enum Blob {
    /// Represents a directory that was located somewhere.
    Directory(Directory),

    /// Representation of a [`Blob`] that is a file.
    File(File),
}

/// Representation of a [`Blob`] that is a file.
#[deprecated(
    since = "0.5.0",
    note = "`FileBlob` was renamed to `File`, this will be removed in v0.7.0"
)]
pub type FileBlob = File;

/// Representation of a [`Blob`] that is a file.
#[derive(Debug, Clone)]
pub struct File {
    /// Returns a `u128` of when this file was last modified, in milliseconds
    /// from January 1st, 1970.
    pub last_modified_at: Option<u128>,

    /// Returns the `Content-Type` header of this file, which should represent
    /// what type of file this is.
    pub content_type: Option<String>,

    /// Returns a `u128` of when this file was last created, in milliseconds
    /// from January 1st, 1970.
    pub created_at: Option<u128>,

    /// Whether or not if this file was a symlink or not. This is only used
    /// in the filesystem crate of remi.
    pub is_symlink: bool,

    /// Given [`Bytes`] container that is the actual data in the file.
    pub data: Bytes,

    /// File name
    pub name: String,

    /// File path, usually `{service}://{full filepath}`
    pub path: String,

    /// file length (in bytes)
    pub size: usize,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // file "assets/openapi.json" (12345 bytes) | application/json; charset=utf-8
        f.write_fmt(format_args!(
            "file [{}] ({} bytes){}",
            self.path,
            self.size,
            match self.content_type {
                Some(ref ct) => format!(" | {ct}"),
                None => "".into(),
            }
        ))
    }
}

/// Representation of a [`Blob`] that is a directory.
#[deprecated(
    since = "0.5.0",
    note = "`DirectoryBlob` was renamed to `Directory`, this will be removed in v0.7.0"
)]
pub type DirectoryBlob = Directory;

/// Represents a directory that was located somewhere.
#[derive(Debug, Clone)]
pub struct Directory {
    /// Returns a `u128` of when this directory was last created, in milliseconds
    /// from January 1st, 1970.
    pub created_at: Option<u128>,

    /// Directory name
    pub name: String,

    /// Directory path, usually `{service}://{full filepath}`
    pub path: String,
}

impl Display for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("directory {}", self.path))
    }
}
