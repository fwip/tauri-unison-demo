use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path,PathBuf};
use tinytemplate::TinyTemplate;
use std::io::prelude::*;
use std::error::Error;


use toml;

#[derive(Deserialize, Serialize, PartialEq)]
struct UnisonConfig {
    ucm_version: String,
    project: String,
    branch: String,
    entrypoint: String,
    dependencies_cache: Vec<String>,
}

#[derive(Deserialize, Serialize, PartialEq)]
struct FullConfig {
    unison: UnisonConfig,
    target_triple: String,
    cargo_dir: PathBuf,
}


impl UnisonConfig {
    fn empty() -> Self {
        UnisonConfig {
            ucm_version: "".to_string(),
            project: "".to_string(),
            branch: "".to_string(),
            entrypoint: "".to_string(),
            dependencies_cache: vec!(),
        }
    }
}
impl FullConfig {
    fn empty() -> Self {
        FullConfig{
            unison: UnisonConfig::empty(),
            target_triple: "".into(),
            cargo_dir: "".into(),
        }
    }
}

fn parse_cargo(cargo_dir: &Path) -> Box<UnisonConfig> {
    let cargo_toml = Path::new(&cargo_dir).join("Cargo.toml");
    let data_str = fs::read_to_string(cargo_toml).expect("could not read cargo.toml");
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

/// Applies the template and creates the new build script
/// Returns 'true' if the new build script differs from the existing one, otherwise false.
fn apply_template(filename: &Path, config: &UnisonConfig) -> bool{
    println!("cargo:rerun-if-changed={}", filename.to_str().unwrap());
    let new_filename = filename
        .parent().unwrap()
        .parent().unwrap()
        .join("build")
        .join(filename.file_name().unwrap());

    let new_path = filename
        .parent()
        .expect("no parent of template?")
        .join(new_filename);

    let mut tt = TinyTemplate::new();
    let template_str = fs::read_to_string(filename)
        .expect(&format!("could not read template file {:?}", filename));
    tt.add_template("temp", &template_str)
        .expect("could not add template");
    let old_contents = match fs::read_to_string(new_path.clone()) {
        Ok(old) => old,
        Err(_) => String::new(),
    };
    let new_contents = tt
        .render("temp", config)
        .expect("could not insert into template");

    fs::write(new_path, new_contents.clone()).expect("could not write file from template");

    old_contents != new_contents
}

use std::io::Read;
use std::io::Cursor;



#[cfg(target_os = "macos")]
fn fetch_ucm(ucm_version: String) -> Result<Vec<u8>, Box<dyn Error>> {
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

fn create_dirs(cargo_dir: &Path) -> Result<(), Box<dyn Error>>{
    fs::create_dir_all(cargo_dir.join("build_scripts"))?;
    fs::create_dir_all(cargo_dir.join("binaries"))?;
    fs::create_dir_all(cargo_dir.join("resources"))?;
    fs::create_dir_all(cargo_dir.join("build"))?;
    Ok(())
}


fn download_ucm(cargo_dir: &Path, unison_config: &UnisonConfig ) -> Result<(), Box<dyn Error>>{
    // Download the ucm binary if it doesn't exist
    let target_triple = env::var("TARGET").expect("cargo did not set $TARGET env");
    let ucm_binary_location = cargo_dir
        .join("binaries")
        .join(format!("ucm-{target_triple}"));
    if ! Path::exists(&ucm_binary_location) {
        let ucm_binary = match fetch_ucm(unison_config.ucm_version.clone()) {
            Ok(bin) => bin,
            Err(e) => {
                println!("Error downloading ucm: {e:?}");
                panic!("Error! {e:?}")
            },
        };

        // Save it to the correct location
        let _ = fs::File::create(ucm_binary_location.clone())
            .expect("Could not open ucm binary location for writing")
            .write_all(&ucm_binary)
            .expect("Could not write ucm binary");
        // Change the permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                ucm_binary_location,
                PermissionsExt::from_mode(0o755)
            ).expect("Could not set permissions");
        }
        // TODO: Windows support
    }

    Ok(())
}

fn read_last_config(cargo_dir: &Path) -> Result<FullConfig, Box<dyn Error>>{
    let config_location = cargo_dir.join("build/config.json");
    let config_f = fs::File::open(config_location)?;
    let config = serde_json::from_reader(config_f)?;

    Ok(config)

}
fn save_config(config: &FullConfig) -> Result<(), Box<dyn Error>> {
    let config_location = config.cargo_dir.join("build/config.json");
    let config_f = fs::File::create(config_location)?;
    serde_json::to_writer(config_f, config)?;
    Ok(())
}

