use regex::Regex;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use thiserror::Error;

const MOZCONFIG_PREFIX: &str = ".mozconfig";

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration \"{0}\" does not exist")]
    ConfigDoesNotExist(String),
    #[error("An error occured: {0}")]
    Io(#[from] std::io::Error),
}

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

    /// Create a new [Mozconfig] by searching for configurations starting from
    /// the given path and each of it's parents.  If root (/) is reached without
    /// finding any, return None.
    pub fn from_child_path<P: AsRef<Path>>(start: P) -> Option<Self> {
        let exists = start.as_ref().join(MOZCONFIG_PREFIX).exists();

        if exists {
            Some(Self {
                root: start.as_ref().to_path_buf(),
            })
        } else {
            Self::from_child_path(start.as_ref().parent()?)
        }
    }

    /// Returns a list of available configurations in the root.
    pub fn list_configs(&self) -> Result<Vec<String>, Error> {
        let mut configs = vec![];
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(config) = Self::config_from_path(path) {
                configs.push(config)
            }
        }

        configs.sort();

        Ok(configs)
    }

    pub fn config_exists(&self, name: &str) -> bool {
        self.path_for_config(name).exists()
    }

    /// Return the name of the current configuration.
    pub fn current(&self) -> Result<Option<String>, Error> {
        let path = self.path_to_active_symlink();

        if !path.exists() {
            return Ok(None);
        }

        // Resolve the active symlink to the actual config.
        let real = path.canonicalize()?;

        Ok(Self::config_from_path(real))
    }

    pub fn create(&self, name: &str) -> Result<(), Error> {
        let path = self.path_for_config(name);

        let contents = format!("mk_add_options MOZ_OBJDIR=obj-{name}");
        std::fs::write(path, contents)?;

        println!("Configuration \"{name}\" created.");

        Ok(())
    }

    /// Activate the configuration with the given name.
    pub fn activate(&self, name: &str) -> Result<(), Error> {
        // Check if the config we want to activate exists.
        if !self.config_exists(name) {
            return Err(Error::ConfigDoesNotExist(name.to_string()));
        }

        let current_path = self.path_to_active_symlink();
        if current_path.exists() {
            std::fs::remove_file(&current_path)?;
        }

        // Create a link pointing to the configuration we want to make active.
        std::os::unix::fs::symlink(self.path_for_config(name), current_path)?;

        Ok(())
    }

    /// Given a name, construct an absolute path to the configuration file with
    /// the given name.
    pub fn path_for_config(&self, name: &str) -> PathBuf {
        self.root.join(format!("{MOZCONFIG_PREFIX}-{name}"))
    }

    /// Return an absolute path to the symlink used for the current
    /// configuration.
    fn path_to_active_symlink(&self) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn new_temp_root(name: &str) -> PathBuf {
        let full = std::env::temp_dir().join("mozconfig-tests").join(name);
        std::fs::remove_dir_all(&full).ok();
        std::fs::create_dir_all(&full).unwrap();
        full
    }

    #[test]
    fn start_with_empty() {
        let mozconfig = Mozconfig::from_path(new_temp_root("start_with_empty"));
        assert!(mozconfig.list_configs().unwrap().is_empty());
    }

    #[test]
    fn can_create_config() {
        let mozconfig = Mozconfig::from_path(new_temp_root("can_create_config"));
        assert!(mozconfig.create("test").is_ok());
        assert!(mozconfig.config_exists("test"));
        assert_eq!(mozconfig.list_configs().unwrap(), vec!["test"]);

        let path = mozconfig.path_for_config("test");
        assert!(std::fs::read_to_string(path).unwrap().contains("obj-test"));

        assert!(mozconfig.create("more").is_ok());
        assert!(mozconfig.config_exists("more"));
        assert_eq!(mozconfig.list_configs().unwrap(), vec!["more", "test"]);
    }

    #[test]
    fn can_make_active() {
        let mozconfig = Mozconfig::from_path(new_temp_root("can_make_active"));
        mozconfig.create("test").unwrap();
        mozconfig.create("debug").unwrap();
        mozconfig.create("release").unwrap();

        assert!(mozconfig.activate("test").is_ok());
        assert!(mozconfig.activate("debug").is_ok());
        assert!(mozconfig.activate("test").is_ok());
        assert!(mozconfig.activate("release").is_ok());

        assert!(matches!(
            mozconfig.activate("wrong"),
            Err(Error::ConfigDoesNotExist(name)) if name == "wrong"
        ));
    }

    #[test]
    fn from_child_path() {
        let root = new_temp_root("from_child_path");
        let mozconfig = Mozconfig::from_path(&root);
        mozconfig.create("test").unwrap();
        mozconfig.activate("test").unwrap();

        let child = root.join("child1").join("child2");

        std::fs::create_dir_all(&child).unwrap();

        let mozconfig = Mozconfig::from_child_path(child);
        assert!(mozconfig.is_some());
        let mozconfig = mozconfig.unwrap();
        assert_eq!(*mozconfig.root, root);
    }
}
