use std::ffi::OsStr;
use std::path::Path;

use tokio::sync::oneshot;

use fskit_rs::{Filesystem, ItemAttributes, Result, VolumeCapabilities};

struct FsHandler;

impl Filesystem for FsHandler {
    fn get_volume_capabilities(&self) -> Result<VolumeCapabilities> {
        todo!()
    }

    fn get_attributes(&self, file_id: u64) -> Result<ItemAttributes> {
        todo!()
    }

    fn lookup_item(&mut self, parent_id: u64, name: &OsStr) -> Result<ItemAttributes> {
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
