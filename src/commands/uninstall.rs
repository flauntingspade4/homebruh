use std::fs;

use crate::{Error, Result};

use toml::{value::Map, Value};

pub fn uninstall(input: &str) -> Result<()> {
    let (original_folder, bin_folder) = {
        let base_directory = directories::ProjectDirs::from("", "", "home_bruh").unwrap();

        let packages_path = base_directory.data_dir();

        (
            packages_path.join("packages").join(input),
            packages_path.join("bin"),
        )
    };

    let map: Map<String, Value> = {
        if let Ok(t) = fs::read_to_string(original_folder.join("bruh.toml")) {
            crate::log!("Reading", "manifest information...");
            toml::from_str(&t)?
        } else {
            fs::remove_dir_all(original_folder)?;
            return Err(Error::Other(
                "Cannot find `bruh.toml` in the package.".to_string(),
            ));
        }
    };

    if let Some(t) = map.get("to_export") {
        for export in t.as_array().unwrap() {
            match std::fs::remove_file(bin_folder.join(export.as_str().unwrap())) {
                Ok(_) => crate::log!("Removed", export),
                Err(e) => crate::log!("Failed to remove", "{} for reason {}", export, e),
            }
        }
    }

    std::fs::remove_dir_all(&original_folder)?;

    Ok(())
}
