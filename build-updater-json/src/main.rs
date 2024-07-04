// see https://tauri.app/v1/guides/distribution/updater/ for json format

use chrono::{Timelike, Utc};
use indexmap::IndexMap;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct UpdaterJson {
    version: String,
    notes: String,
    pub_date: chrono::DateTime<Utc>,
    platforms: IndexMap<String, Platform>,
}

#[derive(Serialize)]
struct Platform {
    signature: String,
    url: String,
}

fn main() {
    // consts
    const DOWNLOAD_URL_BASE: &str =
        "https://github.com/vrc-get/vrc-get/releases/download/gui-v{version}";
    let platform_file_name = [
        ("darwin-x86_64", "ALCOM-{version}-universal.app.tar.gz"),
        ("darwin-aarch64", "ALCOM-{version}-universal.app.tar.gz"),
        ("linux-x86_64", "alcom-{version}-x86_64.AppImage.tar.gz"),
        //("linux-aarch64", "alcom-{version}-aarch64.AppImage.tar.gz"),
        ("windows-x86_64", "ALCOM-{version}-x86_64-setup.nsis.zip"),
        //("windows-aarch64", "ALCOM-{version}-aarch64-setup.nsis.zip"),
    ]
    .into_iter()
    .collect::<IndexMap<_, _>>();

    let version = std::env::var("GUI_VERSION").expect("GUI_VERSION not set");

    let base_url = DOWNLOAD_URL_BASE.replace("{version}", &version);

    // create platforms info
    let mut platforms = IndexMap::new();
    for (platform, file_name) in platform_file_name {
        let file_name = file_name.replace("{version}", &version);

        std::fs::metadata(format!("assets/{file_name}"))
            .unwrap_or_else(|e| panic!("{}: {}", file_name, e));

        let signature = std::fs::read_to_string(format!("assets/{file_name}.sig"))
            .unwrap_or_else(|e| panic!("{}.sig: {}", file_name, e));

        let url = format!("{}/{}", base_url, file_name);
        platforms.insert(platform.to_string(), Platform { signature, url });
    }

    let stable_notes = get_notes("vrc-get-gui/notes.txt".as_ref());
    let beta_notes = get_notes("vrc-get-gui/notes-beta.txt".as_ref());
    let is_beta = version.contains('-');

    let mut updater = UpdaterJson {
        version,
        notes: String::new(),
        pub_date: Utc::now().with_nanosecond(0).unwrap(),
        platforms,
    };

    if !is_beta {
        updater.notes = stable_notes;
        write_json("updater.json", &updater);
    }
    updater.notes = beta_notes;
    write_json("updater-beta.json", &updater);
}

fn write_json(path: impl AsRef<Path>, json: impl Serialize) {
    let json = serde_json::to_string_pretty(&json).unwrap();
    std::fs::write(path, json).expect("write updater.json");
}

fn get_notes(path: &Path) -> String {
    let notes = std::fs::read_to_string(path).expect("read notes.txt");
    // lines starts with # are comments
    notes
        .trim_end()
        .lines()
        .filter(|x| !x.starts_with("#"))
        .map(|x| x.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}
