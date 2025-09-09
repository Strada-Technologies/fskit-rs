use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error::{MountFailed, MountPoint, UnmountFailed};
use crate::error::Result;
use crate::path;

#[derive(Debug)]
pub(super) struct Mounter {
    path: PathBuf,
}

impl Mounter {
    pub(super) fn mount(fs_type: &str, path: PathBuf) -> Result<Self> {
        // Check if the mount point exists
        if !path.exists() {
            return Err(MountPoint);
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
            return Err(MountFailed);
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
            return Err(UnmountFailed);
        }

        println!("Filesystem unmounted - mount point: {}", path!(self.path));

        Ok(())
    }
}
