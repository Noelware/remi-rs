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
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::{ClientBuilder, ContainerClient};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageConfig {
    /// Credentials when contacting the Azure Blob Storage service.
    #[cfg_attr(feature = "serde", serde(default))]
    pub credentials: Credential,

    /// Location on the cloud that you're trying to access the Azure Blob Storage service.
    pub location: CloudLocation,

    /// Blob Storage container to grab any blob from.
    pub container: String,
}

impl StorageConfig {
    pub(crate) fn dummy() -> Self {
        StorageConfig {
            credentials: Credential::Anonymous,
            container: "dummy-test".into(),
            location: CloudLocation::Public("dummy".into()),
        }
    }
}

/// Credentials information for creating a blob container.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Credential {
    /// An access-key based credential.
    /// <https://docs.microsoft.com/azure/storage/common/storage-account-keys-manage>
    AccessKey { account: String, access_key: String },

    /// A shared access signature for temporary access to blobs.
    ///
    /// - <https://docs.microsoft.com/azure/storage/common/storage-sas-overview>
    /// - <https://docs.microsoft.com/azure/applied-ai-services/form-recognizer/create-sas-tokens>
    SASToken(String),

    /// OAuth2.0-based Bearer token credential.
    /// <https://docs.microsoft.com/rest/api/storageservices/authorize-with-azure-active-directory>
    Bearer(String),

    /// Anonymous credential, doesn't require further authentication.
    #[default]
    Anonymous,
}

impl TryFrom<Credential> for StorageCredentials {
    type Error = azure_core::Error;

    fn try_from(value: Credential) -> Result<Self, Self::Error> {
        match value {
            Credential::AccessKey { account, access_key } => {
                Ok(StorageCredentials::access_key(account, Secret::new(access_key)))
            }

            Credential::SASToken(token) => StorageCredentials::sas_token(token),
            Credential::Bearer(token) => Ok(StorageCredentials::bearer_token(token)),
            Credential::Anonymous => Ok(StorageCredentials::anonymous()),
        }
    }
}

impl TryFrom<StorageConfig> for ContainerClient {
    type Error = azure_core::Error;

    fn try_from(value: StorageConfig) -> Result<Self, Self::Error> {
        Ok(
            ClientBuilder::with_location::<StorageCredentials>(value.location.into(), value.credentials.try_into()?)
                .container_client(value.container),
        )
    }
}

/// Newtype enumeration around [`azure_core::CloudLocation`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum CloudLocation {
    /// Location that points to Microsoft Azure's Public Cloud.
    Public(String),

    /// Location that points to Microsoft Azure's China Cloud.
    China(String),

    /// Configures the location around emulation software of Azure Blob Storage.
    Emulator {
        /// Address to the emulator
        address: String,

        /// Port to the emulator
        port: u16,
    },

    /// Custom location that supports the Azure Blob Storage API.
    Custom {
        /// Account name.
        account: String,

        /// URI to point to the service.
        uri: String,
    },
}

impl From<CloudLocation> for azure_storage::CloudLocation {
    fn from(value: CloudLocation) -> Self {
        match value {
            CloudLocation::Public(account) => azure_storage::CloudLocation::Public { account },
            CloudLocation::China(account) => azure_storage::CloudLocation::China { account },
            CloudLocation::Emulator { address, port } => azure_storage::CloudLocation::Emulator { address, port },
            CloudLocation::Custom { account, uri } => azure_storage::CloudLocation::Custom { account, uri },
        }
    }
}
