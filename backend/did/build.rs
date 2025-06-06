use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use candid_parser::bindings::rust::{Config, ExternalConfig, compile};
use candid_parser::configs::Configs;
use candid_parser::pretty_check_file;

const ORBIT_STATION_REPO_URL: &str = "https://github.com/dfinity/orbit";
const ORBIT_STATION_TAG_NAME: &str = "@orbit/station-v0.5.0";

const ORBIT_STATION_CANDID_SPEC_FILE_NAME: &str = "spec.did";

const ORBIT_STATION_DID_PATH: &str = "src/orbit_station.rs";
const SPEC_RUST_FILE_HEADER: &str =
    "//! This file is automatically generated by the crate's `build.rs` script.\n\n";

/// Inspired from https://github.com/dfinity/candid/blob/30c388671462aecdc4a3a9753d50dc2e8208c200/tools/didc/src/main.rs#L237-L247
fn bindings_from_did_file(path: &Path) -> String {
    let (env, actor) = pretty_check_file(path).unwrap();
    let configs = Configs::from_str("").unwrap();
    let external = configs
        .get_subtable(&["didc".to_string(), "rust".to_string()])
        .map(|x| x.clone().try_into().unwrap())
        .unwrap_or(ExternalConfig::default());
    let config = Config::new(configs);
    let (res, _) = compile(&config, &env, &actor, external);

    // add Debug derive to all structs
    res.replace(
        "#[derive(CandidType, Deserialize)]",
        "#[derive(CandidType, Deserialize, Debug)]",
    )
}

/// Clone orbit station repository and return the path of the cloned directory
fn clone_orbit_station(parent: &Path) {
    let repo = git2::Repository::clone(ORBIT_STATION_REPO_URL, parent)
        .expect("Failed to clone orbit station repo");

    let obj = repo
        .revparse_single(ORBIT_STATION_TAG_NAME)
        .expect("Failed to find orbit station tag");

    let commit = obj.peel_to_commit().expect("Failed to peel to commit");

    repo.checkout_tree(&obj, None)
        .expect("Failed to checkout tree");

    repo.set_head_detached(commit.id())
        .expect("Failed to set head detached");
}

fn get_current_tag_name(repo_path: &str) -> String {
    let repo = git2::Repository::open(repo_path).expect("Failed to open orbit station repo");
    let head = repo
        .head()
        .expect("Failed to get head")
        .peel_to_commit()
        .expect("Failed to peel to reference");

    let tags = repo.tag_names(None).expect("Failed to get tag names");
    for tag_name in tags.iter().flatten() {
        if let Ok(reference) = repo.revparse_single(tag_name) {
            if let Ok(tag_commit) = reference.peel_to_commit() {
                if tag_commit.id() == head.id() {
                    return tag_name.to_string();
                }
            }
        }
    }

    panic!("No tag found for the current commit");
}

fn sync_repo_with_tag(repo_path: &Path) {
    let current_tag =
        get_current_tag_name(repo_path.to_str().expect("Failed to convert path to str"));
    // get the current tag
    let repo = git2::Repository::open(repo_path).expect("Failed to open orbit station repo");

    if current_tag != ORBIT_STATION_TAG_NAME {
        // checkout the tag
        let obj = repo
            .revparse_single(ORBIT_STATION_TAG_NAME)
            .expect("Failed to find orbit station tag");

        let commit = obj.peel_to_commit().expect("Failed to peel to commit");

        repo.checkout_tree(&obj, None)
            .expect("Failed to checkout tree");

        repo.set_head_detached(commit.id())
            .expect("Failed to set head detached");
    }
}

fn main() {
    // clone orbit station repo
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let orbit_out_dir = PathBuf::from(target_dir).join("orbit");

    // if the directory already exists, sync the repo if the tag is different
    if orbit_out_dir.exists() {
        sync_repo_with_tag(&orbit_out_dir);
    } else {
        // Create the output directory if it doesn't exist
        fs::create_dir_all(orbit_out_dir.parent().unwrap())
            .expect("Failed to create output directory");

        clone_orbit_station(&orbit_out_dir);
    }

    let orbit_station_did = orbit_out_dir
        .join("core/station/api")
        .join(ORBIT_STATION_CANDID_SPEC_FILE_NAME);

    // Get the output path for our generated Rust file
    let orbit_station_did_src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ORBIT_STATION_DID_PATH);

    // Tell Cargo to rerun if the .did file changes
    println!(
        "cargo:rerun-if-changed={}",
        orbit_station_did.to_string_lossy()
    );

    // Run didc command
    let output = bindings_from_did_file(&orbit_station_did);

    // Write the output to our spec file
    let file_content = [SPEC_RUST_FILE_HEADER, &output].concat();
    fs::write(&orbit_station_did_src, file_content).expect("Failed to write generated Rust file");
}
