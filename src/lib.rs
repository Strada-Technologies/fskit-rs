use std::ffi::OsStr;
use std::path::Path;

pub use crate::error::{Error, Result};
pub use crate::pb::response::{ItemAttributes, ItemType, VolumeCapabilities, VolumeCaseFormat};
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
    fn set_vol_caps(&self) -> Result<VolumeCapabilities>;

    fn init(&mut self) -> Result<()>;

    /// Look up a directory entry by name and get its attributes.
    fn lookup(&mut self, parent: u64, name: &OsStr) -> Result<ItemAttributes>;

    fn getattr(&self, path: &str) -> Result<ItemAttributes>;

    fn setattr(&mut self, path: &str, attr: ItemAttributes) -> Result<()>;

    fn mkdir(&mut self, path: &str, mode: u32) -> Result<()>;

    fn unlink(&mut self, path: &str) -> Result<()>;

    fn rmdir(&mut self, path: &str) -> Result<()>;

    fn rename(&mut self, old: &str, new: &str) -> Result<()>;

    fn open(&self, path: &str, flags: i32) -> Result<()>;

    fn read(&self, path: &str, offset: u64, size: u32) -> Result<Vec<u8>>;

    fn write(&mut self, path: &str, offset: u64, data: &[u8]) -> Result<u32>;

    fn flush(&mut self, path: &str) -> Result<()>;

    fn readdir(&self, path: &str) -> Result<Vec<(String, String)>>;

    fn destroy(&mut self) -> Result<()>;
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
