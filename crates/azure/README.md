<div align="center">
    <h4>Official and maintained <code>remi-rs</code> crate for support of Microsoft's Azure Blob Storage</h4>
    <kbd><a href="https://github.com/Noelware/remi-rs/releases/0.10.0">v0.10.0</a></kbd> | <a href="https://docs.rs/remi">📜 Documentation</a>
    <hr />
</div>

| Crate Features | Description                                                                          | Enabled by default? |
| :------------- | :----------------------------------------------------------------------------------- | ------------------- |
| `export-azure` | Exports all the used Azure crates as a module called `core`                          | Yes.                |
| `unstable`     | Tap into unstable features from `remi_azure` and the `remi` crate.                   | No.                 |
| [`tracing`]    | Enables the use of [`tracing::instrument`] and emit events for actions by the crate. | No.                 |
| [`serde`]      | Enables the use of **serde** in `StorageConfig`                                      | No.                 |
| [`log`]        | Emits log records for actions by the crate                                           | No.                 |

## Example
```rust,no_run
// Cargo.toml:
//
// [dependencies]
// remi = "^0"
// remi-azure = "^0"
// tokio = { version = "^1", features = ["full"] }

use remi_azure::{StorageService, StorageConfig, Credential, CloudLocation};
use remi::{StorageService as _, UploadRequest};

#[tokio::main]
async fn main() {
    let storage = StorageService::new(StorageConfig {
        credentials: Credential::Anonymous,
        container: "my-container".into(),
        location: CloudLocation::Public("my-account".into()),
    }).unwrap();

    // Initialize the container. This will:
    //
    // * create `my-container` if it doesn't exist
    storage.init().await.unwrap();

    // Now we can upload files to Azure.

    // We define a `UploadRequest`, which will set the content type to `text/plain` and set the
    // contents of `weow.txt` to `weow fluff`.
    let upload = UploadRequest::default()
        .with_content_type(Some("text/plain"))
        .with_data("weow fluff");

    // Let's upload it!
    storage.upload("weow.txt", upload).await.unwrap();

    // Let's check if it exists! This `assert!` will panic if it failed
    // to upload.
    assert!(storage.exists("weow.txt").await.unwrap());
}
```

[`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
[`tracing`]: https://crates.io/crates/tracing
[`serde`]: https://serde.rs
[`log`]: https://crates.io/crates/log
