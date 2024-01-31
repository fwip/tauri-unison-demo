use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use tinytemplate::TinyTemplate;
use std::io::prelude::*;


use toml;

#[derive(Deserialize, Serialize)]
struct UnisonConfig {
    ucm_version: String,
    project: String,
    branch: String,
    entrypoint: String,
    dependencies_cache: Vec<String>,
}

fn parse_cargo() -> Box<UnisonConfig> {
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").expect("could not read env");
    let cargo_toml = Path::new(&cargo_dir).join("Cargo.toml");
    let data_str = std::fs::read_to_string(cargo_toml).expect("could not read cargo.toml");
    let table: toml::Table = toml::from_str(&data_str).expect("could not parse Cargo.toml as TOML");
    let unison_str: String = toml::to_string(
        table
            .get("package")
            .expect("no Cargo.toml entry named package.metadata.unison_tauri")
            .get("metadata")
            .expect("no Cargo.toml entry named package.metadata.unison_tauri")
            .get("unison_tauri")
            .expect("no Cargo.toml entry named package.metadata.unison_tauri"),
    )
    .expect("could not reserialize package.metadata.unison_tauri");
    let config: UnisonConfig =
        toml::from_str(&unison_str).expect("could not parse package.metadata.unison_tauri");

    Box::new(config)
}

fn apply_template(filename: &Path, config: &UnisonConfig) {
    println!("cargo:rerun-if-changed={}", filename.to_str().unwrap());
    let new_filename = filename
        .parent().unwrap()
        .parent().unwrap()
        .join("build_scripts")
        .join(filename.file_name().unwrap());

    let new_path = filename
        .parent()
        .expect("no parent of template?")
        .join(new_filename);

    let mut tt = TinyTemplate::new();
    let template_str = std::fs::read_to_string(filename)
        .expect(&format!("could not read template file {:?}", filename));
    tt.add_template("temp", &template_str)
        .expect("could not add template");
    let new_contents = tt
        .render("temp", config)
        .expect("could not insert into template");

    fs::write(new_path, new_contents).expect("could not write file from template");
}

use std::io::Read;
use std::io::Cursor;



#[cfg(target_os = "macos")]
fn fetch_ucm(ucm_version: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::ffi::OsStr;

    use flate2::read::GzDecoder;
    use tar::Archive;
    let ucm_filename = "ucm-macos.tar.gz";
    let url = format!(
    "https://github.com/unisonweb/unison/releases/download/{ucm_version}/{ucm_filename}"
    );
    let response = reqwest::blocking::get(url)?;
    let content =  Cursor::new(response.bytes()?);
    let tar = GzDecoder::new(content);
    let mut archive = Archive::new(tar);
    let os_ucm: &OsStr = OsStr::new("ucm");
    for entry in archive.entries()? {
        if entry.is_ok() {
            let mut e = entry?;

            match e.path()?.file_name() {
                Some(x) if x == os_ucm => {
                    let mut v = Vec::with_capacity(e.size().try_into()?);
                    e.read_to_end(&mut v)?;
                    return Ok(v);
                },
                _ => (),
            }
        }
    }

    return Err("no matching entry in archive".into());
}

fn create_dirs(cargo_dir: &Path) -> Result<(), Box<dyn std::error::Error>>{
    std::fs::create_dir_all(cargo_dir.join("build_scripts"))?;
    std::fs::create_dir_all(cargo_dir.join("binaries"))?;
    std::fs::create_dir_all(cargo_dir.join("resources"))?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let unison_config = parse_cargo();
    let cargo_dir_val = env::var("CARGO_MANIFEST_DIR").expect("could not read env");
    let cargo_dir = Path::new(&cargo_dir_val);
    // Create necessary directories
    create_dirs(&cargo_dir).expect("could not create directories");;

    let template_dir = cargo_dir.join("build_script_templates");
    for template in fs::read_dir(template_dir).unwrap() {
        apply_template(
            &template.unwrap().path(),
            &unison_config,
        );
    }

    // Download the ucm binary if it doesn't exist
    let target_triple = env::var("TARGET").expect("cargo did not set $TARGET env");
    let ucm_binary_location = cargo_dir
        .join("binaries")
        .join(format!("ucm-{target_triple}"));
    if ! Path::exists(&ucm_binary_location) {
        let ucm_binary = match fetch_ucm(unison_config.ucm_version) {
            Ok(bin) => bin,
            Err(e) => {
                println!("Error downloading ucm: {e:?}");
                panic!("Error! {e:?}")
            },
        };

        // Save it to the correct location
        let _ = std::fs::File::create(ucm_binary_location)
            .expect("Could not open ucm binary location for writing")
            .write_all(&ucm_binary)
            .expect("Could not write ucm binary");
    }

    tauri_build::build()
}
