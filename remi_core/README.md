# 🐻‍❄️🧶 remi_core crate
**remi_core** is the base API that all the supporting libraries are based off. This should only be referenced if you're creating your own storage service.

## Example Storage Service
```rust
use remi_core::StorageService;
use async_trait::async_trait;

struct MyStorageService;

#[async_trait]
impl StorageService for MyStorageService {
    /* omitted implementation */
}
```
