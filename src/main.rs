#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use ini::Ini;
use rfd::FileDialog;
use slint::{ComponentHandle, Image, ModelRc, SharedString, VecModel, Weak};
use std::{
    cell::RefCell,
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
};

mod extract;
mod file_manager;
mod profile_manager;

fn main() -> Result<(), Box<dyn Error>> {
    let window = AppWindow::new()?;
    let ui = Rc::new(window);

    let ui_handle = ui.as_weak();
    let overextract = true;

    println!("Hello, world!");

    // let archive_path = Rc::new(RefCell::new(PathBuf::new()));

    {
        // let archive_path = archive_path.clone();
        // let extract_to = extract_to.clone();
        let ui_copy = Rc::clone(&ui);

        ui.on_request_archive_path(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                if let Some(path_str) = path.to_str() {
                    ui_copy.set_archive_path(SharedString::from(path_str));
                }
                // *archive_path.borrow_mut() = path;
            }
        });
    }

    {
        let ui_copy = Rc::clone(&ui);

        ui.on_request_game_path(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                if let Some(path_str) = path.to_str() {
                    ui_copy.set_game_path(SharedString::from(path_str));
                }
            }
        });
    }

    {
        let ui_copy = Rc::clone(&ui);
        ui.on_mod(move || {
            let path = PathBuf::from(ui_copy.get_archive_path().to_string());
            let game_path = PathBuf::from(ui_copy.get_game_path().to_string());
            let extract_to = match &game_path.file_name() {
                Some(name) => format!(".temp/{}/", name.to_string_lossy()),
                None => String::from(".temp/Unknown/"),
            };
            let overwrite = ui_copy.get_overwrite();
            let symlink = ui_copy.get_symlink();
            let exts = ["zip", "rar", "7z"];

            let _ = fs::remove_dir_all(match env::current_dir() {
                Ok(path) => {
                    if !path.join(&extract_to).exists() {
                        path.join(&extract_to)
                    } else {
                        PathBuf::new()
                    }
                }
                Err(e) => {
                    println!("Failed to get current directory: {}", e);
                    PathBuf::new()
                }
            });

            println!("Path: '{}'", path.display());
            ui_copy.set_progress(0.1);
            if path.exists() {
                for entry in fs::read_dir(&*path).unwrap() {
                    let entry = entry.unwrap();
                    println!("Entry: {}", entry.path().display());
                    let path = entry.path();

                    if let Some(extension) = path.extension() {
                        if exts.contains(&extension.to_str().unwrap()) {
                            let _result = match extract::extract_file(
                                &path.to_str().unwrap().to_string(),
                                &extract_to,
                            ) {
                                Ok(_) => {
                                    if let Some(ui) = ui_handle.upgrade() {
                                        println!("Extracted files correctly");
                                        ui.set_footer(SharedString::from(
                                            "Succesfully extracted files",
                                        ));
                                        ui.set_progress(0.5);
                                    }

                                    println!("Now copying over to target directory");

                                    let log_path = PathBuf::from(&extract_to).join("existing.txt");
                                    println!("Path: {}", game_path.display());

                                    {
                                        if log_path.exists() {
                                            match fs::remove_file(&log_path) {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    if let Some(ui) = ui_handle.upgrade() {
                                                        println!(
                                                            "Failed to remove log file: {}",
                                                            e
                                                        );
                                                        let error = format!("Error: {}", e);
                                                        ui.set_footer(SharedString::from(error));
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    {
                                        let mut log_file = fs::File::create(&log_path).unwrap();
                                        match log_file.write_all(
                                            format!("{}\n", &game_path.display()).as_bytes(),
                                        ) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                if let Some(ui) = ui_handle.upgrade() {
                                                    println!("Failed to write log file: {}", e);
                                                    let error = format!("Error: {}", e);
                                                    ui.set_footer(SharedString::from(error));
                                                }
                                            }
                                        }
                                    }

                                    match file_manager::copy_to_dir(
                                        &game_path,
                                        &PathBuf::from(&extract_to),
                                        Path::new(""),
                                        overwrite,
                                        &log_path,
                                        symlink,
                                    ) {
                                        Ok(_) => {
                                            if let Some(ui) = ui_handle.upgrade() {
                                                println!("Copied files correctly");
                                                ui.set_footer(SharedString::from(
                                                    "Succesfully copied files",
                                                ));
                                                ui.set_progress(1.0);

                                                let title = &game_path
                                                    .file_name()
                                                    .unwrap()
                                                    .to_string_lossy();

                                                let temp_path = match env::current_dir() {
                                                    Ok(path) => {
                                                        format!(
                                                            "{}/{}",
                                                            path.display(),
                                                            &extract_to
                                                        )
                                                    }
                                                    Err(e) => {
                                                        println!(
                                                            "Failed to get current directory: {}",
                                                            e
                                                        );
                                                        String::from("")
                                                    }
                                                };
                                                let path_profile = &game_path.to_string_lossy();

                                                profile_manager::save_data(
                                                    title,
                                                    &temp_path,
                                                    path_profile,
                                                )
                                                .unwrap();

                                                reload_profiles(&ui_copy);
                                            }
                                        }
                                        Err(e) => {
                                            if let Some(ui) = ui_handle.upgrade() {
                                                println!("Failed to copy files: {}", e);
                                                let error = format!("Error: {}", e);
                                                ui.set_footer(SharedString::from(error));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    if let Some(ui) = ui_handle.upgrade() {
                                        println!("Failed to extract files: {}", e);
                                        let error = format!("Error: {}", e);
                                        ui.set_footer(SharedString::from(error));
                                    }
                                }
                            };
                        }
                    }
                }
            } else {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_footer(SharedString::from("No archive selected"));
                }
            }
        });
    }

    {
        let ui_copy = Rc::clone(&ui);

        ui.on_restore(move |path_to_profile| {
            let profile = PathBuf::from(path_to_profile.to_string());
            file_manager::restore(&profile);
        });
    }

    {
        let ui_copy = Rc::clone(&ui);

        ui.on_reload_profiles(move || {
            reload_profiles(&ui_copy).unwrap();
        });
    }

    ui.run()?;

    Ok(())
}

fn reload_profiles(ui: &Rc<AppWindow>) -> Result<(), Box<dyn Error>> {
    let mut profiles = Vec::new();
	let profile_path = PathBuf::from("profiles");
	if !profile_path.exists() {
fs::create_dir_all(&profile_path);
}

    for entry in std::fs::read_dir(PathBuf::from("profiles"))? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Ok(conf) = Ini::load_from_file(&path) {
            if let Some(section) = conf.section(Some("profile")) {
                let profile_data = ProfileData {
                    cover_image: slint::Image::load_from_path(Path::new("notfound.png"))
                        .unwrap_or_default(),
                    title: section.get("title").unwrap_or("Unknown").to_string().into(),
                    year: section.get("year").unwrap_or("Unknown").to_string().into(),
                    path_to_profile: section.get("temp_path").unwrap_or("").to_string().into(),
                };

                profiles.push(profile_data);
            }
        }
    }

    // Create and set the model
    let profiles_model = Rc::new(VecModel::from(profiles));
    let profiles_model_rc = ModelRc::from(profiles_model);
    ui.set_profiles(profiles_model_rc);

    Ok(())
}
