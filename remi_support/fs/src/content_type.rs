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

use std::fmt::Debug;

/// Represents a resolver to detect content types from an unknown
/// size of bytes.
///
/// If the `file-format` crate feature is enabled, then the library
/// will do on both image-based and file-based formats, if it
/// is not enabled, then none of it is enabled and you will always
/// get `application/octet-stream`.
pub trait ContentTypeResolver: Send + Sync {
    /// Resolves an unknown size of bytes and returns the content
    /// type that is used.
    fn resolve(&self, bytes: &[u8]) -> String;
}

impl Debug for dyn ContentTypeResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dyn ContentTypeResolver")
            .finish_non_exhaustive()
    }
}

/// A default implementation of a [`ContentTypeResolver`] that uses
/// the [infer]() and [file-format]() crates if the `file-format`
/// crate feature is enabled.
#[derive(Debug, Clone, Copy)]
pub struct DefaultContentTypeResolver;

impl ContentTypeResolver for DefaultContentTypeResolver {
    #[cfg(not(feature = "file-format"))]
    fn resolve(&self, _bytes: &[u8]) -> String {
        "application/octet-stream".into()
    }

    #[cfg(feature = "file-format")]
    fn resolve(&self, bytes: &[u8]) -> String {
        infer::get(bytes)
            .map(|ty| ty.mime_type().to_owned())
            .unwrap_or(
                file_format::FileFormat::from_bytes(bytes)
                    .media_type()
                    .to_owned(),
            )
    }
}
