use std::ffi::OsStr;
use std::path::Path;

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

pub trait Filesystem {
    /// A property that provides the supported capabilities of the volume.
    fn get_volume_capabilities(&self) -> Result<VolumeCapabilities>;

    /// Fetches attributes for the given item.
    fn get_attributes(&self, file_id: u64) -> Result<ItemAttributes>;

    /// Looks up an item within a directory.
    fn lookup_item(&mut self, parent_id: u64, name: &OsStr) -> Result<ItemAttributes>;
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
