use std::ffi::OsStr;
use std::path::Path;

use async_trait::async_trait;
use tokio::sync::oneshot;

use fskit_rs::{
    DirectoryEntries, Filesystem, Item, ItemAttributes, ItemType, OpenMode, Result,
    VolumeCapabilities,
};

struct FsHandler;

#[async_trait]
impl Filesystem for FsHandler {
    async fn get_volume_capabilities(&mut self) -> Result<VolumeCapabilities> {
        todo!()
    }

    async fn get_attributes(&mut self, item_id: u64) -> Result<ItemAttributes> {
        todo!()
    }

    async fn set_attributes(
        &mut self,
        item_id: u64,
        attributes: ItemAttributes,
    ) -> Result<ItemAttributes> {
        todo!()
    }

    async fn lookup_item(&mut self, name: &OsStr, parent_id: u64) -> Result<Item> {
        todo!()
    }

    async fn create_item(
        &mut self,
        name: &OsStr,
        r#type: ItemType,
        parent_id: u64,
        attributes: ItemAttributes,
    ) -> Result<Item> {
        todo!()
    }

    async fn enumerate_directory(
        &mut self,
        item_id: u64,
        cookie: u64,
        verifier: u64,
    ) -> Result<DirectoryEntries> {
        todo!()
    }

    async fn open_item(&mut self, item_id: u64, modes: Vec<OpenMode>) -> Result<()> {
        todo!()
    }

    async fn close_item(&mut self, item_id: u64, modes: Vec<OpenMode>) -> Result<()> {
        todo!()
    }

    async fn read(&mut self, item_id: u64, offset: i64, length: i64) -> Result<Vec<u8>> {
        todo!()
    }

    async fn write(&mut self, contents: Vec<u8>, item_id: u64, offset: i64) -> Result<i64> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
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
