use std::fmt::Debug;
use std::path::{Path, PathBuf};

use regex::Regex;

const MOZCONFIG_PREFIX: &str = ".mozconfig";

pub struct Mozconfig {
    root: PathBuf,
}

impl std::fmt::Display for Mozconfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

impl Mozconfig {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            root: path.as_ref().to_path_buf(),
        }
    }

    pub fn from_child_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let exists = {
            let full = path.as_ref().join(MOZCONFIG_PREFIX);
            std::fs::metadata(full).is_ok()
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

    pub fn config_exists(&self, name: &str) -> bool {
        let path = self.path_for_config(name);
        std::fs::metadata(&path).is_ok()
    }

    /// Return the name of the current configuration.
    pub fn current(&self) -> Option<String> {
        let path = self.path_for_current();
        let link = match std::fs::read_link(path) {
            Ok(path) => {
                if path.is_relative() {
                    self.root.join(path)
                } else {
                    path
                }
            }
            Err(err) => {
                eprintln!("read_link error: {:?}", err);
                return None;
            }
        };

        Self::config_from_path(link)
    }

    pub fn create(&self, name: &str) -> Result<(), std::io::Error> {
        let path = self.path_for_config(name);

        let contents = format!("mk_add_options MOZ_OBJDIR=obj-{}", name);

        std::fs::write(&path, contents)?;

        println!("Configuration \"{}\" created.", name);

        Ok(())
    }

    /// Given a name, construct an absolute path to the configuration file with
    /// the given name.
    fn path_for_config(&self, name: &str) -> PathBuf {
        self.root.join(format!("{}-{}", MOZCONFIG_PREFIX, name))
    }

    /// Return name absolute path to the symlink used for the current
    /// configuration.
    fn path_for_current(&self) -> PathBuf {
        self.root.join(MOZCONFIG_PREFIX)
    }

    /// Retrieve the configuration name from an absolute path.  Returns None if
    /// the path is not a valid configuration.
    fn config_from_path(path: impl AsRef<Path>) -> Option<String> {
        let file_name = path.as_ref().file_name()?.to_str()?;
        if !file_name.starts_with(MOZCONFIG_PREFIX) {
            return None;
        }

        let regex = Regex::new(r"(?m)\.mozconfig-(\S+)$").unwrap();

        regex
            .captures(file_name)?
            .get(1)
            .map(|m| m.as_str().to_owned())
    }
}
