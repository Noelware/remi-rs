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

use azure_core::auth::Secret;
use azure_storage::{CloudLocation, StorageCredentials};
use azure_storage_blobs::prelude::{ClientBuilder, ContainerClient};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageConfig {
    /// Credentials when contacting the Azure Blob Storage service.
    #[cfg_attr(feature = "serde", serde(default))]
    pub credentials: Credential,

    /// Location on the cloud that you're trying to access the Azure Blob Storage service.
    #[cfg_attr(feature = "serde", serde(with = "azure_serde::cloud_location"))]
    pub location: CloudLocation,

    /// Blob Storage container to grab any blob from.
    pub container: String,
}

impl StorageConfig {
    pub(crate) fn dummy() -> Self {
        StorageConfig {
            credentials: Credential::Anonymous,
            container: "dummy-test".into(),
            location: CloudLocation::Public {
                account: "dummy".into(),
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Credential {
    AccessKey {
        account: String,
        access_key: String,
    },

    SASToken(String),
    Bearer(String),

    #[default]
    Anonymous,
}

impl From<Credential> for StorageCredentials {
    fn from(value: Credential) -> Self {
        match value {
            Credential::AccessKey { account, access_key } => {
                StorageCredentials::access_key(account, Secret::new(access_key))
            }

            Credential::SASToken(token) => StorageCredentials::sas_token(token).expect("valid shared access signature"),
            Credential::Bearer(token) => StorageCredentials::bearer_token(token),
            Credential::Anonymous => StorageCredentials::anonymous(),
        }
    }
}

impl From<StorageConfig> for ContainerClient {
    fn from(value: StorageConfig) -> Self {
        ClientBuilder::with_location::<StorageCredentials>(value.location, value.credentials.into())
            .container_client(value.container)
    }
}

#[cfg(feature = "serde")]
pub(crate) mod azure_serde {
    pub(crate) mod cloud_location {
        use azure_storage::CloudLocation;
        use serde::{
            ser::{SerializeMap, Serializer},
            Deserialize, Deserializer,
        };
        use std::collections::HashMap;

        pub fn serialize<S: Serializer>(value: &CloudLocation, serializer: S) -> Result<S::Ok, S::Error> {
            match value {
                CloudLocation::Public { account } => {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry("public", &account)?;
                    map.end()
                }

                CloudLocation::China { account } => {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry("china", &account)?;
                    map.end()
                }

                CloudLocation::Emulator { address, port } => {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry("emulator", &format!("{address}:{port}"))?;
                    map.end()
                }

                CloudLocation::Custom { .. } => {
                    unimplemented!("not supported (yet)")
                }
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<CloudLocation, D::Error> {
            use serde::de::Error;

            let map = HashMap::<String, String>::deserialize(deserializer)?;
            if let Some(val) = map.get("public") {
                return Ok(CloudLocation::Public {
                    account: val.to_owned(),
                });
            }

            if let Some(val) = map.get("china") {
                return Ok(CloudLocation::China {
                    account: val.to_owned(),
                });
            }

            if let Some(mapping) = map.get("emulator") {
                let Some((addr, port)) = mapping.split_once(':') else {
                    return Err(D::Error::custom(format!("failed to parse {mapping} as 'addr:port'")));
                };

                if port.contains(':') {
                    return Err(D::Error::custom("address:port mapping in `emulator` key "));
                }

                return Ok(CloudLocation::Emulator {
                    address: addr.to_owned(),
                    port: port
                        .parse()
                        .map_err(|err| D::Error::custom(format!("failed to parse {port} as u16: {err}")))?,
                });
            }

            Err(D::Error::custom("unhandled"))
        }
    }
}
