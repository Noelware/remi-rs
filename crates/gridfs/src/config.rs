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

use mongodb::options::{ClientOptions, GridFsBucketOptions, ReadConcern, SelectionCriteria, WriteConcern};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageConfig {
    /// Specifies the [`SelectionCriteria`].
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            serialize_with = "serialize_selection_criteria",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub selection_criteria: Option<SelectionCriteria>,

    /// Specifies the [`WriteConcern`] for all level acknowledgment when writing
    /// new documents into the GridFS datastore. Read the [`MongoDB` documentation](https://www.mongodb.com/docs/manual/reference/write-concern)
    /// for more information.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub write_concern: Option<WriteConcern>,

    /// Configure the [`ClientOptions`] that allows to connect to a MongoDB server.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing))]
    pub client_options: ClientOptions,

    /// Specifies the [`ReadConcern`] for isolation for when reading documents from the GridFS store. Read the
    /// [`MongoDB` documentation](https://www.mongodb.com/docs/manual/reference/write-concern) for more information.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub read_concern: Option<ReadConcern>,

    /// Chunk size (in bytes) used to break the user file into chunks.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub chunk_size: Option<u32>,

    /// Database to connect to if [`client_options`][StorageConfig::client_options] was set. It will default
    /// to the default database.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub database: Option<String>,

    /// Bucket name that holds all the GridFS datastore blobs.
    pub bucket: String,
}

impl From<StorageConfig> for GridFsBucketOptions {
    fn from(value: StorageConfig) -> Self {
        GridFsBucketOptions::builder()
            .selection_criteria(value.selection_criteria)
            .read_concern(value.read_concern)
            .write_concern(value.write_concern)
            .chunk_size_bytes(value.chunk_size)
            .bucket_name(value.bucket)
            .build()
    }
}

#[cfg(feature = "serde")]
#[allow(unused)]
fn serialize_selection_criteria<S: ::serde::ser::Serializer>(
    value: &Option<SelectionCriteria>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use ::mongodb::options::ReadPreference;
    use ::serde::{ser::Error, Serialize};

    if let Some(value) = value {
        if matches!(value, SelectionCriteria::Predicate(_)) {
            return Err(S::Error::custom(
                "cannot use `SelectionCriteria::Predicate` to be serialized",
            ));
        }

        match value {
            SelectionCriteria::ReadPreference(rp) => return ReadPreference::serialize(rp, serializer),
            _ => unimplemented!(),
        }
    }

    serializer.serialize_none()
}
