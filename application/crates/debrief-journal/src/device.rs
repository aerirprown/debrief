//! Per-install device id. Created once and stamped onto every file this install writes.

use std::fs;
use std::path::Path;

use uuid::Uuid;

use crate::error::{io, Error, Result};

/// Create a new random device id (UUID v4, lowercase hex without hyphens kept readable).
pub fn create_device_id() -> String {
    Uuid::new_v4().to_string()
}

/// Load a device id from `path`, or create and persist one if the file is missing.
pub fn load_or_create_device_id(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    match fs::read_to_string(path) {
        Ok(raw) => {
            let id = raw.trim();
            if id.is_empty() || id.contains(['\n', '\r', '/', '\\', ' ']) {
                return Err(Error::InvalidDeviceId {
                    path: path.to_path_buf(),
                });
            }
            Ok(id.to_string())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
            }
            let id = create_device_id();
            fs::write(path, format!("{id}\n")).map_err(|e| io(path, e))?;
            Ok(id)
        }
        Err(e) => Err(io(path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_is_unique() {
        let a = create_device_id();
        let b = create_device_id();
        assert_ne!(a, b);
        assert!(Uuid::parse_str(&a).is_ok());
    }

    #[test]
    fn load_or_create_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("device_id");
        let first = load_or_create_device_id(&path).unwrap();
        let second = load_or_create_device_id(&path).unwrap();
        assert_eq!(first, second);
    }
}
