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

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "export-azure")]
#[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "export-azure")))]
/// Exports the [`azure_core`], [`azure_storage`], and [`azure_storage_blobs`]
/// crates without defining them as owned dependencies.
pub mod core {
    pub use azure_core::*;

    /// Exports the [`azure_storage`] and [`azure_storage_blobs`]
    /// crates without defining them as owned dependencies.
    pub mod storage {
        pub use azure_storage::*;

        /// Exports the [`azure_storage_blobs`] crate without defining them as owned dependencies.
        pub mod blobs {
            pub use azure_storage_blobs::*;
        }
    }
}

mod config;
pub use config::*;

mod service;
pub use service::*;
