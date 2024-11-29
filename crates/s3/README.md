<div align="center">
    <h4>Official and maintained <code>remi-rs</code> crate for support of Amazon S3</h4>
    <kbd><a href="https://github.com/Noelware/remi-rs/releases/0.9.0">v0.10.0</a></kbd> | <a href="https://docs.rs/remi-gridfs">📜 Documentation</a>
    <hr />
</div>

| Crate Features  | Description                                                                          | Enabled by default? |
| :-------------- | :----------------------------------------------------------------------------------- | ------------------- |
| `export-crates` | Exports all the used AWS crates as a module called `aws`                             | Yes.                |
| `unstable`      | Tap into unstable features from `remi_gridfs` and the `remi` crate.                  | No.                 |
| [`tracing`]     | Enables the use of [`tracing::instrument`] and emit events for actions by the crate. | No.                 |
| [`serde`]       | Enables the use of **serde** in `StorageConfig`                                      | No.                 |
| [`log`]         | Emits log records for actions by the crate                                           | No.                 |

## Example
```rust,no_run
// Cargo.toml:
//
// [dependencies]
// remi = "^0"
// remi-s3 = { version = "^0", features = ["export-crates"] }
// tokio = { version = "^1", features = ["full"] }

use remi_s3::{StorageService, StorageConfig, aws::s3};
use remi::{StorageService as _, UploadRequest};

#[tokio::main]
async fn main() {
}
```

[`tracing::instrument`]: https://docs.rs/tracing/*/tracing/attr.instrument.html
[`tracing`]: https://crates.io/crates/tracing
[`serde`]: https://serde.rs
[`log`]: https://crates.io/crates/log
