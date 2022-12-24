mod mozconfig;

use std::path::PathBuf;

use mozconfig::Mozconfig;
use structopt::StructOpt;

enum ExitStatus {
    Success = 0,
    Error = 1,
}

fn create_configuration(
    name: &str,
    mozconfig: &Option<Mozconfig>,
) -> Result<ExitStatus, mozconfig::Error> {
    if let Some(ref mozconfig) = mozconfig {
        if !mozconfig.config_exists(name) {
            mozconfig.create(name)?;
            Ok(ExitStatus::Success)
        } else {
            eprintln!("Configuration \"{}\" already exists.", name);
            Ok(ExitStatus::Error)
        }
    } else {
        let root = std::env::current_dir()?;
        if let Err(err) = Mozconfig::from_path(&root).create(name) {
            eprintln!(
                "Could not create new configuration in {} ({})",
                root.display(),
                err
            );
            Ok(ExitStatus::Error)
        } else {
            Ok(ExitStatus::Success)
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// List all available configuratons.
    #[structopt(short, long, name = "list")]
    list: bool,

    /// Create a new .mozconfig configuration with the given name.
    #[structopt(short, long, name = "name")]
    create: Option<String>,

    /// Manually set the root where .mozconfig files should be searched for.
    #[structopt(short, long, name = "root")]
    root: Option<PathBuf>,
}

fn main_with_exit_status() -> ExitStatus {
    let opt = Opt::from_args();

    let start = if let Some(root) = opt.root {
        root
    } else {
        // Start from the current working directory.
        match std::env::current_dir() {
            Ok(path) => path,
            Err(err) => {
                eprintln!("Could not detect current directory ({})", err);
                return ExitStatus::Error;
            }
        }
    };

    let mozconfig = Mozconfig::from_child_path(start);

    // Handle any options that doesn't require a [Mozconfig].
    if let Some(name) = opt.create {
        return match create_configuration(name.as_str(), &mozconfig) {
            Ok(exit_status) => exit_status,
            Err(err) => {
                eprintln!("Could not create configuration ({})", err);
                ExitStatus::Error
            }
        };
    }

    let mozconfig = match mozconfig {
        Some(root) => root,
        None => {
            eprintln!(
                "\".mozconfig\" file not found. Run \"mozconfig --init <variant>\" to create one."
            );
            return ExitStatus::Error;
        }
    };

    if opt.list {
        match mozconfig.list_configs() {
            Ok(list) => {
                list.iter().for_each(|config| println!("{}", config));
            }

            Err(err) => {
                eprintln!("Could not list configurations ({})", err);
                return ExitStatus::Error;
            }
        }

        return ExitStatus::Success;
    }

    // Default command is to show the current configuration.
    if let Some(config) = mozconfig.current() {
        println!("{}", config);
        ExitStatus::Success
    } else {
        ExitStatus::Error
    }
}

fn main() {
    std::process::exit(main_with_exit_status() as i32);
}
