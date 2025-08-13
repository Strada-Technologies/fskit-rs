use std::ffi::{OsStr, OsString};
use std::path::Path;

use async_trait::async_trait;

pub use crate::error::{Error, Result};
pub use crate::pb::{ItemAttributes, ItemType, VolumeCapabilities, VolumeCaseFormat};
use crate::session::Session;

mod error;
mod handler;
mod mounter;
mod session;
mod socket;

mod pb {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

#[async_trait]
pub trait Filesystem {
    /// A property that provides the supported capabilities of the volume.
    async fn get_volume_capabilities(&mut self) -> Result<VolumeCapabilities>;

    /// Fetches attributes for the given item.
    async fn get_attributes(&mut self, file_id: u64) -> Result<ItemAttributes>;

    /// Looks up an item within a directory.
    async fn lookup_item(
        &mut self,
        name: &OsStr,
        parent_id: u64,
    ) -> Result<(ItemAttributes, OsString)>;

    /// Creates a new file or directory item.
    async fn create_item(
        &mut self,
        name: &OsStr,
        r#type: ItemType,
        parent_id: u64,
        attributes: ItemAttributes,
    ) -> Result<(ItemAttributes, OsString)>;
}

/// Mount the given filesystem to the given mountpoint. This function spawns
/// a background thread to handle filesystem operations while being mounted.
/// The returned handle should be stored to reference the mounted filesystem.
/// If it's dropped, the filesystem will be unmounted.
pub async fn mount<FS, P>(filesystem: FS, mount_point: P) -> Result<Session<FS>>
where
    FS: Filesystem + Send + Sync + 'static,
    P: AsRef<Path>,
{
    Session::new(filesystem, mount_point).await
}

#[macro_export]
macro_rules! path {
    ($arg:expr) => {
        $arg.as_os_str().to_str().unwrap()
    };
}
