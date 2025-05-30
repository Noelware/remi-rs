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

use crate::{default_resolver, ContentTypeResolver, StorageConfig};
use remi::{async_trait, Blob, Bytes, Directory, File, ListBlobsRequest, StorageService as _, UploadRequest};
use std::{
    borrow::Cow,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use tokio::{fs, io::*};

#[cfg(feature = "tracing")]
use tracing::instrument;

/// Represents an implementation of a [`StorageService`](remi::StorageService) for the
/// local filesystem.
#[derive(Clone)]
pub struct StorageService {
    resolver: Arc<dyn ContentTypeResolver>,
    config: StorageConfig,
}

impl StorageService {
    /// Creates a new [`StorageService`] instance.
    pub fn new<P: AsRef<Path>>(path: P) -> StorageService {
        Self::with_config(StorageConfig::new(path))
    }

    /// Creates a new [`StorageService`] instance with a provided configuration object.
    pub fn with_config(config: StorageConfig) -> StorageService {
        StorageService {
            resolver: Arc::new(default_resolver),
            config,
        }
    }

    /// Updates the given [`ContentTypeResolver`] to something else.
    pub fn with_resolver<R: ContentTypeResolver + 'static>(mut self, resolver: R) -> StorageService {
        self.resolver = Arc::new(resolver);
        self
    }

    /// Attempts to normalize a given path and returns a canonical, absolute
    /// path. It must follow some strict rules:
    ///
    /// * If the path starts with `./`, then it will resolve from [`StorageConfig::directory`] if
    ///   the directory was found. Otherwise, it'll use the current directory.
    ///
    /// * If the path starts with `~/`, then it will resolve from the home directory from [`etcetera::home_dir`].
    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.normalize",
            skip_all,
            fields(remi.service = "fs", path = %path.as_ref().display())
        )
    )]
    pub fn normalize<P: AsRef<Path>>(&self, path: P) -> io::Result<Option<PathBuf>> {
        let path = path.as_ref();

        #[cfg(feature = "tracing")]
        tracing::trace!("resolving path");

        #[cfg(feature = "log")]
        log::trace!("resolving path: {}", path.display());

        if path == self.config.directory {
            return std::fs::canonicalize(&self.config.directory).map(|x| Ok(Some(x)))?;
        }

        if path.starts_with("./") {
            let Some(directory) = self.normalize(&self.config.directory)? else {
                #[cfg(feature = "tracing")]
                tracing::warn!(
                    directory = %self.config.directory.display(),
                    "unable to resolve directory from config"
                );

                #[cfg(feature = "log")]
                log::warn!("unable to resolve given directory from config");

                return Ok(None);
            };

            let normalized = format!("{}/{}", directory.display(), path.strip_prefix("./").unwrap().display());

            #[cfg(feature = "tracing")]
            tracing::trace!(%normalized, "resolved path to");

            #[cfg(feature = "log")]
            log::trace!("resolved path {} ~> {normalized}", path.display());

            return Ok(Some(Path::new(&normalized).to_path_buf()));
        }

        if path.starts_with("~/") {
            let homedir = etcetera::home_dir()
                .inspect_err(|e| {
                    #[cfg(feature = "tracing")]
                    tracing::error!(error = %e, "failed to get home directory");

                    #[cfg(feature = "log")]
                    log::error!("failed to get home directory: {e}");

                    let _ = e;
                })
                .map_err(|_| <std::io::ErrorKind as Into<std::io::Error>>::into(io::ErrorKind::InvalidData))?;

            let normalized = format!("{}/{}", homedir.display(), path.strip_prefix("~/").unwrap().display());

            #[cfg(feature = "tracing")]
            tracing::trace!(%normalized, "resolved path to");

            #[cfg(feature = "log")]
            log::trace!("resolved path {} ~> {normalized}", path.display());

            return Ok(Some(Path::new(&normalized).to_path_buf()));
        }

        Ok(Some(path.to_path_buf()))
    }

    async fn create_file(&self, path: &Path) -> io::Result<File> {
        let metadata = path.metadata();
        let is_symlink = metadata.as_ref().map(|m| m.is_symlink()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let last_modified_at = match metadata {
            Ok(ref m) => Some(
                m.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| io::Error::other("clock went backwards?!"))?
                    .as_millis(),
            ),

            Err(_) => None,
        };

        let created_at = match metadata {
            Ok(ref m) => Some(
                m.created()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| io::Error::other("clock went backwards?!"))?
                    .as_millis(),
            ),

            Err(_) => None,
        };

        let bytes = self.open(path).await?.map_or(Bytes::new(), |x| x);
        let content_type = self.resolver.resolve(bytes.as_ref());

        Ok(File {
            last_modified_at,
            content_type: Some(content_type.to_string()),
            metadata: Default::default(),
            created_at,
            is_symlink,
            data: bytes,
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            path: format!("fs://{}", path.display()),
            size: size as usize,
        })
    }

    async fn create_file_from_entry(&self, path: &Path, entry: fs::DirEntry) -> io::Result<File> {
        let metadata = entry.metadata().await;
        let is_symlink = metadata.as_ref().map(|m| m.is_symlink()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let last_modified_at = match metadata {
            Ok(ref m) => Some(
                m.modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| io::Error::other("clock went backwards?!"))?
                    .as_millis(),
            ),

            Err(_) => None,
        };

        let created_at = match metadata {
            Ok(ref m) => Some(
                m.created()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| io::Error::other("clock went backwards?!"))?
                    .as_millis(),
            ),

            Err(_) => None,
        };

        let bytes = self.open(path).await?.map_or(Bytes::new(), |x| x);
        let content_type = self.resolver.resolve(bytes.as_ref());

        Ok(File {
            last_modified_at,
            content_type: Some(content_type.to_string()),
            metadata: Default::default(),
            created_at,
            is_symlink,
            data: bytes,
            name: entry.file_name().to_string_lossy().into_owned(),
            path: format!("fs://{}", path.display()),
            size: size as usize,
        })
    }
}

