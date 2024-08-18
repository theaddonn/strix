use std::env::temp_dir;
use std::fs;
use std::path::{Path, PathBuf};
use crate::args::CliBuildSubCommand;
use crate::config::{StrixBuildConfigProfile, StrixConfig, StrixConfigPackType, StrixConfigProjectType, STRIX_CONFIG};
use log::{error, info, warn};
use uuid::Uuid;
use std::io;
use walkdir::WalkDir;

fn get_mojang_folder() -> PathBuf {
    
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

macro_rules! try_make_dir {
    ($path:path) => {
        if !$path.is_dir() || !$path.exists() {
            match fs::create_dir(&$path) {
                Ok(_) => {}
                Err(err) => {
                    error!("An unexpected Error occurred while trying to create {:?}, Err: {err}", $path);
                    return true;
                }
            }
        };
    }
}

pub async fn build(build: CliBuildSubCommand, config: Option<StrixConfig>) -> bool {
    let config = config.unwrap_or_else(|| {
        warn!(
            "Couldn't find {:?}, it is recommend to create it for enabling more build options",
            STRIX_CONFIG
        );
        StrixConfig::default()
    });

    let profile_name = build.profile.unwrap_or(config.build.default_profile);

    let profile = match config.build.profiles.get(&profile_name) {
        None => {
            error!(
                "Couldn't find build profile {:?}, in {:?}",
                profile_name,
                config.build.profiles.keys()
            );
            return true;
        }
        Some(v) => v,
    };

    if !build.quiet {
        info!("Starting build on Profile {:?}", profile_name);
        info!("[{profile_name}] minify: {}", profile.minify);
        info!("[{profile_name}] compress: {}", profile.compress);
        info!("[{profile_name}] encrypt: {}", profile.encrypt);
    }

    let target_folder = PathBuf::from(config.build.build_path);
    try_make_dir!(target_folder);

    let build_folder = target_folder.join("build");
    try_make_dir!(build_folder);

    let temp_build_folder = build_folder.join(format!("{}", Uuid::new_v4()));
    try_make_dir!(temp_build_folder);

    match config.project_type {
        StrixConfigProjectType::Vanilla => { build_vanilla(&profile, &config, &temp_build_folder) }
        StrixConfigProjectType::Regolith => { unimplemented!() }
        StrixConfigProjectType::Dash => { unimplemented!() }
    }
}

fn build_vanilla(profile: &StrixBuildConfigProfile, config: &StrixConfig, temp_build_folder: &PathBuf) -> bool {
    for (project, project_type) in &config.projects {
        let project_path = temp_build_folder.join(&project);
        try_make_dir!(project_path);

        match copy_dir_all(&project, &project_path) {
            Ok(_) => {}
            Err(err) => {
                error!("An unexpected Error occurred while trying to copy {project:?} to {project_path:?}, Err: {err}");
                return true;
            }
        }

        let walk: Vec<_> = WalkDir::new(project_path)
            .into_iter()
            .filter(|v| {
                if let Ok(v) = v {
                    v.file_type().is_file()
                } else {
                    false
                }
            })
            .collect();

        for entry in walk.into_iter().flatten() {
            /* DO SOME PROCESSING LIKE MINIFICATION, COMPRESSION, ENCRYPTION */
        }

        if profile.mojang_dev_folder {
            let mojang_folder = get_mojang_folder();

            match project_type {
                StrixConfigPackType::Behaviour => match copy_dir_all(mojang_folder.join("development_behavior_packs"), &project_path) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("An unexpected Error occurred while trying to copy {project:?} to {project_path:?}, Err: {err}");
                        return true;
                    }
                }
                StrixConfigPackType::Resource => match copy_dir_all(mojang_folder.join("development_resource_packs"), &project_path) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("An unexpected Error occurred while trying to copy {project:?} to {project_path:?}, Err: {err}");
                        return true;
                    }
                }
                _ => {}
            }
        }
    }

    false
}
