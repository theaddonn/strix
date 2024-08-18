use std::fs;
use std::path::{Path, PathBuf};
use crate::args::CliBuildSubCommand;
use crate::config::{StrixBuildConfigProfile, StrixConfig, StrixConfigPackType, StrixConfigProjectType, STRIX_CONFIG};
use log::{error, info, warn};
use uuid::Uuid;
use std::io;
use std::io::{Seek, Write};
use std::process::exit;
use walkdir::{DirEntry, WalkDir};
use zip::write::SimpleFileOptions;

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

    let profile_name = build.profile.unwrap_or(config.build.default_profile.clone());

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
        StrixConfigProjectType::Vanilla => { build_vanilla(&profile, &config, &temp_build_folder) }
        StrixConfigProjectType::Regolith => { unimplemented!() }
        StrixConfigProjectType::Dash => { unimplemented!() }
    }
}

async fn build_vanilla(profile: &StrixBuildConfigProfile, config: &StrixConfig, temp_build_folder: &PathBuf) -> bool {
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
            /* DO SOME PROCESSING LIKE MINIFICATION, COMPRESSION, ENCRYPTION, OBFUSCATION */

        }

        if profile.dev_folder {
            let mojang_folder = get_mojang_folder();

            match project_type {
                StrixConfigPackType::Behaviour => {
                    let path = mojang_folder.join("development_behavior_packs").join(project);
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
                    let path = mojang_folder.join("development_resource_packs").join(project);
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

    false
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> Result<(), String>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
