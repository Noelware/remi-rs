<div align="center">
    <h4>Official and maintained <code>remi-rs</code> crate for support of the local filesystem</h4>
    <kbd><a href="https://github.com/Noelware/remi-rs/releases/0.9.0">v0.9.0</a></kbd> | <a href="https://docs.rs/remi">📜 Documentation</a>
    <hr />
</div>

| Crate Features    | Description                                                                            | Enabled by default?  |
| :---------------- | :------------------------------------------------------------------------------------- | -------------------- |
| `unstable`        | Tap into unstable features from `remi_fs` and the `remi` crate.                        | No.                  |
| [`serde_json`]    | Uses the [`serde_json`] crate to detect JSON documents and return `application/json`   | No.                  |
| [`serde_yaml_ng`] | Allows to detect YAML documents with the [`serde_yaml_ng`] crate.                      | No.                  |
| [`file-format`]   | Uses the [`file-format`] crate to find media types on any external datatype.           | Yes.                 |
| [`tracing`]       | Enables the use of [`tracing::instrument`] and emit events for actions by the crate.   | No.                  |
| [`infer`]         | Uses the [`infer`] crate to infer external datatypes and map them to their media type. | Yes.                 |
| [`serde`]         | Enables the use of **serde** in `StorageConfig`                                        | No.                  |
| [`log`]           | Emits log records for actions by the crate                                             | No.                  |

## Example
```rust,no_run
// Cargo.toml:
//
// [dependencies]
// remi = "^0"
// remi-fs = "^0"
// tokio = { version = "^1", features = ["full"] }

use remi_fs::{StorageService, StorageConfig};
use remi::{StorageService as _, UploadRequest};

#[tokio::main]
async fn main() {
    // Initialize a `StorageService` that uses your local filesystem for storing files.
    let storage = StorageService::with_directory("./data");

    // Next, we will run the `init` function which will create
    // the ./data directory if it doesn't exist already.
    storage.init().await.unwrap();

    // We define a `UploadRequest`, which will set the content type to `text/plain` and set the
    // contents of `weow.txt` to `weow fluff`.
    let upload = UploadRequest::default()
        .with_content_type(Some("text/plain"))
        .with_data("weow fluff");

    // Let's upload it!
    storage.upload("./weow.txt", upload).await.unwrap();

    // Let's check if it exists! This `assert!` will panic if it failed
    // to upload.
    assert!(storage.exists("./weow.txt").await.unwrap());
}
```

[`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
[`serde_yaml_ng`]: https://crates.io/crates/serde_yaml_ng
[`file-format`]: https://crates.io/crates/file-format
[`serde_json`]: https://crates.io/crates/serde_json
[`tracing`]: https://crates.io/crates/tracing
[`infer`]: https://crates.io/crates/infer
[`serde`]: https://serde.rs
[`log`]: https://crates.io/crates/log
