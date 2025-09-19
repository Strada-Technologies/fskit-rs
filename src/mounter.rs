use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::{MountOptions, path};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(super) struct Mounter {
    path: PathBuf,
}

impl Mounter {
    pub(super) fn mount(opts: MountOptions) -> Result<Self> {
        // Force unmount a file system
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

        // Mount a file system
        let args = format!(
            "-F -t {} {} {}",
            opts.fs_type,
            device,
            path!(opts.mount_point)
        );
        let mut process = Command::new("mount")
            .args(args.split_whitespace())
            .stderr(Stdio::piped())
            .spawn()?;
        let start = Instant::now();
        loop {
            match process.try_wait()? {
                Some(status) => {
                    if status.success() {
                        break;
                    }
                    let stderr = process.wait_with_output()?.stderr;
                    let out = std::str::from_utf8(&stderr).unwrap();
                    eprintln!("{out}");
                    return if out.contains("is disabled") {
                        Err(Error::ExtensionDisabled)
                    } else if out.contains("Resource busy") {
                        Err(Error::MountPointBusy)
                    } else if out.contains("Probing resource") || out.contains("Loading resource") {
                        Err(Error::NeedReboot)
                    } else {
                        Err(Error::MountFailed)
                    };
                }
                None => {
                    if start.elapsed() >= Duration::from_secs(3) {
                        eprintln!("mount command hung, killing process");
                        let _ = process.kill();
                        return Err(Error::NeedReboot);
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        println!(
            "File system mounted - type: {}, mount point: {} ({device})",
            opts.fs_type,
            path!(opts.mount_point)
        );

        Ok(Self {
            path: opts.mount_point,
        })
    }

    pub(super) fn unmount(&self) -> Result<()> {
        unmount(&self.path)?;
        println!("File system unmounted - mount point: {}", path!(self.path));
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

    #[error("file system extension is disabled")]
    ExtensionDisabled,

    #[error("the command could not be completed, please reboot the system and try again")]
    NeedReboot,

    #[error("unable to complete the mount request")]
    MountFailed,

    #[error("unable to complete the unmount request")]
    UnmountFailed,
}
