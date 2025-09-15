use std::ffi::OsStr;
use std::path::PathBuf;

use async_trait::async_trait;

pub use crate::pb::{
    CaseFormat, DirectoryEntries, Item, ItemAttributes, ItemType, OpenMode, PathConfOperations,
    ProbeResult, SetXattrPolicy, StatFsResult, TaskOptions, VolumeCapabilities, VolumeIdentifier,
    XattrOperations, Xattrs, directory_entries,
};
use crate::session::Session;

mod handler;
pub mod mounter;
pub mod session;
pub mod socket;

mod pb {
    include!(concat!(env!("OUT_DIR"), "/pb.rs"));
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Filesystem {
    /// Requests that the file system probe the specified resource.
    async fn probe_resource(&mut self) -> Result<ProbeResult>;

    /// Get the volume identifier and name.
    async fn get_volume_identifier(&mut self) -> Result<VolumeIdentifier>;

    /// Properties implemented by volumes that support providing the values of system limits or options.
    async fn get_path_conf_operations(&mut self) -> Result<PathConfOperations>;

    /// A property that provides the supported capabilities of the volume.
    async fn get_volume_capabilities(&mut self) -> Result<VolumeCapabilities>;

    /// A property that provides up-to-date statistics of the volume.
    async fn get_volume_statistics(&mut self) -> Result<StatFsResult>;

    /// Mounts this volume, using the specified options.
    async fn mount(&mut self, options: TaskOptions) -> Result<()>;

    /// Unmounts this volume.
    async fn unmount(&mut self) -> Result<()>;

    /// Synchronizes the volume with its underlying resource.
    async fn synchronize(&mut self, flags: u32) -> Result<()>;

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

    /// Reclaims an item, releasing any resources allocated for the item.
    async fn reclaim_item(&mut self, item_id: u64) -> Result<()>;

    /// Reads a symbolic link.
    async fn read_symbolic_link(&mut self, item_id: u64) -> Result<Vec<u8>>;

    /// Creates a new file or directory item.
    async fn create_item(
        &mut self,
        name: &OsStr,
        r#type: ItemType,
        directory_id: u64,
        attributes: ItemAttributes,
    ) -> Result<Item>;

    /// Creates a new symbolic link.
    async fn create_symbolic_link(
        &mut self,
        name: &OsStr,
        directory_id: u64,
        new_attributes: ItemAttributes,
        contents: Vec<u8>,
    ) -> Result<Item>;

    /// Removes an existing item from a given directory.
    async fn remove_item(&mut self, item_id: u64, name: &OsStr, directory_id: u64) -> Result<()>;

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

    /// Activates the volume using the specified options.
    async fn activate(&mut self, options: TaskOptions) -> Result<Item>;

    /// Tears down a previously initialized volume instance.
    async fn deactivate(&mut self) -> Result<()>;

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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("POSIX error: {0}")]
    Posix(std::ffi::c_int),
}

/// Configuration for mounting the filesystem and connecting between `FSKitExt` and `fskit-rs`.
///
/// # Parameters
/// * `socket_port` — TCP port for the local IPC endpoint. Default: `35367`.
/// * `fs_type` — Filesystem type selector. On macOS FSKit this must equal `FSFileSystemType`
///   in the appex Info.plist Default: `bridgefs`.
/// * `mount_point` — Existing (usually empty) directory to mount onto. Use `/Volumes/<Name>`
///   (may require `sudo`) or a user-owned path. Default: `/tmp/bridgefs-mount-point`.
/// * `force` — If `true`, preflight **unmounts** anything already mounted at `mount_point`
///   before mounting. Default: `true`.
#[derive(Debug, Clone)]
pub struct MountOptions {
    pub socket_port: u16,
    pub fs_type: String,
    pub mount_point: PathBuf,
    pub force: bool,
}

impl Default for MountOptions {
    fn default() -> Self {
        Self {
            socket_port: 35367,
            fs_type: "bridgefs".to_string(),
            mount_point: PathBuf::from("/tmp/bridgefs-mount-point"),
            force: true,
        }
    }
}

/// Mounts a user-space filesystem at `opts.mount_point` and returns a `Session` that
/// keeps the mount alive. Non-blocking: background workers serve kernel requests;
/// dropping `Session` cleanly unmounts.
///
/// # Parameters
/// * `fs` — Your `Filesystem` impl. Must be `Send + Sync + Clone + 'static`.
///   Prefer keeping heavy state in `Arc<_>`.
/// * `opts` — Combined mount/connection configuration.
///
/// # Returns
/// A `Session` handle; while it’s alive the mount remains active. Dropping it unmounts.
///
/// # macOS (FSKit) notes
/// * The extension must be **enabled** in System Settings (File System Extensions)
///   or via the in-app Extension Browser.
/// * FSKit mounts use `noowners`; you can store/report uid/gid in metadata,
///   but host POSIX enforcement still be disabled.
pub async fn mount<FS>(fs: FS, opts: MountOptions) -> session::Result<Session>
where
    FS: Filesystem + Send + Sync + Clone + 'static,
{
    Session::new(fs, opts).await
}

#[macro_export]
macro_rules! path {
    ($arg:expr) => {
        $arg.as_os_str().to_str().unwrap()
    };
}
