// 🐻‍❄️🧶 remi-rs: Asynchronous Rust crate to handle communication between applications and object storage providers
// Copyright (c) 2022-2025 Noelware, LLC. <team@noelware.org>
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

// `cargo run --example filesystem` ~ gives you an overhead on how to work with the `remi_fs` library.
//
// > Cargo.toml:
// [dependencies]
// remi-fs = "*"
// remi = "*"
// tokio = { version = "*", features = ["full"] }

use remi::{Blob, StorageService as _, UploadRequest};
use remi_fs::{StorageConfig, StorageService};
use std::{io, path::PathBuf};
use tracing_subscriber::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), io::Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = StorageConfig {
        directory: PathBuf::from("./data"),
    };

    let fs = StorageService::with_config(config);

    eprintln!("init ./data dir");
    fs.init().await?;
    eprintln!("init ./data dir :: ok");

    // should fail if it exists
    assert!(!fs.exists("./weow.txt").await?);

    eprintln!("upload ./weow.txt");
    fs.upload(
        "./weow.txt",
        UploadRequest::default()
            .with_content_type(Some("text/plain; charset=utf-8"))
            .with_data("weow fluff"),
    )
    .await?;
    eprintln!("upload ./weow.txt :: ok");

    // now it should exist
    assert!(fs.exists("./weow.txt").await?);

    // there should be only one file available in ./data
    assert_eq!(fs.blobs::<&str>(None, None).await?.len(), 1);

    // get file info
    eprintln!("get blob ./weow.txt");
    let Some(blob) = fs.blob("./weow.txt").await? else {
        panic!("./weow.txt should exist");
    };

    eprintln!("get blob ./weow.txt :: ok");
    assert!(matches!(blob, Blob::File(_)));

    let Blob::File(blob) = blob else { unreachable!() };

    eprintln!("read blob ./weow.txt data");
    let content = String::from_utf8(blob.data.to_vec()).expect("valid utf-8"); // it should never fail
    eprintln!("read blob ./weow.txt data :: {content}");
    assert_eq!(content.trim(), "weow fluff");
    eprintln!("read blob ./weow.txt data :: ok");

    // delete the file
    eprintln!("delete blob ./weow.txt");
    fs.delete("./weow.txt").await?;

    // it should no longer exist
    assert!(!fs.exists("./weow.txt").await?);
    eprintln!("delete blob ./weow.txt :: ok");

    eprintln!("goodbye we're done :3");
    Ok(())
}
