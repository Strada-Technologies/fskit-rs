use std::ffi::OsStr;
use std::path::Path;

use async_trait::async_trait;
use tokio::sync::oneshot;

use fskit_rs::{
    DirectoryEntries, Error, Filesystem, Item, ItemAttributes, ItemType, OpenMode,
    PathConfOperations, Result, SetXattrPolicy, VolumeCapabilities, XattrOperations, Xattrs,
};

struct FsHandler;

#[async_trait]
impl Filesystem for FsHandler {
    async fn get_path_conf_operations(&mut self) -> Result<PathConfOperations> {
        Err(Error::Posix(libc::ENOSYS))
    }

    async fn get_volume_capabilities(&mut self) -> Result<VolumeCapabilities> {
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

    async fn create_item(
        &mut self,
        _name: &OsStr,
        _type: ItemType,
        _directory_id: u64,
        _attributes: ItemAttributes,
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
async fn main() {
    let (_stop_tx, stop_rx) = oneshot::channel::<()>();
    tokio::task::spawn_blocking(move || {
        futures::executor::block_on(async {
            let handler = FsHandler;

            let mount_point = Path::new("/tmp/fskit-rs-mount_point");

            let session = match fskit_rs::mount(handler, mount_point).await {
                Ok(session) => session,
                Err(err) => {
                    eprintln!("{}", err);
                    return;
                }
            };

            let _ = stop_rx.await;

            drop(session);
        });
    })
    .await
    .unwrap();
}