fn safe_remove(path: &Path) {
    if !path.exists() {
        return
    }
    // Make sure they live in the manifest directory
    let manifest_str = &env::var("CARGO_MANIFEST_DIR").expect("Could not read manifest_dir");
    let manifest_dir = Path::new(&manifest_str);
    let canon = path.canonicalize().expect("could not canonicalize path to remove");
    let canon_root = manifest_dir.canonicalize().expect("could not canonicalize manifest_dir to remove");
    if ! canon.starts_with(canon_root) {
        panic!("Can't remove path '{path:?}' because it's not in {manifest_dir:?}");
    }
    if canon.is_file() {
        println!("Removing file {path:?}");
        fs::remove_file(canon).expect("Could not delete file");
    } else if canon.is_dir() {
        println!("Removing dir {path:?}");
        fs::remove_dir_all(canon).expect("Could not delete directory");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let cargo_dir_str = &env::var("CARGO_MANIFEST_DIR").expect("could not read env");
    let cargo_dir: PathBuf = cargo_dir_str.into();
    let target_triple = env::var("TARGET").expect("could not read env");
    let unison_config = parse_cargo(&cargo_dir.clone());
    let config = FullConfig{
        unison: *unison_config,
        cargo_dir: cargo_dir.clone(),
        target_triple,
    };

    let last_config = match read_last_config(&cargo_dir) {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading last config file {e:?}, using empty config");
            FullConfig::empty()
        }
    };

    create_dirs(&cargo_dir).expect("could not create directories");


    use std::process::Command;
    let target_triple = env::var("TARGET").expect("cargo did not set $TARGET env");
    let ucm_binary_location = cargo_dir
        .join("binaries")
        .join(format!("ucm-{target_triple}"));

    let base_location = cargo_dir.join("build").join("base");
    let project_location = cargo_dir.join("build").join("project");
    let resource_location = cargo_dir.join("resources");
    let main_location = resource_location.join("main.uc");

    // Remove stale files
    // UCM or target changed, clean everything
    if (last_config.unison.ucm_version != config.unison.ucm_version)
    || (last_config.target_triple != config.target_triple)
    {
        safe_remove(&ucm_binary_location);
        safe_remove(&base_location);
        safe_remove(&project_location);
        safe_remove(&main_location);
    }

    // Create UCM transcripts and see if they've changed
    let fetch_transcript_location = cargo_dir.join("build_script_templates").join("fetch_base.md");
    let compile_transcript_location = cargo_dir.join("build_script_templates").join("compile_main.md");
    let base_changed = apply_template(&fetch_transcript_location, &config.unison);
    let project_changed = apply_template(&compile_transcript_location, &config.unison);

    // If the fetch_base transcript changed, we need to fetch new dependencies.
    if base_changed {
        safe_remove(&base_location);
        safe_remove(&project_location);
        safe_remove(&main_location);
    }
    // If the compile_main transcript changed, we need to recompile a new main.
    if project_changed {
        safe_remove(&project_location);
        safe_remove(&main_location);
    }

    // Actually build everything
    // Download UCM binary for this platform
    if !Path::exists(&ucm_binary_location){
        download_ucm(&cargo_dir, &config.unison).expect("Could not download UCM");
    }


    // Run fetch_base transcript
    if !Path::exists(&base_location.join(".unison")){
        let template_location = cargo_dir.join("build").join("fetch_base.md");
        println!("Fetching dependencies");
        let result = Command::new(ucm_binary_location.clone())
            .arg("transcript")
            .arg("--save-codebase-to")
            .arg(base_location.clone())
            .arg(template_location)
            .status()
            .expect("Could not fetch base");
        if !result.success() {
            panic!("could not fetch base (see log for details)");
        }

    }
    // Run compile transcript
    // This generates main.uc
    if !Path::exists(&project_location.join(".unison"))
    || !Path::exists(&main_location){
        println!("Compiling");
        // Transcript is not idempotent, so remove existing before rerunning
        safe_remove(&project_location);
        safe_remove(&main_location);
        let template_location = cargo_dir.join("build").join("compile_main.md");
        let result = Command::new(ucm_binary_location)
            .current_dir(resource_location)
            .arg("transcript")
            .arg("--codebase").arg(base_location)
            .arg("--save-codebase-to").arg(project_location)
            .arg(template_location)
            .status()
            .expect("Could not compile target");
        if !result.success() {
            panic!("could not fetch base (see log for details)");
        }
    }

    if !main_location.exists() {
        panic!("{main_location:?} does not exist at end of build");
    }

    save_config(&config).expect("Could not save build config to file");

    tauri_build::build()
}
