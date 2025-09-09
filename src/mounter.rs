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
    pub(super) fn mount(fs_type: &str, path: PathBuf, force: bool) -> Result<Self> {
        // Force unmount a filesystem
        if force {
            let _ = unmount(&path);
        }

        // Check if the mount point exists
        if !path.exists() {
            return Err(Error::MountPointMissing);
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
        let status = Command::new("mount")
            .args(args.split_whitespace())
            .status()?;
        if !status.success() {
            return if status.code().unwrap_or(1) == 69 {
                Err(Error::MountPointBusy)
            } else {
                Err(Error::MountFailed)
            };
        }

        println!(
            "Filesystem mounted - type: {fs_type}, mount point: {} ({device})",
            path!(path)
        );

        Ok(Self { path })
    }

    pub(super) fn unmount(&self) -> Result<()> {
        unmount(&self.path)?;
        println!("Filesystem unmounted - mount point: {}", path!(self.path));
        Ok(())
    }
}

fn unmount(path: &PathBuf) -> Result<()> {
    if Command::new("umount")
        .arg("-f")
        .arg(path)
        .status()?
        .success()
    {
        Ok(())
    } else {
        Err(Error::UnmountFailed)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("mount point does not exist")]
    MountPointMissing,

    #[error("mount point is already in use")]
    MountPointBusy,

    #[error("unable to complete the mount request")]
    MountFailed,

    #[error("unable to complete the unmount request")]
    UnmountFailed,
}
