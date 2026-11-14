// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 binoema

// dewz als Desktop-App: ein Webview-Fenster mit src/index.html plus wenige, eng gefasste Befehle für den
// Dateizugriff. Die Oberfläche bekommt keinen allgemeinen Dateisystem-Zugriff:
//   * Sie kann nur Ordner lesen, die der Nutzer im Ordnerdialog gewählt hat (oder der gemerkte Ordner),
//   * schreibt ausschließlich die Datendatei DATA_NAME in den aktivierten Ordner,
//   * und Exporte nur an den Ort, den der Nutzer im „Speichern unter“-Dialog bestimmt.
// Verschlüsselung und Passwort bleiben komplett in der Oberfläche; hier kommt nur fertiger Text an.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

const DATA_NAME: &str = "dewz-daten.json";
const LEGACY_DATA_NAME: &str = "finanz-buddy-daten.json"; // Dateiname vor der Umbenennung der App
const SETTINGS_NAME: &str = "einstellungen.json";

#[derive(Default)]
struct Folders {
    allowed: HashSet<PathBuf>, // im Dialog gewählt oder gemerkt
    active: Option<PathBuf>,   // hierhin schreibt write_data
}
type FolderState = Mutex<Folders>;

/// Sprache der Oberfläche (true = Englisch), damit Dialogtitel und Fehlermeldungen zur Oberfläche passen.
#[derive(Default)]
struct English(AtomicBool);
type Lang<'a> = State<'a, English>;

