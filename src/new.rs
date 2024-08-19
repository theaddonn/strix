use crate::args::CliNewSubCommand;
use crate::config::{StrixConfig, StrixConfigPackType, StrixConfigProjectType, STRIX_CONFIG};
use dialoguer::{Input, MultiSelect, Select};
use log::error;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn get_text(name: &'static str) -> Result<String, String> {
    Input::new()
        .with_prompt(format!("Addon {name}"))
        .interact()
        .map_err(|err| err.to_string())
}

pub async fn new(new: CliNewSubCommand) -> bool {
    let name = match get_text("Name") {
        Ok(v) => v,
        Err(err) => {
            error!("An unexpected Error occurred while trying to prompt for the Addon Name, Err: {err}");
            return true;
        }
    };

    let description = match get_text("Description") {
        Ok(v) => v,
        Err(err) => {
            error!("An unexpected Error occurred while trying to prompt for the Addon Description, Err: {err}");
            return true;
        }
    };

    let select = Select::new()
        .with_prompt(format!("Select an Addon Generator for {name:?}"))
        .items(&["Vanilla", "Regolith", "Dash"])
        .report(true)
        .default(0);

    if let Some(path) = &new.path {
        if !path.exists() || !path.is_dir() {
            match fs::create_dir(&path) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "An unexpected Error occurred while trying to create {:?}, Err: {err}",
                        path
                    );
                    return true;
                }
            }
        }
    }

    let path = new.path.unwrap_or_default();

    let mut config = StrixConfig {
        name,
        description,
        ..Default::default()
    };

    let error_out = match select.interact() {
        Ok(0) => {
            config.project_type = StrixConfigProjectType::Vanilla;
            new_vanilla(&mut config, path.clone())
        }
        Ok(1) => {
            config.project_type = StrixConfigProjectType::Regolith;
            unimplemented!()
        }
        Ok(2) => {
            config.project_type = StrixConfigProjectType::Dash;
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
    };

    match fs::write(
        path.join(&STRIX_CONFIG),
        serde_json::to_string_pretty(&config).unwrap(),
    ) {
        Ok(_) => {}
        Err(err) => {
            error!(
                "An unexpected Error occurred while trying to create {:?}, Err: {err}",
                path.join(".strix")
            );
            return true;
        }
    };

    error_out
}

fn new_vanilla(config: &mut StrixConfig, path: PathBuf) -> bool {
    let select = MultiSelect::new()
        .with_prompt(format!("Select the Packs for {:?}", config.name))
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
        let addon_name = format!("{}BP", config.name);
        config
            .projects
            .insert(addon_name.clone(), StrixConfigPackType::Behaviour);

        let addon_path = &path.join(addon_name);

        let json = serde_json::to_string_pretty(&json!({
            "format_version": 2,
            "header": {
                "name": config.name,
                "description": config.description,
                "uuid": Uuid::new_v4(),
                "version": [ 1, 0, 0 ],
                "min_engine_version": [ 1, 16, 0 ]
            },
            "modules": [
                {
                    "type": "data",
                    "description": config.description,
                    "uuid": Uuid::new_v4(),
                    "version": [ 1, 0, 0 ],
                }
            ]
        }))
        .unwrap_or_default();

        if !addon_path.exists() {
            if let Err(err) = fs::create_dir(addon_path) {
                error!(
                    "An unexpected Error occurred while trying to write {:?}, Err: {err}",
                    addon_path.display()
                );
                return true;
            }
        }

        if let Err(err) = fs::write(addon_path.join("manifest.json"), json) {
            error!(
                "An unexpected Error occurred while trying to write {:?}, Err: {err}",
                addon_path.join("manifest.json").display()
            );
            return true;
        };
    }

    // Resource Pack
    if selected.contains(&1) {
        let addon_name = format!("{}RP", config.name);
        config
            .projects
            .insert(addon_name.clone(), StrixConfigPackType::Resource);

        let addon_path = &path.join(addon_name);

        let json = serde_json::to_string_pretty(&json!({
            "format_version": 2,
            "header": {
                "name": config.name,
                "description": config.description,
                "uuid": Uuid::new_v4(),
                "version": [ 1, 0, 0 ],
                "min_engine_version": [ 1, 16, 0 ]
            },
            "modules": [
                {
                    "type": "resource",
                    "description": config.description,
                    "uuid": Uuid::new_v4(),
                    "version": [ 1, 0, 0 ],
                }
            ]
        }))
        .unwrap_or_default();

        if !addon_path.exists() {
            if let Err(err) = fs::create_dir(addon_path) {
                error!(
                    "An unexpected Error occurred while trying to write {:?}, Err: {err}",
                    addon_path.display()
                );
                return true;
            }
        }

        if let Err(err) = fs::write(addon_path.join("manifest.json"), json) {
            error!(
                "An unexpected Error occurred while trying to write {:?}, Err: {err}",
                addon_path.join("manifest.json").display()
            );
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
