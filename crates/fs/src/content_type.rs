// 🐻‍❄️🧶 remi-rs: Asynchronous Rust crate to handle communication between applications and object storage providers
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
        // representing "true", "false", "null", "{any string}", "{any number}" should be plain text
        match serde_json::from_slice(data).unwrap() {
            serde_json::Value::String(_)
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Null => return Cow::Borrowed("text/plain"),

            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                return Cow::Borrowed("application/json; charset=utf-8")
            }
        }
    }

    #[cfg(feature = "serde_yaml")]
    if serde_yaml::from_slice::<serde_yaml::Value>(data).is_ok() {
        fn match_value(value: &serde_yaml::Value) -> Cow<'static, str> {
            match value {
                serde_yaml::Value::Bool(_)
                | serde_yaml::Value::Number(_)
                | serde_yaml::Value::String(_)
                | serde_yaml::Value::Null => Cow::Borrowed("text/plain"),

                serde_yaml::Value::Tagged(m) => match_value(&m.value),
                serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => {
                    Cow::Borrowed("text/yaml; charset=utf-8")
                }
            }
        }

        return match_value(&serde_yaml::from_slice(data).unwrap());
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
        // representing "true", "false", "null", "{any string}", "{any number}" should be plain text
        match serde_json::from_slice(data).unwrap() {
            serde_json::Value::String(_)
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Null => return Cow::Borrowed("text/plain"),

            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                return Cow::Borrowed("application/json; charset=utf-8")
            }
        }
    }

    #[cfg(feature = "serde_yaml")]
    if serde_yaml::from_slice::<serde_yaml::Value>(data).is_ok() {
        fn match_value(value: &serde_yaml::Value) -> Cow<'static, str> {
            match value {
                serde_yaml::Value::Bool(_)
                | serde_yaml::Value::Number(_)
                | serde_yaml::Value::String(_)
                | serde_yaml::Value::Null => Cow::Borrowed("text/plain"),

                serde_yaml::Value::Tagged(m) => match_value(&m.value),
                serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => {
                    Cow::Borrowed("text/yaml; charset=utf-8")
                }
            }
        }

        return match_value(&serde_yaml::from_slice(data).unwrap());
    }

    DEFAULT_CONTENT_TYPE.into()
}

#[cfg(test)]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod tests {
    use super::default_resolver;

    #[cfg(feature = "file-format")]
    #[test]
    fn test_other_stuff() {
        assert_eq!("text/plain", default_resolver(b"some plain text"));
        assert_eq!("image/jpeg", default_resolver(&[0xFF, 0xD8, 0xFF, 0xAA]));
    }

    #[cfg(feature = "serde_json")]
    #[test]
    fn test_json() {
        use serde_json::{json, to_vec};

        for (value, assertion) in [
            (json!(null), "text/plain"),
            (json!(true), "text/plain"),
            (json!(false), "text/plain"),
            (json!("any string"), "text/plain"),
            (json!(1.2), "text/plain"),
            (json!({ "hello": "world" }), "application/json; charset=utf-8"),
            (json!(["hello", "world"]), "application/json; charset=utf-8"),
        ] {
            assert_eq!(
                assertion,
                default_resolver(&to_vec(&value).expect("failed to convert to JSON"))
            );
        }
    }

    #[cfg(feature = "serde_yaml")]
    #[test]
    fn test_yaml() {
        for (value, assertion) in [
            (serde_yaml::Value::Null, "text/plain"),
            (serde_yaml::Value::Bool(true), "text/plain"),
            (serde_yaml::Value::Bool(false), "text/plain"),
            (serde_yaml::Value::String("hello world".into()), "text/plain"),
            (serde_yaml::Value::Number(1.into()), "text/plain"),
            (
                serde_yaml::Value::Sequence(vec![serde_yaml::Value::Bool(true)]),
                "text/yaml; charset=utf-8",
            ),
            (
                serde_yaml::Value::Mapping({
                    let mut map = serde_yaml::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("hello".into()),
                        serde_yaml::Value::String("world".into()),
                    );

                    map
                }),
                "text/yaml; charset=utf-8",
            ),
        ] {
            assert_eq!(
                assertion,
                default_resolver(serde_yaml::to_string(&value).expect("failed to parse YAML").as_bytes())
            );
        }
    }
}
