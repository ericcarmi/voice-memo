use rusqlite::{named_params, Connection};
use std::fs;
use tauri::AppHandle;

#[derive(Debug)]
struct Preferences {
    id: i32,
    darkmode: bool,
    accent: String,
    always_on_top: bool,
}

const CURRENT_DB_VERSION: u32 = 1;
const DB_PATH: &str = "assets/cv_voicecontrol_database.sqlite";

/// Initializes the database connection, creating the .sqlite file if needed, and upgrading the database
/// if it's out of date.
pub fn initialize_database(app_handle: &AppHandle) -> Result<Connection, rusqlite::Error> {
    let sqlite_path = app_handle
        .path_resolver()
        .resolve_resource(DB_PATH)
        .expect("The app data directory should exist.");

    let mut db = Connection::open(sqlite_path)?;

    // let mut user_pragma = db.prepare("PRAGMA user_version")?;
    // let existing_user_version: u32 = user_pragma.query_row([], |row| Ok(row.get(0)?))?;
    // drop(user_pragma);

    // upgrade_database_if_needed(&mut db, existing_user_version)?;

    Ok(db)
}

/// Upgrades the database to the current version.
pub fn upgrade_database_if_needed(
    db: &mut Connection,
    existing_version: u32,
) -> Result<(), rusqlite::Error> {
    if existing_version < CURRENT_DB_VERSION {
        db.pragma_update(None, "journal_mode", "WAL")?;

        let tx = db.transaction()?;

        //   tx.pragma_update(None, "user_version", CURRENT_DB_VERSION)?;

        //   tx.execute_batch(
        //       "
        // CREATE TABLE items (
        //   title TEXT NOT NULL
        // );",
        //   )?;

        // tx.commit()?;
    }

    Ok(())
}

pub fn set_dark_mode(dark_mode: bool, app_handle: &AppHandle) -> Result<bool, rusqlite::Error> {
    let sqlite_path = app_handle
        .path_resolver()
        .resolve_resource(DB_PATH)
        .expect("The app data directory should exist.");

    let mut db = Connection::open(sqlite_path)?;

    let r = db.execute("UPDATE prefs set darkmode=(?2)", (0, dark_mode))?;
    Ok(true)
}

pub fn set_accent(accent_color: String, app_handle: &AppHandle) -> Result<bool, rusqlite::Error> {
    let sqlite_path = app_handle
        .path_resolver()
        .resolve_resource(DB_PATH)
        .expect("The app data directory should exist.");

    let mut db = Connection::open(sqlite_path)?;

    let r = db.execute("UPDATE prefs set accent=(?2)", (0, accent_color))?;

    Ok(true)
}

pub fn set_on_top(on_top: bool, app_handle: &AppHandle) -> Result<bool, rusqlite::Error> {
    let sqlite_path = app_handle
        .path_resolver()
        .resolve_resource(DB_PATH)
        .expect("The app data directory should exist.");

    let mut db = Connection::open(sqlite_path)?;

    let r = db.execute("UPDATE prefs set alwaysOnTop=(?2)", (0, on_top))?;

    Ok(true)
}

pub fn get_all(app_handle: &AppHandle) -> Result<Vec<String>, rusqlite::Error> {
    let sqlite_path = app_handle
        .path_resolver()
        .resolve_resource(DB_PATH)
        .expect("The app data directory should exist.");

    let mut db = Connection::open(sqlite_path)?;

    let mut statement = db.prepare("SELECT * FROM prefs")?;

    let mut items = Vec::new();

    let prefs_iter = statement.query_map([], |row| {
        Ok(Preferences {
            id: row.get(0)?,
            darkmode: row.get(1)?,
            accent: row.get(2)?,
            always_on_top: row.get(3)?,
        })
    })?;

    for pref in prefs_iter {
        if pref.is_ok() {
            let p = pref.unwrap();
            items.push(p.darkmode.to_string());
            items.push(p.accent);
            items.push(p.always_on_top.to_string());
        }
    }

    Ok(items)
}
