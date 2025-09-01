use std::path::Path;

use crate::Filesystem;
use crate::error::Result;
use crate::handler::Handler;
use crate::mounter::Mounter;
use crate::socket::Socket;

/// The session data structure
#[derive(Debug)]
pub struct Session {
    socket: Socket,
    mounter: Option<Mounter>,
}

impl Session {
    pub(super) async fn new<FS, P>(filesystem: FS, mount_point: P) -> Result<Self>
    where
        FS: Filesystem + Send + Sync + Clone + 'static,
        P: AsRef<Path>,
    {
        let handler = Handler::new(filesystem);

        let socket = Socket::start(handler).await?;

        let mounter = match Mounter::mount(mount_point.as_ref().to_path_buf()) {
            Ok(mount) => mount,
            Err(err) => {
                socket.stop().await;
                return Err(err);
            }
        };

        Ok(Self {
            socket,
            mounter: Some(mounter),
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(mounter) = self.mounter.take() {
            mounter.unmount().unwrap();
        }

        let socket = self.socket.clone();
        tokio::spawn(async move {
            socket.stop().await;
        });
    }
}
