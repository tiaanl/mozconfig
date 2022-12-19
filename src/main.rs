use std::fmt::Debug;
use std::path::{Path, PathBuf};

const MOZCONFIG_PREFIX: &str = ".mozconfig";

struct MozconfigDir {
    root: PathBuf,
}

impl std::fmt::Display for MozconfigDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

impl MozconfigDir {
    fn from_child_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let exists = {
            let full = path.as_ref().join(MOZCONFIG_PREFIX);
            std::fs::metadata(&full).is_ok()
        };

        if exists {
            Some(Self {
                root: path.as_ref().to_path_buf(),
            })
        } else {
            match path.as_ref().parent() {
                Some(parent) => Self::from_child_path(parent),
                None => None,
            }
        }
    }
}

fn main() {
    // Start from the current working directory.
    let start = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Could not detect current directory ({})", err);
            return;
        }
    };

    let mozconfig = match MozconfigDir::from_child_path(&start) {
        Some(root) => root,
        None => {
            eprintln!("\".mozconfig\" file not found. Run \"mozconfig --init\" to create one.");
            return;
        }
    };

    println!("root = {}", mozconfig);
}
