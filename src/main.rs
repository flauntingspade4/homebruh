#![feature(format_args_nl)]

mod commands;

use commands::{
    builder::build,
    install::{install_from_file, install_remote},
    sync::sync,
    uninstall::uninstall,
};

#[macro_export]
macro_rules! log {
    ($left:expr, $right:expr) => {
        {
            println!("\x1b[0;32m{}\x1b[0m {}", $left, $right)
        }
    };
    ($left:expr, $($arg:tt)*) => {
        {
            println!("\x1b[0;32m{}\x1b[0m {}", $left, format_args_nl!($($arg)*))
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        match arg.as_str() {
            "build" => build().unwrap(),
            "install" => match args.next() {
                Some(package) => match args.next() {
                    Some(inner_package) => {
                        if package == "-i" {
                            install_from_file(&inner_package)?;
                        }
                    }
                    None => install_remote(&package).unwrap(),
                },
                None => println!("Usage: {} install [-i] <package>.", env!("CARGO_PKG_NAME")),
            },
            "uninstall" => match args.next() {
                Some(package) => match args.next() {
                    Some(inner_package) => {
                        if package == "-i" {
                            uninstall(&inner_package).unwrap();
                        }
                    }
                    None => uninstall(&package).unwrap(),
                },
                None => println!("Usage: {} uninstall [-i] <package>", env!("CARGO_PKG_NAME")),
            },
            "sync" => sync().unwrap(),
            _ => help(),
        }
    }

    Ok(())
}

fn help() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("{}", env!("CARGO_PKG_AUTHORS"));
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));

    // Usage
    println!("\nUSAGE:");
    println!("\n{} <COMMAND> [OPTIONS]", env!("CARGO_PKG_NAME"));

    // Flags
    println!("\nFLAGS:");
    println!("\t--help   \tDisplays this message.");
    println!("\t--version\tDisplays version information.");

    // Commands
    println!("\nCOMMANDS:");
    println!("\tbuild                     \tBuilds the package reffering to `bruh.toml`.");
    println!("\tinstall -i $package_file  \tInstalls the specified package file.");
    println!("\tuninstall -i $package_file\tUninstalls the specified package file.");
    println!();
    println!("\tsync                      \tSynchronizes community database.");
    println!("\tinstall $package_name     \tInstalls the specified pacakge from the sources.");
    println!("\tuninstall $package_name   \tUninstalls the specified package.");

    println!();
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(TomlError),
    Other(String),
    Request(reqwest::Error),
}

#[derive(Debug)]
pub enum TomlError {
    DeError(toml::de::Error),
    SerError(toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Self::Toml(TomlError::SerError(e))
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(TomlError::DeError(e))
    }
}
