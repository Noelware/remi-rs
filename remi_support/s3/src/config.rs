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

use aws_sdk_s3::{
    model::{BucketCannedAcl, ObjectCannedAcl},
    Region,
};

use derive_builder::Builder;

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct S3StorageConfig {
    enable_signer_v4_requests: bool,
    enforce_path_access_style: bool,
    default_object_acl: Option<ObjectCannedAcl>,
    default_bucket_acl: Option<BucketCannedAcl>,
    secret_access_key: String,
    access_key_id: String,
    endpoint: Option<String>,
    prefix: Option<String>,
    region: Option<Region>,
    bucket: String,
}

impl S3StorageConfig {
    /// Returns a [builder][S3StorageConfigBuilder] object
    pub fn builder() -> S3StorageConfigBuilder {
        S3StorageConfigBuilder::default()
    }

    /// Checks if using AWS Signer v4 requests should be used instead of the default
    pub fn enable_signer_v4_requests(&self) -> bool {
        self.enable_signer_v4_requests
    }

    /// Checks if the AWS endpoint should enforce the path style access (`<endpoint>/<bucket>`)
    /// rather than `<bucket>.<endpoint>`. This is recommended to be set in MinIO installations that
    /// use this crate.
    pub fn enforce_path_access_style(&self) -> bool {
        self.enforce_path_access_style
    }

    /// Returns the default [`ObjectCannedACL`] for all blobs that are uploaded with
    /// the `upload` function. If this was a `Option::None` variant, it will use the
    /// `public-read` ACL.
    pub fn default_object_acl(&self) -> ObjectCannedAcl {
        match self.default_object_acl.clone() {
            Some(o) => o,
            None => ObjectCannedAcl::PublicRead,
        }
    }

    /// Returns the default [`BucketCannedACL`] for the bucket if the bucket doesn't already exists
    /// in S3. If this option was a `Option::None` variant, it will use the `public-read` ACL permission.
    pub fn default_bucket_acl(&self) -> BucketCannedAcl {
        match self.default_bucket_acl.clone() {
            Some(b) => b,
            None => BucketCannedAcl::PublicRead,
        }
    }

    /// Returns the secret access key when authenticating with AWS S3.
    pub fn secret_access_key(&self) -> String {
        self.secret_access_key.clone()
    }

    /// Returns the access key ID when authenticating with AWS S3.
    pub fn access_key_id(&self) -> String {
        self.access_key_id.clone()
    }

    /// Returns the S3 endpoint to connect to. If this is a `Option::None` variant, it will default
    /// to `s3.amazonaws.com`.
    ///
    /// If you're using Wasabi, you will need to set the endpoint to `s3.wasabisys.com`. If you're connecting to
    /// your MinIO server, just use the endpoint that is used with the S3 API.
    pub fn endpoint(&self) -> String {
        match self.endpoint.clone() {
            Some(endpoint) => endpoint,
            None => "https://s3.amazonaws.com".to_owned(),
        }
    }

    /// Returns the prefix to use when interacting with blobs. This is most useful to filter out
    /// any blobs that you might not be using. The prefix will be resolved when using any method
    /// of the storage service.
    ///
    /// ## Examples
    /// - `prefix: Some("awau/owo".into())` -> `s3://<bucket>/awau/owo/<path>`
    /// - `prefix: None` -> `s3://<bucket>/<path>`
    pub fn prefix(&self) -> Option<String> {
        self.prefix.clone()
    }

    /// Returns the region to connect to when connecting to S3. By default, it will use the `us-east-1` region.
    pub fn region(&self) -> Region {
        match self.region.clone() {
            Some(region) => region,
            None => Region::from_static("us-east-1"),
        }
    }

    /// Returns the bucket to operate on.
    pub fn bucket(&self) -> String {
        self.bucket.clone()
    }
}
