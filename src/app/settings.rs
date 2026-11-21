//! What the app remembers between runs: the look, and every CHOpt setting (CHOpt itself keeps
//! squeeze, early whammy, lazy whammy, whammy delay, video lag and lefty flip).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    /// Frosted glass behind the window (Windows; takes effect on restart).
    pub frosted: bool,
    /// How see-through the glass is, 0.1-1.0.
    pub transparency: f32,
    pub squeeze: u32,
    pub early_whammy: u32,
    pub lazy_whammy_ms: i32,
    pub whammy_delay_ms: i32,
    pub video_lag_ms: i32,
    pub speed: i32,
    pub activation_opacity: u32,
    pub draw_bpms: bool,
    pub draw_time_sigs: bool,
    pub draw_solos: bool,
    pub blank: bool,
    pub precision: bool,
    pub lefty: bool,
    /// The folder charts were last loaded from.
    pub last_folder: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "Rust".into(),
            frosted: cfg!(windows),
            transparency: 0.5,
            squeeze: 100,
            early_whammy: 100,
            lazy_whammy_ms: 0,
            whammy_delay_ms: 0,
            video_lag_ms: 0,
            speed: 100,
            activation_opacity: 33,
            draw_bpms: true,
            draw_time_sigs: true,
            draw_solos: true,
            blank: false,
            precision: false,
            lefty: false,
            last_folder: None,
        }
    }
}

fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "CHOpt Truss").map(|d| d.config_dir().join("settings.json"))
}

impl Settings {
    pub fn load() -> Self {
        path().and_then(|p| std::fs::read(p).ok()).and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_vec_pretty(self) {
            // Written beside and renamed over, so a crash mid-write never leaves half a file.
            let tmp = p.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
        }
    }
}
