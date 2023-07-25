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

const SYMLINK_BIT: usize = 1 << 1;

/// Represents a representation of a file or directory from any storage service.
#[derive(Debug, Clone)]
pub enum Blob {
    /// File.
    File(FileBlob),

    /// Directory.
    Directory(DirectoryBlob),
}

/// Representation of a [`Blob`] that is a regular file.
#[derive(Debug, Clone)]
pub struct FileBlob {
    last_modified_at: Option<u128>,
    content_type: Option<String>,
    created_at: Option<u128>,
    flags: usize,
    data: Bytes,
    name: String,
    path: String,
    size: usize,
}

/// Represents a directory that was located
#[derive(Debug, Clone)]
pub struct DirectoryBlob {
    created_at: Option<u128>,
    name: String,
    path: String,
}

impl FileBlob {
    /// Create a new [`FileBlob`].
    ///
    /// ## Arguments
    /// - `last_modified_at`: Option variant of when this file was last modified at
    /// - `content_type`: Option variant of the content type of this file.
    /// - `created_at`: Option variant of when this file was last created at
    /// - `symlink`: If the file was a symlink, or not. At the moment, it cannot detect the refernence if this is `true`.
    /// - `service`: The service that created this [`FileBlob`].
    /// - `data`: [Bytes] structure of the data itself.
    /// - `name`: File name
    /// - `size`: Size of this file.
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub fn new(
        last_modified_at: Option<u128>,
        content_type: Option<String>,
        created_at: Option<u128>,
        symlink: bool,
        service: String,
        data: Bytes,
        name: String,
        size: usize,
    ) -> FileBlob {
        let mut flags = 0;
        if symlink {
            flags = SYMLINK_BIT;
        }

        FileBlob {
            last_modified_at,
            content_type,
            created_at,
            data,
            flags,
            name: name.clone(),
            size,
            path: format!("{service}://{name}"),
        }
    }

    /// Checks if this [`FileBlob`] is a symbolic link or not. If the filesystem service determines that
    /// a file is a symbolic link, it will not be de-referenced.
    pub fn is_symlink(&self) -> bool {
        (self.flags & SYMLINK_BIT) != 0
    }

    /// Returns the last modified data of this file. This can be a `Option::None` variant if this
    /// blob is a directory or the storage backend doesn't keep track of the last modification state.
    pub fn last_modified_at(&self) -> Option<u128> {
        self.last_modified_at
    }

    /// Returns the content type of this file that can be applicable with the `Content-Type` HTTP header. This
    /// can be a `Option::None` variant if this blob is a directory.
    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    /// Returns the file creation date for this file. This can be a `Option::None` variant if this
    /// blob is a directory or the storage backend doesn't keep track of when this file was created.
    pub fn created_at(&self) -> Option<u128> {
        self.created_at
    }

    /// Returns a reference of the [bytes][Bytes] of the file data.
    pub fn data(&self) -> &Bytes {
        &self.data
    }

    /// Returns a reference to this file's name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// The size of this [`FileBlob`]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns a reference to the actual file path of this file.
    pub fn path(&self) -> &String {
        &self.path
    }
}

impl DirectoryBlob {
    #[allow(dead_code)]
    pub fn new(created_at: Option<u128>, service: String, name: String) -> DirectoryBlob {
        DirectoryBlob {
            created_at,
            name: name.clone(),
            path: format!("{service}://{name}"),
        }
    }

    /// Returns the creation date for this directory. This can be a `Option::None` variant if this
    /// blob is a directory or the storage backend doesn't keep track of the last modification state.
    pub fn created_at(&self) -> Option<u128> {
        self.created_at
    }

    /// Returns this [`DirectoryBlob`] name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the full path for this [`DirectoryBlob`].
    pub fn path(&self) -> String {
        self.path.clone()
    }
}
