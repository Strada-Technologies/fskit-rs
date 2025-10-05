use std::path::PathBuf;
use std::process::Command;

use plist::Value;
use regex::Regex;

use crate::handler::Handler;
use crate::mounter::Mounter;
use crate::socket::Socket;
use crate::{Filesystem, MountOptions, mounter, socket};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Session {
    socket: Socket,
    mounter: Mounter,
}

impl Session {
    pub(super) async fn new<FS>(fs: FS, opts: MountOptions) -> Result<Self>
    where
        FS: Filesystem + Send + Sync + Clone + 'static,
    {
        let (server_port, fs_type) = read_config(&opts.fskit_id)?;

        let handler = Handler::new(fs);

        let socket = Socket::start(handler, server_port).await?;

        let mounter = match Mounter::mount(opts, &fs_type) {
            Ok(mount) => mount,
            Err(err) => {
                socket.stop().await;
                return Err(Error::Mounter(err));
            }
        };

        Ok(Self { socket, mounter })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.mounter.unmount().inspect_err(|err| eprintln!("{err}"));

        futures::executor::block_on(async {
            self.socket.stop().await;
        });
    }
}

fn read_config(fskit_id: &str) -> Result<(u16, String)> {
    // Get the output of the 'pluginkit' command
    let args = format!("-m -i {fskit_id} --raw");
    let output = Command::new("pluginkit")
        .args(args.split_whitespace())
        .output()?;
    let out = std::str::from_utf8(&output.stdout).unwrap_or_default();

    // Find the full path to Info.plist
    let reg = Regex::new(r#"(?m)^\s*path = "([^"]+)";"#).unwrap();
    let line = &reg
        .captures_iter(out)
        .last()
        .ok_or(Error::ExtensionNotRegistered)?;
    let info = PathBuf::from(format!("{}/Contents/Info.plist", &line[1]));

    // Parse Info.plist
    let data = std::fs::read(info)?;
    let root = Value::from_reader_xml(&*data).map_err(|_| Error::ExtensionConfig)?;

    // Get configuration
    let server_port = root
        .as_dictionary()
        .and_then(|d| d.get("Configuration"))
        .and_then(Value::as_dictionary)
        .and_then(|d| d.get("serverPort"))
        .and_then(Value::as_string)
        .and_then(|s| s.parse::<u16>().ok());

    let fs_type = root
        .as_dictionary()
        .and_then(|d| d.get("EXAppExtensionAttributes"))
        .and_then(Value::as_dictionary)
        .and_then(|d| d.get("FSFileSystemType"))
        .and_then(Value::as_string)
        .map(|s| s.to_string());

    if let Some(server_port) = server_port
        && let Some(fs_type) = fs_type
    {
        Ok((server_port, fs_type))
    } else {
        Err(Error::ExtensionConfig)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("file system extension not registered")]
    ExtensionNotRegistered,

    #[error("invalid file system extension Info.plist configuration")]
    ExtensionConfig,

    #[error(transparent)]
    Socket(#[from] socket::Error),

    #[error(transparent)]
    Mounter(#[from] mounter::Error),
}
