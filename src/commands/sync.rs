use std::{fs, io::Write, path::Path};

static COMMUNITY_SOURCES_LINK: &str =
    "https://raw.githubusercontent.com/Wafelack/homebruh/dev/community/packages.list";

pub fn sync() -> crate::Result<()> {
    let (packages_path, bin_path) = {
        let base_directory = directories::ProjectDirs::from("", "", "home_bruh").unwrap();

        let packages_path = base_directory.data_dir();

        (packages_path.join("packages"), packages_path.join("bin"))
    };

    if !Path::new(&packages_path).exists() {
        crate::log!("Creating", "local package repository");
        fs::create_dir_all(&packages_path)?;
    }

    if !Path::new(&bin_path).exists() {
        crate::log!("Creating", "binary directory");
        fs::create_dir_all(&bin_path)?;
    }

    let list_of_packages = reqwest::blocking::get(COMMUNITY_SOURCES_LINK)?.bytes()?;

    crate::log!("Reading", "package database");

    let list_of_packages = std::str::from_utf8(&list_of_packages).unwrap().trim();
    let len = list_of_packages.chars().filter(|&a| a == '\n').count() + 1;

    crate::log!("Downloading", "packages manifests.");
    for (i, line) in list_of_packages.lines().enumerate() {
        let tomlfied = format!("{}.toml", line);

        let link = format!(
            "https://raw.githubusercontent.com/Wafelack/homebruh/dev/community/{}",
            tomlfied
        );

        let fcontent = reqwest::blocking::get(&link)?.bytes()?;

        let path = packages_path.join(tomlfied);

        let mut f = fs::File::create(path)?;
        f.write_all(&fcontent)?;

        print!("[");
        for _ in 0..(i + 1 / len * 50) {
            print!("#");
        }
        for _ in 0..((len - (i + 1)) / len * 50) {
            print!("-");
        }
        println!("] {}/{}", i + 1, len);
    }

    println!();
    crate::log!("Sucessfully", "synchronized package database.");

    Ok(())
}
