use std::env::current_dir;
use std::ffi::OsStr;
use crate::args::CliBuildSubCommand;
use crate::config::{
    StrixBuildConfigProfile, StrixConfig, StrixConfigPackType, StrixConfigProjectType, STRIX_CONFIG,
};
use anyhow::Context;
use log::{error, info, warn};
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use json_comments::StripComments;
use serde_json::Value;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

#[inline(always)]
fn try_rm_prefix(path: &Path) -> PathBuf {
    path.strip_prefix(current_dir().unwrap_or_default())
        .unwrap_or(path)
        .to_path_buf()
}

fn get_mojang_folder() -> PathBuf {
    if let Some(dir) = directories::BaseDirs::new() {
        dir.home_dir()
            .join("AppData")
            .join("Local")
            .join("Packages")
            .join("Microsoft.MinecraftUWP_8wekyb3d8bbwe")
            .join("LocalState")
            .join("games")
            .join("com.mojang")
    } else {
        error!("Couldn't get BaseDirs");
        exit(1);
    }
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
                    error!(
                        "An unexpected Error occurred while trying to create {:?}, Err: {err}",
                        $path
                    );
                    return true;
                }
            }
        };
    };
}

pub async fn build(build: CliBuildSubCommand, config: Option<StrixConfig>) -> bool {
    let config = config.unwrap_or_else(|| {
        warn!(
            "Couldn't find {:?}, it is recommend to create it for enabling more build options",
            STRIX_CONFIG
        );
        StrixConfig::default()
    });

    let profile_name = build
        .profile
        .unwrap_or(config.build.default_profile.clone());

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
        info!("[{profile_name}] minify:     {}", profile.minify);
        info!("[{profile_name}] obfuscate:  {}", profile.obfuscate);
        info!("[{profile_name}] compress:   {}", profile.compress);
        info!("[{profile_name}] encrypt:    {}", profile.encrypt);
        info!("[{profile_name}] dev folder: {}", profile.dev_folder);
        info!("[{profile_name}] package:    {}", profile.package);
    }

    let target_folder = PathBuf::from(config.build.build_path.clone());
    try_make_dir!(target_folder);

    let build_folder = target_folder.join("build");
    try_make_dir!(build_folder);

    let temp_build_folder = build_folder.join(format!("{}", Uuid::new_v4()));
    try_make_dir!(temp_build_folder);

    match config.project_type {
        StrixConfigProjectType::Vanilla => {
            build_vanilla(
                &profile,
                &config,
                &temp_build_folder,
                &target_folder,
                build.quiet,
            )
            .await
        }
        StrixConfigProjectType::Regolith => {
            unimplemented!()
        }
        StrixConfigProjectType::Dash => {
            unimplemented!()
        }
    }
}

async fn build_vanilla(
    profile: &StrixBuildConfigProfile,
    config: &StrixConfig,
    temp_build_folder: &PathBuf,
    target_folder: &PathBuf,
    quiet: bool,
) -> bool {
    let mut project_paths = vec![];

    for (project, project_type) in &config.projects {
        let project_path = temp_build_folder.join(&project);
        try_make_dir!(project_path);

        project_paths.push((
            WalkDir::new(project_path.clone()),
            Path::new(project),
            Path::new(temp_build_folder),
        ));

        match copy_dir_all(&project, &project_path) {
            Ok(_) => {}
            Err(err) => {
                error!("An unexpected Error occurred while trying to copy {project:?} to {project_path:?}, Err: {err}");
                return true;
            }
        }

        let walk: Vec<_> = WalkDir::new(&project_path)
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
            let mut text = match fs::read_to_string(entry.path()) {
                Ok(v) => v,
                Err(_) => {
                    continue
                }
            };

            if let Some(ext) = entry.path().extension().and_then(OsStr::to_str) {
                if profile.minify {
                    match ext {
                        "json" => {
                            let stripped = StripComments::new(text.as_bytes());
                            let json: serde_json::error::Result<Value> = serde_json::from_reader(stripped);

                            match json {
                                Ok(json) => {
                                    match serde_json::to_string(&json)  {
                                        Ok(v) => {
                                            text = v
                                        },
                                        Err(err) => {
                                            error!(
                                                "An unexpected Error occurred while trying to serialize {:?}\n{}",
                                                try_rm_prefix(entry.path()).display(),
                                                err
                                            );
                                        }
                                    };
                                }
                                Err(err) => {
                                    error!(
                                        "An unexpected Error occurred while trying to deserialize {:?}\n{}",
                                        try_rm_prefix(entry.path()).display(),
                                        err
                                    );
                                }
                            };
                        }
                        _ => {}
                    }
                }
            }

            /* DO SOME PROCESSING LIKE MINIFICATION, COMPRESSION, ENCRYPTION, OBFUSCATION */

            match fs::write(entry.path(), text) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "An unexpected Error occurred while trying to write {:?}\n{}",
                        try_rm_prefix(entry.path()).display(),
                        err
                    );
                }
            };
        }

        if profile.dev_folder {
            let mojang_folder = get_mojang_folder();

            match project_type {
                StrixConfigPackType::Behaviour => {
                    let path = mojang_folder
                        .join("development_behavior_packs")
                        .join(project);
                    try_make_dir!(path);

                    match copy_dir_all(&project_path, &path) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("An unexpected Error occurred while trying to copy {project:?} to {project_path:?}, Err: {err}");
                            return true;
                        }
                    }
                }
                StrixConfigPackType::Resource => {
                    let path = mojang_folder
                        .join("development_resource_packs")
                        .join(project);
                    try_make_dir!(path);

                    match copy_dir_all(&project_path, &path) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("An unexpected Error occurred while trying to copy {project:?} to {project_path:?}, Err: {err}");
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if profile.package {
        if let Err(err) = zip_dir(
            project_paths,
            &target_folder.join(format!("{}.mcaddon", config.name)),
            quiet,
        ) {
            error!(
                "An unexpected Error occurred while trying to zip {:?}, Err: {err}",
                config.name
            );
            return true;
        }
    }

    false
}

fn zip_dir(it: Vec<(WalkDir, &Path, &Path)>, path: &Path, quiet: bool) -> anyhow::Result<()> {
    let file = File::create(path)?;

    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();

    for (walk, parent, prefix) in it {
        let walk = walk.into_iter();
        let walk = walk.filter_map(|e| e.ok());

        for entry in walk {
            let path = entry.path();
            let name = parent.join(path.strip_prefix(prefix.join(parent))?);
            let path_as_string = name
                .to_str()
                .map(str::to_owned)
                .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

            if path.is_file() {
                if !quiet {
                    println!("Zipping file {path:?}");
                }

                zip.start_file(path_as_string, options)?;
                let mut f = File::open(path)?;

                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(path_as_string, options)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}
