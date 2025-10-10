use std::fs;
use std::path::Path;
use std::process::Command;

use crate::info;
use crate::info::Info;

pub(super) type Result<T> = std::result::Result<T, Error>;

pub(super) fn run(path: &Path, force: bool) -> Result<()> {
    let apps = Path::new("/Applications");
    let app = apps.join(path.file_name().unwrap());
    let appex = app.join("Contents/Extensions/FSKitExt.appex");

    if !path.exists() {
        return Err(Error::AppNotFound);
    }

    if !appex.exists() {
        return Err(Error::ExtensionNotFound);
    }

    // 1. rm -rf /Applications/<app>
    if app.exists() {
        if !force {
            return Err(Error::AppInstalled);
        }
        fs::remove_dir_all(&app)?;
    }

    // 2. cp -r <path_to_app> /Applications
    run_cmd(
        "cp",
        &["-r", path.to_str().unwrap(), apps.to_str().unwrap()],
    )?;

    // 3. xattr -dr com.apple.quarantine /Applications/<app>
    run_cmd(
        "xattr",
        &["-dr", "com.apple.quarantine", app.to_str().unwrap()],
    )?;

    // 4. pluginkit -a /Applications/<app>/Contents/Extensions/FSKitExt.appex
    run_cmd("pluginkit", &["-a", appex.to_str().unwrap()])?;

    // 5. pluginkit -e use -i <fskit_id>
    let info = Info::new(&appex)?;
    run_cmd("pluginkit", &["-e", "use", "-i", &info.bundle_id()?])?;

    Ok(())
}

fn run_cmd(cmd: &str, args: &[&str]) -> std::io::Result<()> {
    Command::new(cmd).args(args).status()?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("host application not found")]
    AppNotFound,

    #[error("file system extension not found")]
    ExtensionNotFound,

    #[error("host application is already installed")]
    AppInstalled,

    #[error(transparent)]
    Info(#[from] info::Error),
}
