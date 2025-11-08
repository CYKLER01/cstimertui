use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
directories::ProjectDirs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Solve {
    pub time: u128,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Solves {
    pub solves: Vec<Solve>,
}

fn get_solves_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "cykler", "rustcstimer") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).ok()?;
        }
        Some(config_dir.join("solves.json"))
    } else {
        None
    }
}

pub fn load_solves() -> Solves {
    if let Some(solves_path) = get_solves_path() {
        if let Ok(solves_str) = fs::read_to_string(&solves_path) {
            if let Ok(solves) = serde_json::from_str(&solves_str) {
                return solves;
            }
        }
        let solves = Solves::default();
        if let Ok(solves_str) = serde_json::to_string_pretty(&solves) {
            fs::write(solves_path, solves_str).ok();
        }
        solves
    } else {
        Solves::default()
    }
}

pub fn add_solve(solve: Solve) -> std::io::Result<()> {
    if let Some(solves_path) = get_solves_path() {
        let mut solves = load_solves();
        solves.solves.push(solve);
        let solves_str = serde_json::to_string_pretty(&solves)?;
        fs::write(solves_path, solves_str)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find config directory",
        ))
    }
}
