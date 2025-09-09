use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::path;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(super) struct Mounter {
    path: PathBuf,
}

impl Mounter {
    pub(super) fn mount(fs_type: &str, path: PathBuf) -> Result<Self> {
        // Check if the mount point exists
        if !path.exists() {
            return Err(Error::MountPoint);
        }

        // Create a blank disk image
        let dmg_path = format!("/tmp/fskit-{fs_type}").to_lowercase();
        let image = Path::new(&dmg_path);
        if !image.exists() {
            File::create(image)?;
        }

        // Attach the raw image as a virtual disk
        let args = format!(
            "attach -imagekey diskimage-class=CRawDiskImage -nomount {}",
            path!(image)
        );
        let output = Command::new("hdiutil")
            .args(args.split_whitespace())
            .output()?;
        let device = std::str::from_utf8(&output.stdout).unwrap().trim();

        // Mount a filesystem
        let args = format!("-F -t {fs_type} {device} {}", path!(path));
        if !Command::new("mount")
            .args(args.split_whitespace())
            .status()?
            .success()
        {
            return Err(Error::MountFailed);
        }

        println!(
            "Filesystem mounted - type: {fs_type}, mount point: {} ({device})",
            path!(path)
        );

        Ok(Self { path })
    }

    pub(super) fn unmount(&self) -> Result<()> {
        // Unmount a filesystem
        if !Command::new("umount")
            .arg("-f")
            .arg(self.path.clone())
            .status()?
            .success()
        {
            return Err(Error::UnmountFailed);
        }

        println!("Filesystem unmounted - mount point: {}", path!(self.path));

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Mount failed: mount point does not exist")]
    MountPoint,

    #[error("Mount failed: unable to complete the request")]
    MountFailed,

    #[error("Unmount failed: unable to complete the request")]
    UnmountFailed,
}
