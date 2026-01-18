use polodb_core::Database;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum SortType {
    #[default]
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
    SizeAsc,
    SizeDesc,
}

use crate::input_settings::InputMap;
use crate::i18n::Language;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectorySettings {
    pub path: String, // Key
    #[serde(default = "default_false")]
    pub spread_view: bool,
    #[serde(default = "default_true")]
    pub right_to_left: bool,
    #[serde(default)]
    pub dir_sort: SortType,
    #[serde(default)]
    pub image_sort: SortType,
}
fn default_false() -> bool { false }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default = "default_key")]
    pub key: String,
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    #[serde(default = "default_false")]
    pub default_spread_view: bool,
    #[serde(default = "default_true")]
    pub default_right_to_left: bool,
    #[serde(default)]
    pub default_dir_sort: SortType,
    #[serde(default)]
    pub default_image_sort: SortType,
    #[serde(default = "default_false")]
    pub loop_images: bool,
    #[serde(default = "default_false")]
    pub single_first_page: bool,
    #[serde(default = "default_true")]
    pub archives_on_top: bool,
    #[serde(default)]
    pub input_map: InputMap,
    #[serde(default)]
    pub language: Language,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppState {
    #[serde(default = "default_key")]
    pub key: String,
    pub last_path: Option<String>,
}

fn default_key() -> String { "global".to_string() }
fn default_true() -> bool { true }

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            key: default_key(),
            dark_mode: true,
            default_spread_view: false,
            default_right_to_left: true,
            default_dir_sort: SortType::NameAsc,
            default_image_sort: SortType::NameAsc,
            loop_images: false,
            single_first_page: false,
            archives_on_top: true,
            input_map: InputMap::default(),
            language: Language::default(),
        }
    }
}

use std::time::Duration;
use std::thread;

pub struct DbHelper {
    path: PathBuf,
}

impl DbHelper {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Test open to fail early if strictly permissions issues, 
        // but for locking issues we want to be lenient.
        // However, existing code expects valid DB on init.
        // Let's just store the path.
        Ok(Self { path })
    }

    fn get_db(&self) -> Result<Database> {
        let mut attempts = 0;
        loop {
            match Database::open_file(&self.path) {
                Ok(db) => return Ok(db),
                Err(e) => {
                    attempts += 1;
                    if attempts >= 5 {
                        return Err(anyhow::anyhow!("Failed to open DB after retries: {}", e));
                    }
                    eprintln!("DB locked, retrying in 100ms... (attempt {}/5)", attempts);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        let db = self.get_db()?;
        let collection = db.collection::<AppSettings>("settings");
        if let Ok(Some(settings)) = collection.find_one(polodb_core::bson::doc! { "key": "global" }) {
             Ok(settings)
        } else {
             // Create default
             let settings = AppSettings::default();
             let _ = collection.insert_one(settings.clone());
             Ok(settings)
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let db = self.get_db()?;
        let collection = db.collection::<AppSettings>("settings");
        let doc = polodb_core::bson::to_document(settings)?;
        let mut update_doc = polodb_core::bson::Document::new();
        update_doc.insert("$set", doc);

        if let Ok(result) = collection.update_one(polodb_core::bson::doc! { "key": "global" }, update_doc) {
            if result.modified_count == 0 {
                // Check if exists
                if collection.find_one(polodb_core::bson::doc! { "key": "global" })?.is_none() {
                    collection.insert_one(settings.clone())?;
                }
            }
        }
        Ok(())
    }

    pub fn get_app_state(&self) -> Result<AppState> {
        let db = self.get_db()?;
        let collection = db.collection::<AppState>("app_state");
        if let Ok(Some(state)) = collection.find_one(polodb_core::bson::doc! { "key": "global" }) {
            Ok(state)
        } else {
            Ok(AppState { key: "global".to_string(), last_path: None })
        }
    }

    pub fn save_app_state(&self, state: &AppState) -> Result<()> {
         let db = self.get_db()?;
         let collection = db.collection::<AppState>("app_state");
         let doc = polodb_core::bson::to_document(state)?;
         let mut update_doc = polodb_core::bson::Document::new();
         update_doc.insert("$set", doc);
         
         if collection.find_one(polodb_core::bson::doc! { "key": "global" })?.is_none() {
             collection.insert_one(state.clone())?;
         } else {
             let _ = collection.update_one(polodb_core::bson::doc! { "key": "global" }, update_doc);
         }
         Ok(())
    }

    pub fn get_directory_settings(&self, path: &str) -> Result<Option<DirectorySettings>> {
        let db = self.get_db()?;
        let collection = db.collection::<DirectorySettings>("directory_settings");
        if let Ok(Some(settings)) = collection.find_one(polodb_core::bson::doc! { "path": path }) {
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }

    pub fn save_directory_settings(&self, settings: &DirectorySettings) -> Result<()> {
        let db = self.get_db()?;
        let collection = db.collection::<DirectorySettings>("directory_settings");
        let doc = polodb_core::bson::to_document(settings)?;
        let mut update_doc = polodb_core::bson::Document::new();
        update_doc.insert("$set", doc);
        
        if collection.find_one(polodb_core::bson::doc! { "path": &settings.path })?.is_none() {
            collection.insert_one(settings.clone())?;
        } else {
            let _ = collection.update_one(polodb_core::bson::doc! { "path": &settings.path }, update_doc);
        }
        Ok(())
    }
}
