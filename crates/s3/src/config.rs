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

use aws_config::AppName;
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::{
    config::Region,
    types::{BucketCannedAcl, ObjectCannedAcl},
};

/// Represents the main configuration struct to configure a [`S3StorageService`][crate::S3StorageService].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct S3StorageConfig {
    /// Whether if the S3 storage backend should enable AWSv4 signatures when requests
    /// come in or not.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enable_signer_v4_requests: bool,

    /// Whether if path access style should be enabled or not. This is recommended
    /// to be set to `true` on MinIO instances.
    ///
    /// - Enabled: `https://{host}/{bucket}/...`
    /// - Disabled: `https://{bucket}.{host}/...`
    #[cfg_attr(feature = "serde", serde(default))]
    pub enforce_path_access_style: bool,

    /// Default ACL for all new objects.
    #[cfg_attr(
        feature = "serde",
        serde(with = "__serde::object_acl", skip_serializing_if = "Option::is_none")
    )]
    pub default_object_acl: Option<ObjectCannedAcl>,

    /// Default ACL to use when a bucket doesn't exist and #init was called
    /// from the backend.
    #[cfg_attr(
        feature = "serde",
        serde(with = "__serde::bucket_acl", skip_serializing_if = "Option::is_none")
    )]
    pub default_bucket_acl: Option<BucketCannedAcl>,

    /// The secret access key to authenticate with S3
    pub secret_access_key: String,

    /// The access key ID to authenticate with S3
    pub access_key_id: String,

    /// Application name. This is set to `remi-s3` if not provided.
    #[cfg_attr(feature = "serde", serde(default))]
    pub app_name: Option<String>,

    /// AWS endpoint to reach.
    #[cfg_attr(feature = "serde", serde(default))]
    pub endpoint: Option<String>,

    /// Prefix for querying and inserting new blobs into S3.
    #[cfg_attr(feature = "serde", serde(default))]
    pub prefix: Option<String>,

    /// The region to use, this will default to `us-east-1`.
    #[cfg_attr(
        feature = "serde",
        serde(with = "__serde::region", skip_serializing_if = "Option::is_none")
    )]
    pub region: Option<Region>,

    /// Bucket to use for querying and inserting objects in.
    pub bucket: String,
}

impl From<S3StorageConfig> for aws_sdk_s3::Config {
    fn from(config: S3StorageConfig) -> aws_sdk_s3::Config {
        let mut cfg = aws_sdk_s3::Config::builder();
        cfg.set_credentials_provider(Some(SharedCredentialsProvider::new(Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "remi-rs",
        ))))
        .set_endpoint_url(config.endpoint.clone())
        .set_app_name(Some(
            AppName::new(config.app_name.clone().unwrap_or(String::from("remi-rs"))).unwrap(),
        ));

        if config.enforce_path_access_style {
            cfg.set_force_path_style(Some(true));
        }

        cfg.region(config.region).build()
    }
}

impl S3StorageConfig {
    /// Sets the [`enable_signer_v4_requests`](S3StorageConfig::enable_signer_v4_requests) property
    /// to whatever `enabled` is.
    pub fn with_enable_signer_v4_requests(mut self, yes: bool) -> Self {
        self.enable_signer_v4_requests = yes;
        self
    }

    /// Sets the [`enforce_path_access_style`](S3StorageConfig::enforce_path_access_style) property
    /// to what `enforce` is.
    pub fn with_enforce_path_access_style(mut self, yes: bool) -> Self {
        self.enforce_path_access_style = yes;
        self
    }

    /// Sets the [`default_object_acl`](S3StorageConfig::default_object_acl) property
    /// to what `acl` is.
    pub fn with_default_object_acl(&mut self, acl: Option<ObjectCannedAcl>) -> &mut Self {
        self.default_object_acl = acl;
        self
    }

    /// Sets the [`default_bucket_acl`](S3StorageConfig::default_bucket_acl) property
    /// to what `acl` is.
    pub fn with_default_bucket_acl(&mut self, acl: Option<BucketCannedAcl>) -> &mut Self {
        self.default_bucket_acl = acl;
        self
    }

    /// Sets the [`secret_access_key`](S3StorageConfig::secret_access_key) property
    /// to what `key` is.
    pub fn with_secret_access_key<I: Into<String>>(&mut self, key: I) -> &mut Self {
        self.secret_access_key = key.into();
        self
    }

    /// Sets the [`access_key_id`](S3StorageConfig::access_key_id) property
    /// to what `key` is.
    pub fn with_access_key_id<I: Into<String>>(&mut self, key: I) -> &mut Self {
        self.access_key_id = key.into();
        self
    }

    /// Sets the [`app_name`](S3StorageConfig::app_name) property to what `name` is.
    pub fn with_app_name<I: Into<String>>(&mut self, name: Option<I>) -> &mut Self {
        self.app_name = name.map(|i| i.into());
        self
    }

    /// Sets the [`endpoint`](S3StorageConfig::endpoint) property to what `endpoint` is.
    pub fn with_endpoint<I: Into<String>>(&mut self, endpoint: Option<I>) -> &mut Self {
        self.endpoint = endpoint.map(|i| i.into());
        self
    }

    /// Sets the [`prefix`](S3StorageConfig::prefix) property to what `prefix` is.
    pub fn with_prefix<I: Into<String>>(&mut self, prefix: Option<I>) -> &mut Self {
        self.prefix = prefix.map(|i| i.into());
        self
    }

    /// Sets the [`prefix`](S3StorageConfig::region) property to what `region` is.
    pub fn with_region<I: Into<Region>>(&mut self, region: Option<I>) -> &mut Self {
        self.region = region.map(|i| i.into());
        self
    }

    /// Sets the [`bucket`](S3StorageConfig::prefix) property to what `bucket` is.
    pub fn with_bucket<I: Into<String>>(&mut self, bucket: I) -> &mut Self {
        self.bucket = bucket.into();
        self
    }
}

#[cfg(feature = "serde")]
mod __serde {
    pub mod region {
        use aws_sdk_s3::config::Region;
        use serde::{de::Deserializer, ser::Serializer, Deserialize};
        use std::borrow::Cow;

        pub fn serialize<S: Serializer>(region: &Option<Region>, serializer: S) -> Result<S::Ok, S::Error> {
            match region {
                Some(region) => serializer.serialize_str(region.as_ref()),
                None => unreachable!(), // it shouldn't serialize if it is Option<None>
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Region>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Some(Region::new(Cow::Owned(s))))
        }
    }

    pub mod bucket_acl {
        use aws_sdk_s3::types::BucketCannedAcl;
        use serde::*;

        pub fn serialize<S: Serializer>(acl: &Option<BucketCannedAcl>, serializer: S) -> Result<S::Ok, S::Error> {
            match acl {
                Some(acl) => serializer.serialize_str(acl.as_str()),
                None => unreachable!(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<BucketCannedAcl>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Some(s.as_str().into()))
        }
    }

    pub mod object_acl {
        use aws_sdk_s3::types::ObjectCannedAcl;
        use serde::*;

        pub fn serialize<S: Serializer>(acl: &Option<ObjectCannedAcl>, serializer: S) -> Result<S::Ok, S::Error> {
            match acl {
                Some(acl) => serializer.serialize_str(acl.as_str()),
                None => unreachable!(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectCannedAcl>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Some(s.as_str().into()))
        }
    }
}
