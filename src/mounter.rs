use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{MountOptions, path};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(super) struct Mounter {
    path: PathBuf,
}

impl Mounter {
    pub(super) fn mount(opts: MountOptions) -> Result<Self> {
        // Force unmount a filesystem
        if opts.force {
            let _ = unmount(&opts.mount_point);
        }

        // Check if the mount point exists
        if !opts.mount_point.exists() {
            return Err(Error::MountPointMissing);
        }

        // Create a blank disk image
        let dmg_path = format!("/tmp/fskit-{}", opts.fs_type);
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
        let args = format!(
            "-F -t {} -o port={} {device} {}",
            opts.socket_port,
            opts.fs_type,
            path!(opts.mount_point)
        );
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
            "Filesystem mounted - type: {}, mount point: {} ({device})",
            opts.fs_type,
            path!(opts.mount_point)
        );

        Ok(Self {
            path: opts.mount_point,
        })
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
