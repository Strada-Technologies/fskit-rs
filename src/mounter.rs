use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error::{MountFailed, UnmountFailed};
use crate::error::Result;
use crate::path;

#[derive(Debug)]
pub(super) struct Mounter {
    path: PathBuf,
}

impl Mounter {
    pub(super) fn mount(path: PathBuf) -> Result<Self> {
        let mount = Self { path: path.clone() };

        // Create a blank disk image
        let image = Path::new("/tmp/fskit.dmg");
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

        // Check if already mounted
        let output = Command::new("mount").output()?;
        let out = std::str::from_utf8(&output.stdout).unwrap();
        // TODO: another path for device
        if out.contains(path!(path)) || out.contains(device) {
            mount.unmount()?;
        }

        // Check if the mount point exists
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }

        // Mount a filesystem
        let args = format!("-F -t BridgeFS {} {}", device, path!(path));
        if !Command::new("mount")
            .args(args.split_whitespace())
            .status()?
            .success()
        {
            Err(MountFailed)?
        }

        println!(
            "Filesystem mounted - type: BridgeFS, mount point: {} ({})",
            path!(path),
            device
        );

        Ok(mount)
    }

    pub(super) fn unmount(&self) -> Result<()> {
        // Unmount a filesystem
        if !Command::new("umount")
            .arg(self.path.clone())
            .status()?
            .success()
        {
            Err(UnmountFailed)?
        }

        println!(
            "Filesystem unmounted - type: BridgeFS, mount point: {}",
            path!(self.path)
        );

        Ok(())
    }
}
