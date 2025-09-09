use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use async_trait::async_trait;
use tokio::signal;

use fskit_rs::{
    DirectoryEntries, Error, Filesystem, Item, ItemAttributes, ItemType, OpenMode,
    PathConfOperations, Result, SetXattrPolicy, StatFsResult, TaskOptions, VolumeCapabilities,
    XattrOperations, Xattrs, session,
};

#[derive(Clone)]
struct FsHandler;

#[async_trait]
impl Filesystem for FsHandler {
    async fn get_path_conf_operations(&mut self) -> Result<PathConfOperations> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_volume_capabilities(&mut self) -> Result<VolumeCapabilities> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_volume_statistics(&mut self) -> Result<StatFsResult> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn mount(&mut self, _options: TaskOptions) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn unmount(&mut self) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn synchronize(&mut self, _flags: u32) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_attributes(&mut self, _item_id: u64) -> Result<ItemAttributes> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn set_attributes(
        &mut self,
        _item_id: u64,
        _attributes: ItemAttributes,
    ) -> Result<ItemAttributes> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn lookup_item(&mut self, _name: &OsStr, _directory_id: u64) -> Result<Item> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn reclaim_item(&mut self, _item_id: u64) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn read_symbolic_link(&mut self, _item_id: u64) -> Result<Vec<u8>> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn create_item(
        &mut self,
        _name: &OsStr,
        _type: ItemType,
        _directory_id: u64,
        _attributes: ItemAttributes,
    ) -> Result<Item> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn create_symbolic_link(
        &mut self,
        _name: &OsStr,
        _directory_id: u64,
        _new_attributes: ItemAttributes,
        _contents: Vec<u8>,
    ) -> Result<Item> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn remove_item(
        &mut self,
        _item_id: u64,
        _name: &OsStr,
        _directory_id: u64,
    ) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn rename_item(
        &mut self,
        _item_id: u64,
        _source_directory_id: u64,
        _source_name: &OsStr,
        _destination_name: &OsStr,
        _destination_directory_id: u64,
        _over_item_id: Option<u64>,
    ) -> Result<Vec<u8>> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn enumerate_directory(
        &mut self,
        _directory_id: u64,
        _cookie: u64,
        _verifier: u64,
    ) -> Result<DirectoryEntries> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn activate(&mut self, _options: TaskOptions) -> Result<Item> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn deactivate(&mut self) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_xattr_operations(&mut self) -> Result<XattrOperations> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_xattr(&mut self, _name: &OsStr, _item_id: u64) -> Result<Vec<u8>> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn set_xattr(
        &mut self,
        _name: &OsStr,
        _value: Option<Vec<u8>>,
        _item_id: u64,
        _policy: SetXattrPolicy,
    ) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_xattrs(&mut self, _item_id: u64) -> Result<Xattrs> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn open_item(&mut self, _item_id: u64, _modes: Vec<OpenMode>) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn close_item(&mut self, _item_id: u64, _modes: Vec<OpenMode>) -> Result<()> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn read(&mut self, _item_id: u64, _offset: i64, _length: i64) -> Result<Vec<u8>> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn write(&mut self, _contents: Vec<u8>, _item_id: u64, _offset: i64) -> Result<i64> {
        Err(Error::Posix(libc::ENOSYS))
    }
}

#[tokio::main]
async fn main() -> session::Result<()> {
    let handler = FsHandler;

    let mount_point = Path::new("/tmp/fskit-mount-point");
    if !mount_point.exists() {
        fs::create_dir(mount_point)?;
    }

    let session = match fskit_rs::mount(handler, "BridgeFS", mount_point, true).await {
        Ok(session) => session,
        Err(err) => {
            eprintln!("{err}");
            return Err(err);
        }
    };

    signal::ctrl_c().await?;

    drop(session);

    Ok(())
}
