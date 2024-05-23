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

use std::borrow::Cow;

/// Default content type given from a [`ContentTypeResolver`]
pub const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

/// Represents a resolver to resolve content types from a byte slice.
pub trait ContentTypeResolver: Send + Sync {
    /// Resolves a byte slice and returns the content type, or [`DEFAULT_CONTENT_TYPE`]
    /// if none can be resolved from this resolver.
    fn resolve(&self, data: &[u8]) -> Cow<'static, str>;
}

impl<F> ContentTypeResolver for F
where
    F: Fn(&[u8]) -> Cow<'static, str> + Send + Sync,
{
    fn resolve(&self, data: &[u8]) -> Cow<'static, str> {
        (self)(data)
    }
}

#[cfg(feature = "file-format")]
pub fn default_resolver(data: &[u8]) -> Cow<'static, str> {
    #[cfg(feature = "serde_json")]
    if serde_json::from_slice::<serde_json::Value>(data).is_ok() {
        return Cow::Borrowed("application/json; charset=utf-8");
    }

    #[cfg(feature = "serde_yaml")]
    if serde_yaml::from_slice::<serde_yaml::Value>(data).is_ok() {
        return Cow::Borrowed("text/yaml; charset=utf-8");
    }

    infer::get(data).map(|ty| Cow::Borrowed(ty.mime_type())).unwrap_or({
        let format = file_format::FileFormat::from_bytes(data);
        Cow::Owned(format.media_type().to_owned())
    })
}

/// A default implementation of a [`ContentTypeResolver`].
#[cfg(not(feature = "file-format"))]
pub fn default_resolver(data: &[u8]) -> Cow<'static, str> {
    #[cfg(feature = "serde_json")]
    if serde_json::from_slice::<serde_json::Value>(data).is_ok() {
        return Cow::Borrowed("application/json; charset=utf-8");
    }

    #[cfg(feature = "serde_yaml")]
    if serde_yaml::from_slice::<serde_yaml::Value>(data).is_ok() {
        return Cow::Borrowed("text/yaml; charset=utf-8");
    }

    DEFAULT_CONTENT_TYPE.into()
}

#[cfg(test)]
mod tests {
    use super::default_resolver;

    #[cfg_attr(feature = "file_format", test)]
    #[cfg_attr(
        not(feature = "file_format"),
        ignore = "requires `file_format` feature to actually be correct"
    )]
    #[cfg_attr(not(feature = "file_format"), allow(unused))]
    fn test_other_stuff() {
        assert_eq!("text/plain", default_resolver(b"some plain text"));
    }

    #[cfg_attr(feature = "serde_json", test)]
    #[cfg_attr(not(feature = "serde_json"), ignore = "requires `serde_json` feature")]
    #[cfg_attr(not(feature = "serde_json"), allow(unused))]
    fn test_json() {}

    #[cfg_attr(feature = "serde_yaml", test)]
    #[cfg_attr(not(feature = "serde_yaml"), ignore = "requires `serde_yaml` feature")]
    #[cfg_attr(not(feature = "serde_yaml"), allow(unused))]
    fn test_yaml() {}
}
