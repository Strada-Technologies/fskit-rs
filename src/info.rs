use std::path::Path;

use plist::{Dictionary, Value};

pub(super) type Result<T> = std::result::Result<T, Error>;

pub(super) struct Info {
    root: Dictionary,
}

impl Info {
    pub(super) fn new(path: &Path) -> Result<Self> {
        let root = Value::from_file(path.join("Contents/Info.plist"))?
            .into_dictionary()
            .ok_or(Error::Invalid)?;
        Ok(Self { root })
    }

    pub(super) fn bundle_id(&self) -> Result<String> {
        self.root
            .get("CFBundleIdentifier")
            .and_then(Value::as_string)
            .map(|s| s.to_string())
            .ok_or(Error::Invalid)
    }

    pub(super) fn server_port(&self) -> Result<u16> {
        self.root
            .get("Configuration")
            .and_then(Value::as_dictionary)
            .and_then(|d| d.get("serverPort"))
            .and_then(Value::as_string)
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or(Error::Invalid)
    }

    pub(super) fn fs_type(&self) -> Result<String> {
        self.root
            .get("EXAppExtensionAttributes")
            .and_then(Value::as_dictionary)
            .and_then(|d| d.get("FSFileSystemType"))
            .and_then(Value::as_string)
            .map(|s| s.to_string())
            .ok_or(Error::Invalid)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Plist(#[from] plist::Error),

    #[error("invalid Info.plist configuration")]
    Invalid,
}
