use std::path::Path;

use crate::Filesystem;
use crate::error::Result;
use crate::handler::Handler;
use crate::mounter::Mounter;
use crate::socket::Socket;

/// The session data structure
#[derive(Debug)]
pub struct Session<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    socket: Socket<FS>,
    mounter: Option<Mounter>,
}

impl<FS> Session<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
    pub(super) async fn new<P>(filesystem: FS, mount_point: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let handler = Handler::new(filesystem);

        let socket = Socket::start(handler).await?;

        let mounter = match Mounter::mount(mount_point.as_ref().to_path_buf()) {
            Ok(mount) => mount,
            Err(e) => {
                socket.stop().await;
                return Err(e);
            }
        };

        Ok(Self {
            socket,
            mounter: Some(mounter),
        })
    }
}

impl<FS> Drop for Session<FS>
where
    FS: Filesystem + Send + Sync + 'static,
{
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
