use crate::Error;

use sha2::{Digest, Sha256};
use toml::Value;

/// Attempts to download `package`, and will return the bytes downloaded.
/// Validates the download
pub fn download_package(package: &str) -> crate::Result<bytes::Bytes> {
    let path = {
        let base_directory = directories::ProjectDirs::from("", "", "home_bruh").unwrap();

        let packages_path = base_directory.data_dir();

        packages_path.join(format!("packages/{}.toml", package))
    };

    let pkg = match std::fs::read_to_string(&path) {
        Ok(string) => toml::from_str::<Value>(&string)?,
        Err(_) => return Err(Error::Other(format!("target not found: {:?}", path))),
    };

    let package_content = pkg.as_table().unwrap();

    if !package_content.contains_key("link") || !package_content.contains_key("sha256") {
        return Err(Error::Other("Invalid package manifest.".to_string()));
    }

    let sha256 = package_content["sha256"].as_str().unwrap();
    let link = package_content["link"].as_str().unwrap();
    let bytes = reqwest::blocking::get(link)?.bytes()?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let file_hash = format!("{:x}", hasher.finalize());

    if file_hash != sha256 {
        return Err(Error::Other(format!(
            "Invalid sha256 hash.\nExpected: {}\nFound: {}",
            sha256, file_hash
        )));
    }

    Ok(bytes)
}