/// Wählt den Text in der Sprache der Oberfläche.
fn tr(lang: &Lang, de: &str, en: &str) -> String {
    if lang.0.load(Ordering::Relaxed) { en } else { de }.to_owned()
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Settings {
    data_folder: Option<PathBuf>,
}

#[derive(Serialize)]
struct DataFile {
    text: String,
    legacy: bool,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Nur im Debug-Build: eigener Einstellungsordner, z. B. für Screenshots mit Beispieldaten, ohne dass
    // der echte, gemerkte Datenordner geöffnet wird. Im Release-Build gibt es diesen Schalter nicht.
    #[cfg(debug_assertions)]
    if let Some(dir) = std::env::var_os("DEWZ_SETTINGS_DIR") {
        return Ok(PathBuf::from(dir).join(SETTINGS_NAME));
    }
    app.path().app_config_dir().map(|d| d.join(SETTINGS_NAME)).map_err(|e| e.to_string())
}

fn load_settings(app: &AppHandle) -> Settings {
    settings_path(app)
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn path_string(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn allowed_dir(state: &State<FolderState>, lang: &Lang, dir: &str) -> Result<PathBuf, String> {
    let dir = PathBuf::from(dir);
    if !state.lock().unwrap().allowed.contains(&dir) {
        return Err(tr(lang, "Dieser Ordner wurde nicht im Dialog ausgewählt.", "This folder was not chosen in the dialog."));
    }
    if !dir.is_dir() {
        let p = path_string(&dir);
        return Err(tr(lang, &format!("Der Ordner „{p}“ existiert nicht (mehr)."), &format!("The folder “{p}” does not exist (any more).")));
    }
    Ok(dir)
}

/// Schreibt erst eine Hilfsdatei und benennt sie dann um – ein Absturz mitten im Schreiben
/// hinterlässt so nie eine halbe Datendatei.
fn write_atomic(target: &Path, content: &str) -> std::io::Result<()> {
    let mut tmp = target.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, target).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// Stellt die Sprache für Dialoge und Fehlermeldungen um ("de" oder "en").
#[tauri::command]
fn set_language(lang: String, state: Lang) {
    state.0.store(lang == "en", Ordering::Relaxed);
}

/// Der beim letzten Mal verwendete Datenordner, falls einer gemerkt ist.
#[tauri::command]
fn remembered_folder(app: AppHandle, state: State<FolderState>) -> Option<String> {
    let dir = load_settings(&app).data_folder?;
    let s = path_string(&dir);
    state.lock().unwrap().allowed.insert(dir);
    Some(s)
}

/// Ordnerdialog des Systems. Liefert null, wenn der Nutzer abbricht.
#[tauri::command]
async fn choose_folder(app: AppHandle, state: State<'_, FolderState>, lang: Lang<'_>) -> Result<Option<String>, String> {
    let title = tr(&lang, "Datenordner wählen", "Choose data folder");
    let Some(picked) = app.dialog().file().set_title(title).blocking_pick_folder() else {
        return Ok(None);
    };
    let dir = picked.into_path().map_err(|e| e.to_string())?;
    let s = path_string(&dir);
    state.lock().unwrap().allowed.insert(dir);
    Ok(Some(s))
}

/// Liest die Datendatei aus dem Ordner (oder die Datei unter dem früheren Namen); null, wenn keine da ist.
#[tauri::command]
fn read_data(dir: String, state: State<FolderState>, lang: Lang) -> Result<Option<DataFile>, String> {
    let dir = allowed_dir(&state, &lang, &dir)?;
    for (name, legacy) in [(DATA_NAME, false), (LEGACY_DATA_NAME, true)] {
        match fs::read_to_string(dir.join(name)) {
            Ok(text) => return Ok(Some(DataFile { text, legacy })),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(tr(&lang, &format!("{name} konnte nicht gelesen werden: {e}"), &format!("{name} could not be read: {e}")))
            }
        }
    }
    Ok(None)
}

/// Macht den Ordner zum Ziel von write_data und merkt ihn für den nächsten Start.
#[tauri::command]
fn activate_folder(dir: String, app: AppHandle, state: State<FolderState>, lang: Lang) -> Result<(), String> {
    let dir = allowed_dir(&state, &lang, &dir)?;
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let settings = Settings { data_folder: Some(dir.clone()) };
    write_atomic(&path, &serde_json::to_string_pretty(&settings).unwrap()).map_err(|e| e.to_string())?;
    state.lock().unwrap().active = Some(dir);
    Ok(())
}

/// Schreibt den (ggf. schon verschlüsselten) Inhalt in die Datendatei des aktiven Ordners.
#[tauri::command]
fn write_data(content: String, state: State<FolderState>, lang: Lang) -> Result<(), String> {
    let active = state.lock().unwrap().active.clone();
    let dir = active.ok_or_else(|| tr(&lang, "Es ist kein Datenordner geöffnet.", "No data folder is open."))?;
    write_atomic(&dir.join(DATA_NAME), &content).map_err(|e| e.to_string())
}

/// „Speichern unter“-Dialog für Exporte. Liefert den gewählten Pfad oder null bei Abbruch.
#[tauri::command]
async fn save_export(name: String, content: String, app: AppHandle, lang: Lang<'_>) -> Result<Option<String>, String> {
    let ext = Path::new(&name).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let label = match ext.as_str() {
        "csv" => tr(&lang, "CSV-Datei", "CSV file"),
        "json" => tr(&lang, "JSON-Datei", "JSON file"),
        _ => return Err(tr(&lang, &format!("Unbekannter Dateityp: {name}"), &format!("Unknown file type: {name}"))),
    };
    let Some(picked) = app
        .dialog()
        .file()
        .set_title(tr(&lang, "Exportieren", "Export"))
        .set_file_name(&name)
        .add_filter(&label, &[ext.as_str()])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = picked.into_path().map_err(|e| e.to_string())?;
    write_atomic(&path, &content).map_err(|e| e.to_string())?;
    Ok(Some(path_string(&path)))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(FolderState::default())
        .manage(English::default())
        .invoke_handler(tauri::generate_handler![
            set_language,
            remembered_folder,
            choose_folder,
            read_data,
            activate_folder,
            write_data,
            save_export
        ])
        .run(tauri::generate_context!())
        .expect("dewz konnte nicht gestartet werden");
}
