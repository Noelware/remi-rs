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

/// Default content type given from a [`ContentTypeResolver`]
pub const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

/// Represents a resolver to resolve content types from a byte slice.
pub trait ContentTypeResolver: Send + Sync {
    /// Resolves a byte slice and returns the content type, or [`DEFAULT_CONTENT_TYPE`]
    /// if none can be resolved from this resolver.
    fn resolve(&self, data: &[u8]) -> String;
}

impl<F> ContentTypeResolver for F
where
    F: Fn(&[u8]) -> String + Send + Sync,
{
    fn resolve(&self, data: &[u8]) -> String {
        (self)(data)
    }
}

/// A default implementation of a [`ContentTypeResolver`] that uses
/// the [infer](https://docs.rs/infer) and [file-format](https://docs.rs/file-format) crates if the `file-format`
/// crate feature is enabled.
///
/// This doesn't include checking for JSON or YAML formats (if the respected crate features are enabled),
/// please use the new [`default_resolver`] function to do so.
#[derive(Debug, Clone, Copy)]
#[deprecated(
    since = "0.5.0",
    note = "please use the new `default_resolver` fn to implement `ContentTypeResolver`"
)]
pub struct DefaultContentTypeResolver;

#[allow(deprecated)]
impl ContentTypeResolver for DefaultContentTypeResolver {
    #[cfg(not(feature = "file-format"))]
    fn resolve(&self, _bytes: &[u8]) -> String {
        "application/octet-stream".into()
    }

    #[cfg(feature = "file-format")]
    fn resolve(&self, bytes: &[u8]) -> String {
        infer::get(bytes)
            .map(|ty| ty.mime_type().to_owned())
            .unwrap_or(file_format::FileFormat::from_bytes(bytes).media_type().to_owned())
    }
}

/// A default implementation of a [`ContentTypeResolver`].
#[cfg(feature = "file-format")]
pub fn default_resolver(data: &[u8]) -> String {
    #[cfg(feature = "serde_json")]
    match serde_json::from_slice::<()>(data) {
        Ok(_) => return String::from("application/json; charset=utf-8"),
        Err(_) => {}
    }

    #[cfg(feature = "serde_yaml")]
    match serde_yaml::from_slice::<()>(data) {
        Ok(_) => return String::from("application/yaml; charset=utf-8"),
        Err(_) => {}
    }

    infer::get(data)
        .map(|ty| ty.mime_type().to_owned())
        .unwrap_or(file_format::FileFormat::from_bytes(data).media_type().to_owned())
}

/// A default implementation of a [`ContentTypeResolver`].
#[cfg(not(feature = "file-format"))]
pub fn default_resolver(data: &[u8]) -> String {
    #[cfg(feature = "serde_json")]
    match serde_json::from_slice(data) {
        Ok(_) => String::from("application/json; charset=utf-8"),
        Err(_) => {}
    }

    #[cfg(feature = "serde_yaml")]
    match serde_yaml::from_slice(data) {
        Ok(_) => String::from("application/yaml; charset=utf-8"),
        Err(_) => {}
    }

    DEFAULT_CONTENT_TYPE.into()
}
