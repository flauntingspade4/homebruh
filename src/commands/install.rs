use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{commands::download::download_package, Error, Result};

use flate2::read::GzDecoder;
use tar::Archive;
use toml::{value::Map, Value};

/// Installs `package` from the local list of packages
pub fn install_remote(package: &str) -> Result<()> {
    let bytes = download_package(package)?;

    install(bytes, package)?;

    Ok(())
}

/// Opens `file_name`, and attempts to install to local. Assums
/// the contents of `file_name` are already validated
pub fn install_from_file(file_name: &str) -> crate::Result<()> {
    match File::open(file_name) {
        Ok(mut f) => {
            let mut bytes = Vec::with_capacity(f.metadata().unwrap().len() as usize);
            f.read_to_end(&mut bytes)?;
            install(bytes::Bytes::from(bytes), file_name)
        }
        Err(_) => Err(Error::Other(format!("Cannot find file `{}`.", &file_name))),
    }
}

/// Installs package with name `package`, and bytes `bytes`
pub fn install(bytes: bytes::Bytes, package: &str) -> crate::Result<()> {
    let (dest_folder, bin_path) = {
        let base_directory = directories::ProjectDirs::from("", "", "home_bruh").unwrap();

        let packages_path = base_directory.data_dir();

        (
            packages_path
                .join("packages")
                .join(package.replace(".bpkg", "")),
            packages_path.join("bin"),
        )
    };

    crate::log!("Decompressing", "`{}`...", &package);

    Archive::new(GzDecoder::new(&*bytes)).unpack(&dest_folder)?;

    crate::log!("Sucessfully", "decompressed `{}`.", &package);

    let manifest_path = dest_folder.join("bruh.toml");

    let map: Map<String, Value> = {
        if let Ok(t) = fs::read_to_string(manifest_path) {
            crate::log!("Reading", "manifest information...");
            toml::from_str(&t)?
        } else {
            fs::remove_dir_all(dest_folder)?;
            return Err(Error::Other(
                "Cannot find `bruh.toml` in the package.".to_string(),
            ));
        }
    };

    if !map.contains_key("name") || !map.contains_key("version") || !map.contains_key("files") {
        fs::remove_dir_all(dest_folder)?;
        return Err(Error::Other(
            "One or more keys are missing from manifest.".to_string(),
        ));
    }

    if let Some(value) = map.get("startup_script") {
        crate::log!("Executing", "startup script...");

        let script = dest_folder.join(value.as_str().unwrap());

        run_script(script, &dest_folder)?;
    }

    let name = map["name"].as_str().unwrap();
    let version = map["version"].as_str().unwrap();
    let fs_path = dest_folder.join(map["files"].as_str().unwrap());

    if let Some(value) = map.get("to_export") {
        let to_export = value
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap());

        for export in to_export {
            crate::log!("Exporting", "{} to path", export);

            #[cfg(target_os = "windows")]
            std::os::windows::fs::symlink_file(fs_path.join(export), bin_path.join(export))
                .unwrap();
            #[cfg(target_os = "unix")]
            std::os::unix::fs::symlink(fs_path.join(export), bin_path.join(export)).unwrap();
        }
    }

    if let Some(value) = map.get("cleanup_script") {
        crate::log!("Executing", "cleanup script...");
        let script = dest_folder.join(value.as_str().unwrap());

        run_script(script, &dest_folder)?;
    }

    crate::log!("Cleaning", "packages files");
    crate::log!("Sucessfully", "installed {} v{}", name, version);

    Ok(())
}

/// Runs the script with path `script_path`, and will clean up the folder
/// `dest_folder` should the script fail
fn run_script(script_path: PathBuf, dest_folder: &PathBuf) -> crate::Result<()> {
    if !Path::new(&script_path).exists() {
        fs::remove_dir_all(dest_folder)?;
        return Err(Error::Other(format!(
            "Cannot find `{}` in the package.",
            script_path.to_str().unwrap()
        )));
    }

    let status = Command::new(&script_path).status()?;

    if !status.success() {
        fs::remove_dir_all(dest_folder)?;
        return Err(Error::Other(format!(
            "Script exited with an error code: {}.",
            status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}
