use std::ffi::OsStr;
use std::path::Path;

use async_trait::async_trait;

pub use crate::error::{Error, Result};
pub use crate::pb::response::{DirectoryEntries, Item, Xattrs, directory_entries};
pub use crate::pb::{
    CaseFormat, ItemAttributes, ItemType, OpenMode, PathConfOperations, SetXattrPolicy,
    VolumeCapabilities, XattrOperations,
};
use crate::session::Session;

mod error;
mod handler;
mod mounter;
mod session;
mod socket;

mod pb {
    include!(concat!(env!("OUT_DIR"), "/pb.rs"));
}

#[async_trait]
pub trait Filesystem {
    /// Properties implemented by volumes that support providing the values of system limits or options.
    async fn get_path_conf_operations(&mut self) -> Result<PathConfOperations>;

    /// A property that provides the supported capabilities of the volume.
    async fn get_volume_capabilities(&mut self) -> Result<VolumeCapabilities>;

    /// Fetches attributes for the given item.
    async fn get_attributes(&mut self, item_id: u64) -> Result<ItemAttributes>;

    /// Sets the given attributes on an item.
    async fn set_attributes(
        &mut self,
        item_id: u64,
        attributes: ItemAttributes,
    ) -> Result<ItemAttributes>;

    /// Looks up an item within a directory.
    async fn lookup_item(&mut self, name: &OsStr, directory_id: u64) -> Result<Item>;

    /// Creates a new file or directory item.
    async fn create_item(
        &mut self,
        name: &OsStr,
        r#type: ItemType,
        directory_id: u64,
        attributes: ItemAttributes,
    ) -> Result<Item>;

    /// Renames an item from one path in the file system to another.
    async fn rename_item(
        &mut self,
        item_id: u64,
        source_directory_id: u64,
        source_name: &OsStr,
        destination_name: &OsStr,
        destination_directory_id: u64,
        over_item_id: Option<u64>,
    ) -> Result<Vec<u8>>;

    /// Enumerates the contents of the given directory.
    async fn enumerate_directory(
        &mut self,
        directory_id: u64,
        cookie: u64,
        verifier: u64,
    ) -> Result<DirectoryEntries>;

    /// Properties implemented by volumes that natively or partially support extended attributes.
    async fn get_xattr_operations(&mut self) -> Result<XattrOperations>;

    /// Gets the specified extended attribute of the given item.
    async fn get_xattr(&mut self, name: &OsStr, item_id: u64) -> Result<Vec<u8>>;

    /// Sets the specified extended attribute data on the given item.
    async fn set_xattr(
        &mut self,
        name: &OsStr,
        value: Option<Vec<u8>>,
        item_id: u64,
        policy: SetXattrPolicy,
    ) -> Result<()>;

    /// Gets the list of extended attributes currently set on the given item.
    async fn get_xattrs(&mut self, item_id: u64) -> Result<Xattrs>;

    /// Opens a file for access.
    async fn open_item(&mut self, item_id: u64, modes: Vec<OpenMode>) -> Result<()>;

    /// Closes a file from further access.
    async fn close_item(&mut self, item_id: u64, modes: Vec<OpenMode>) -> Result<()>;

    /// Reads the contents of the given file item.
    async fn read(&mut self, item_id: u64, offset: i64, length: i64) -> Result<Vec<u8>>;

    /// Writes contents to the given file item.
    async fn write(&mut self, contents: Vec<u8>, item_id: u64, offset: i64) -> Result<i64>;
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
