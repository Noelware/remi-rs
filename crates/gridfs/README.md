# 🐻‍❄️🧶 MongoDB GridFS Support for `remi-rs`
**remi-gridfs** is an official implementation of using Remi with MongoDB GridFS by Noelware.

## Features
### serde (disabled)
Enables the use of [`serde`](https://docs.rs/serde) for (de)serializing for configuration files.

### log (disabled)
Enables the use of [`log`](https://docs.rs/log) for adding unstructured logging events to track down why something broke.

### tracing (disabled)
Enables the use of [`tracing::instrument`](https://docs.rs/tracing/*/tracing/attr.instrument.html) for adding spans to method calls to track down why something went wrong or to debug performance hits.
