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

use derive_builder::Builder;
use mongodb::options::{ReadConcern, ReadPreference, WriteConcern};
use mongodb_gridfs::options::GridFSBucketOptions;

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridfsStorageConfig {
    bucket_name: String,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    chunk_size_bytes: Option<u32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    write_concern: Option<WriteConcern>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    read_concern: Option<ReadConcern>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    read_preference: Option<ReadPreference>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    disable_md5: Option<bool>,
}

impl GridfsStorageConfig {
    /// Creates a builder for this configuration object
    pub fn builder() -> GridfsStorageConfigBuilder {
        GridfsStorageConfigBuilder::default()
    }

    /// Turns this [`GridfsStorageConfig`] object into a [`GridFSBucketOptions`] object.
    pub fn to_gridfs_options(&self) -> GridFSBucketOptions {
        GridFSBucketOptions::builder()
            .bucket_name(self.bucket_name.clone())
            .chunk_size_bytes(self.chunk_size_bytes.unwrap_or(255 * 1024))
            .write_concern(self.write_concern.clone())
            .read_concern(self.read_concern.clone())
            .read_preference(self.read_preference.clone())
            .disable_md5(self.disable_md5.unwrap_or(false))
            .build()
    }

    pub fn bucket_name(&self) -> String {
        self.bucket_name.clone()
    }

    pub fn chunk_size_bytes(&self) -> u32 {
        self.chunk_size_bytes.unwrap_or(255 * 1024)
    }

    pub fn write_concern(&self) -> Option<WriteConcern> {
        self.write_concern.clone()
    }

    pub fn read_concern(&self) -> Option<ReadConcern> {
        self.read_concern.clone()
    }

    pub fn read_preference(&self) -> Option<ReadPreference> {
        self.read_preference.clone()
    }

    pub fn disable_md5(&self) -> bool {
        self.disable_md5.unwrap_or(false)
    }
}
