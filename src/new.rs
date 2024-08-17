use crate::args::CliNewSubCommand;
use dialoguer::{MultiSelect, Select};
use log::error;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn new(new: CliNewSubCommand) -> bool {
    let name = new.name;

    let select = Select::new()
        .with_prompt(format!("Select an Addon Generator for {name:?}"))
        .items(&["Vanilla", "Regolith", "Dash"])
        .report(true)
        .default(0);

    let path = new.path.unwrap_or_default();

    if !path.exists() || !path.is_dir() {
        match fs::create_dir(&path) {
            Ok(_) => {}
            Err(err) => {
                error!("An unexpected Error occurred while trying to create {:?}, Err: {err}", path);
                return true;
            }
        }
    }

    match select.interact() {
        Ok(0) => new_vanilla(name, path),
        Ok(1) => {
            unimplemented!()
        }
        Ok(2) => {
            unimplemented!()
        }
        Ok(other) => {
            error!("An unexpected Error occurred while trying to prompt for the Addon Generator, Err: Unexpected index {other}");
            true
        }
        Err(err) => {
            error!("An unexpected Error occurred while trying to prompt for the Addon Generator, Err: {err}");
            true
        }
    }
}

fn new_vanilla(name: String, path: PathBuf) -> bool {
    let select = MultiSelect::new()
        .with_prompt(format!("Select the packs for {name:?}"))
        .items(&[
            "Behaviour Pack",
            "Resource Pack",
            "World Template",
            "Skin Pack",
        ])
        .report(true);

    let selected = match select.interact() {
        Ok(selected) => selected,
        Err(err) => {
            error!("An unexpected Error occurred while trying to prompt for the Addon Packs, Err: {err}");
            return true;
        }
    };

    // Behaviour Pack
    if selected.contains(&0) {
        let addon_path = &path.join(format!("{name}BP"));

        let json = serde_json::to_string_pretty(&json!({
            "format_version": 2,
            "header": {
                "name": name,
                "description": "",
                "uuid": Uuid::new_v4(),
                "version": [ 1, 0, 0 ],
                "min_engine_version": [ 1, 16, 0 ]
            },
            "modules": [
                {
                    "type": "data",
                    "description": "",
                    "uuid": Uuid::new_v4(),
                    "version": [ 1, 0, 0 ],
                }
            ]
        }))
        .unwrap_or_default();

        if !addon_path.exists() {
            if let Err(err) = fs::create_dir(&addon_path) {
                error!("An unexpected Error occurred while trying to write {:?}, Err: {err}", addon_path.display());
                return true;
            }
        }

        if let Err(err) = fs::write(addon_path.join("manifest.json"), json) {
            error!("An unexpected Error occurred while trying to write {:?}, Err: {err}", addon_path.join("manifest.json").display());
            return true;
        };
    }

    // Resource Pack
    if selected.contains(&1) {
        let addon_path = &path.join(format!("{name}RP"));

        let json = serde_json::to_string_pretty(&json!({
            "format_version": 2,
            "header": {
                "name": name,
                "description": "",
                "uuid": Uuid::new_v4(),
                "version": [ 1, 0, 0 ],
                "min_engine_version": [ 1, 16, 0 ]
            },
            "modules": [
                {
                    "type": "resource",
                    "description": "",
                    "uuid": Uuid::new_v4(),
                    "version": [ 1, 0, 0 ],
                }
            ]
        }))
        .unwrap_or_default();

        if !addon_path.exists() {
            if let Err(err) = fs::create_dir(&addon_path) {
                error!("An unexpected Error occurred while trying to write {} the tokio runtime, Err: {err}", addon_path.display());
                return true;
            }
        }

        if let Err(err) = fs::write(addon_path.join("manifest.json"), json) {
            error!("An unexpected Error occurred while trying to write {} the tokio runtime, Err: {err}", addon_path.join("manifest.json").display());
            return true;
        };
    }

    // World Template
    if selected.contains(&2) {
        unimplemented!()
    }


    // Skin Pack
    if selected.contains(&3) {
        unimplemented!()
    }

    false
}
