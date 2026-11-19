//! Match a running process image path to a user-confirmed executable or folder.

use std::path::{Component, Path, PathBuf};

/// True when `running` is the confirmed executable, or lives under a confirmed folder.
pub fn path_matches(running: &Path, confirmed: &Path) -> bool {
    let Ok(run) = normalize(running) else {
        return false;
    };
    let Ok(conf) = normalize(confirmed) else {
        return false;
    };
    if conf.as_os_str().is_empty() || run.as_os_str().is_empty() {
        return false;
    }
    if eq_path(&run, &conf) {
        return true;
    }
    // Confirmed folder: running path is a descendant.
    is_descendant(&run, &conf)
}

#[cfg(windows)]
fn is_descendant(run: &Path, conf: &Path) -> bool {
    let run_s = run.to_string_lossy();
    let conf_s = conf.to_string_lossy();
    if run_s.len() <= conf_s.len() {
        return false;
    }
    if !run_s[..conf_s.len()].eq_ignore_ascii_case(&conf_s) {
        return false;
    }
    run_s.as_bytes().get(conf_s.len()) == Some(&b'\\')
        || run_s.as_bytes().get(conf_s.len()) == Some(&b'/')
}

#[cfg(not(windows))]
fn is_descendant(run: &Path, conf: &Path) -> bool {
    run.starts_with(conf)
}

fn normalize(path: &Path) -> Result<PathBuf, ()> {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => out.push(c.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(());
                }
            }
            Component::Normal(s) => out.push(s),
        }
    }
    Ok(out)
}

#[cfg(windows)]
fn eq_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}

#[cfg(not(windows))]
fn eq_path(a: &Path, b: &Path) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_exe_matches() {
        let exe = Path::new(r"C:\Games\Souls\game.exe");
        assert!(path_matches(exe, exe));
    }

    #[test]
    fn folder_prefix_matches() {
        let folder = Path::new(r"C:\Games\Souls");
        let exe = Path::new(r"C:\Games\Souls\bin\game.exe");
        assert!(path_matches(exe, folder));
    }

    #[test]
    fn sibling_does_not_match() {
        let folder = Path::new(r"C:\Games\Souls");
        let other = Path::new(r"C:\Games\Other\game.exe");
        assert!(!path_matches(other, folder));
    }
}