#[async_trait]
impl remi::StorageService for StorageService {
    type Error = io::Error;

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("remi:gridfs")
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.open",
            skip_all,
            fields(
                remi.service = "fs",
                directory = %self.config.directory.display()
            )
        )
    )]
    async fn init(&self) -> io::Result<()> {
        if !self.config.directory.try_exists()? {
            #[cfg(feature = "tracing")]
            tracing::info!("creating directory since it doesn't exist");

            #[cfg(feature = "log")]
            log::info!(
                "creating directory [{}] since it doesn't exist",
                self.config.directory.display(),
            );

            fs::create_dir_all(&self.config.directory).await?;
        }

        if !self.config.directory.is_dir() {
            #[cfg(not(no_io_errorkind))]
            return Err(Error::new(
                io::ErrorKind::NotADirectory,
                format!("path [{}] is a file, not a directory", self.config.directory.display()),
            ));

            #[cfg(no_io_errorkind)]
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("path [{}] is a file, not a directory", self.config.directory.display()),
            ));
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.open",
            skip_all,
            fields(
                remi.service = "fs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn open<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Bytes>> {
        let path = path.as_ref();
        let Some(path) = self.normalize(path)? else {
            #[cfg(feature = "tracing")]
            tracing::warn!("path given couldn't be normalized");

            #[cfg(feature = "log")]
            log::warn!("path given [{}] was a file, not a directory", path.display());

            return Ok(None);
        };

        if !path.try_exists()? {
            #[cfg(feature = "tracing")]
            tracing::warn!("path doesn't exist");

            #[cfg(feature = "log")]
            log::warn!("path [{}] doesn't exist", path.display());

            return Ok(None);
        }

        if path.is_dir() {
            #[cfg(not(no_io_errorkind))]
            return Err(Error::new(
                io::ErrorKind::NotADirectory,
                format!("path [{}] is a file, not a directory", self.config.directory.display()),
            ));

            #[cfg(no_io_errorkind)]
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                format!("path [{}] is a file, not a directory", self.config.directory.display()),
            ));
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("attempting to open file");

        #[cfg(feature = "log")]
        log::trace!("attempting to open file [{}]", path.display());

        let mut file = fs::OpenOptions::new()
            .create(false)
            .write(false)
            .read(true)
            .open(&path)
            .await?;

        let metadata = file.metadata().await?;
        let size = metadata.len();
        let mut buffer = vec![0; size as usize];

        buffer.resize(size as usize, 0);
        file.read_exact(&mut buffer).await?;

        Ok(Some(Bytes::from(buffer)))
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.blob",
            skip_all,
            fields(
                remi.service = "fs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn blob<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Option<Blob>> {
        let path = path.as_ref();
        let Some(path) = self.normalize(path)? else {
            #[cfg(feature = "tracing")]
            tracing::warn!("path given couldn't be normalized");

            #[cfg(feature = "log")]
            log::warn!("path given [{}] couldn't be normalized", path.display());

            return Ok(None);
        };

        if path.is_dir() {
            let metadata = path.metadata()?;
            let created_at = match metadata.created() {
                Ok(sys) => Some(
                    sys.duration_since(SystemTime::UNIX_EPOCH)
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "clock went backwards?!"))?
                        .as_millis(),
                ),

                Err(_) => None,
            };

            let name = path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or(Cow::Borrowed("<root or relative path>"))
                .to_string();

            return Ok(Some(Blob::Directory(Directory {
                created_at,
                name,
                path: format!("fs://{}", path.display()),
            })));
        }

        Ok(Some(Blob::File(self.create_file(&path).await?)))
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.blobs",
            skip_all,
            fields(
                remi.service = "fs",
                path = ?path.as_ref().map(|path| path.as_ref().display())
            )
        )
    )]
    async fn blobs<P: AsRef<Path> + Send>(
        &self,
        path: Option<P>,
        options: Option<ListBlobsRequest>,
    ) -> io::Result<Vec<Blob>> {
        let options = options.unwrap_or_default();
        let prefix = options.prefix.clone().unwrap_or_default();
        let path = match path {
            Some(ref p) => p.as_ref(),
            None => &self.config.directory,
        };

        let Some(path) = self.normalize(path)? else {
            #[cfg(feature = "tracing")]
            tracing::warn!("path given couldn't be normalized");

            #[cfg(feature = "log")]
            log::warn!("path given [{}] was a file, not a directory", path.display());

            return Ok(vec![]);
        };

        if path.is_file() {
            #[cfg(feature = "tracing")]
            tracing::warn!("path given was a file, not a directory");

            #[cfg(feature = "log")]
            log::warn!("path given [{}] was a file, not a directory", path.display());

            return Ok(vec![]);
        }

        let search = format!("{}{prefix}", path.display());
        #[cfg(feature = "tracing")]
        tracing::trace!(%search, "attempting to search all blobs in given path");

        #[cfg(feature = "log")]
        log::trace!(
            "attempting to search in [{search}] for all blobs in given path [{}]",
            path.display()
        );

        let mut files = fs::read_dir(search).await?;
        let mut blobs = vec![];

        while let Some(entry) = files.next_entry().await? {
            if entry.path().is_dir() && options.include_dirs {
                blobs.push(Blob::Directory(Directory {
                    created_at: match entry.metadata().await {
                        Ok(sys) => Some(
                            sys.created()?
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map_err(|_| io::Error::other("clock went backwards?!"))?
                                .as_millis(),
                        ),

                        Err(_) => None,
                    },

                    name: path
                        .file_name()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or(Cow::Borrowed("<root or relative path>"))
                        .to_string(),

                    path: format!("fs://{}", entry.path().display()),
                }));

                continue;
            }

            let path = entry.path();
            let ext_allowed = match path.extension() {
                Some(s) => options.is_ext_allowed(s.to_str().expect("valid utf-8 in path extension")),
                None => true,
            };

            if !ext_allowed {
                continue;
            }

            blobs.push(Blob::File(self.create_file_from_entry(&path, entry).await?));
        }

        Ok(blobs)
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.delete",
            skip_all,
            fields(
                remi.service = "fs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn delete<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        let Some(path) = self.normalize(path)? else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unable to normalize given path",
            ));
        };

        if path.is_dir() {
            #[cfg(feature = "tracing")]
            tracing::trace!("deleting directory");

            #[cfg(feature = "log")]
            log::trace!("deleting directory [{}]", path.display());

            fs::remove_dir(path).await?;
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("deleting file");

        #[cfg(feature = "log")]
        log::trace!("deleting file [{}]...", path.display());

        fs::remove_file(path).await
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.exists",
            skip_all,
            fields(
                remi.service = "fs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn exists<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<bool> {
        let path = path.as_ref();
        let Some(path) = self.normalize(path)? else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unable to normalize given path",
            ));
        };

        path.try_exists()
    }

    #[cfg_attr(
        feature = "tracing",
        instrument(
            name = "remi.filesystem.upload",
            skip_all,
            fields(
                remi.service = "fs",
                path = %path.as_ref().display()
            )
        )
    )]
    async fn upload<P: AsRef<Path> + Send>(&self, path: P, options: UploadRequest) -> io::Result<()> {
        let path = path.as_ref();
        let Some(path) = self.normalize(path)? else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unable to normalize given path",
            ));
        };

        if path.try_exists()? {
            #[cfg(feature = "tracing")]
            tracing::warn!("contents in given path will be overwritten");

            #[cfg(feature = "log")]
            log::trace!("contents in given path [{}] will be overwritten", path.display());
        }

        #[cfg(feature = "tracing")]
        tracing::warn!("uploading file");

        #[cfg(feature = "log")]
        log::trace!("uploading file [{}]", path.display());

        // ensure that the parent exists, if not, it'll attempt
        // to create all paths in the given parent
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::OpenOptions::new();
        file.write(true);

        if !path.try_exists()? {
            // atomically create the file if it doesn't exist
            file.create_new(true);
        }

        let mut file = file.open(path).await?;
        file.write_all(options.data.as_ref()).await?;
        file.flush().await?;

        Ok(())
    }

    #[cfg(feature = "unstable")]
    #[cfg_attr(any(noeldoc, docsrs), doc(cfg(feature = "unstable")))]
    async fn healthcheck(&self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // built to not repeat setup functionality
    macro_rules! build_testcases {
        ($(
            $(#[$meta:meta])*
            $name:ident($storage:ident) $code:block
        )*) => {
            $(
                $(#[$meta])*
                #[::tokio::test]
                async fn $name() -> ::std::io::Result<()> {
                    let tempdir = ::tempfile::tempdir().expect("failed to create tempdir");
                    assert!(fs::try_exists(&tempdir).await.expect("tempdir to actually exist"));

                    let $storage = $crate::StorageService::new(&tempdir);
                    ($storage).init().await.expect("initialization part to be successful");

                    assert!(fs::try_exists(tempdir).await.expect("should actually exist?!"));

                    let __ret: ::std::io::Result<()> = $code;
                    __ret
                }
            )*
        };
    }

    build_testcases! {
        init(_storage) {
            Ok(())
        }

        // open(storage) {
        //     #[cfg(feature = "tracing")]
        //     use tracing_subscriber::prelude::*;

        //     #[cfg(feature = "tracing")]
        //     let _guard = tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).set_default();

        //     // 1. upload the contents and see if we can do so
        //     let contents: remi::Bytes = "{\"wuff\":true}".into();
        //     storage.upload("./wuff.json", UploadRequest::default()
        //         .with_data(contents.clone())
        //         .with_content_type(Some(default_resolver(contents.as_ref())))
        //     ).await.expect("unable to upload ./wuff.json");

        //     // 2. assert that it exists
        //     assert!(storage.exists("./wuff.json").await?);

        //     // 3. open the file and check if it is the same
        //     assert_eq!(contents, storage.open("./wuff.json").await?.unwrap());

        //     Ok(())
        // }
    }
}
