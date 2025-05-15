#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use dirs::data_dir;
use ini::Ini;
use rfd::FileDialog;
use slint::{ComponentHandle, SharedString};
use std::{
    env,
    error::Error,
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

mod extract;
mod file_manager;
mod profile_manager;

fn main() -> Result<(), Box<dyn Error>> {
    let window = AppWindow::new()?;
    let ui = Arc::new(window);

    let ui_handle = Arc::downgrade(&ui);

    println!("Hello, world!");

    // let archive_path = Rc::new(RefCell::new(PathBuf::new()));

    {
        // let archive_path = archive_path.clone();
        // let extract_to = extract_to.clone();
        let ui_copy = Arc::clone(&ui);

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
        let ui_copy = Arc::clone(&ui);

        ui.on_request_game_path(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                if let Some(path_str) = path.to_str() {
                    ui_copy.set_game_path(SharedString::from(path_str));
                }
            }
        });
    }

    {
        let ui_copy = Arc::clone(&ui);
        ui.on_mod(move || {
            let path = PathBuf::from(ui_copy.get_archive_path().to_string());
            let game_path = PathBuf::from(ui_copy.get_game_path().to_string());
            let extract_to = match &game_path.file_name() {
                Some(name) => format!("oxide/.temp/{}/", name.to_string_lossy()),
                None => String::from("oxide/.temp/Unknown/"),
            };

            let extract_to_data_dir = match data_dir() {
                Some(mut data_path) => {
                    data_path.push(extract_to);
                    data_path
                }
                None => {
                    eprintln!("Error getting data directory");
                    PathBuf::new()
                }
            };

            let overwrite = ui_copy.get_overwrite();
            let symlink = ui_copy.get_symlink();
            let exts = ["zip", "rar", "7z"];

            let _ = fs::remove_dir_all(match data_dir() {
                Some(path) => {
                    if !path.join(&extract_to_data_dir).exists() {
                        extract_to_data_dir.clone()
                    } else {
                        PathBuf::new()
                    }
                }
                None => {
                    println!("Failed to get data directory");
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
                            match extract::extract_file(
                                path.to_str().unwrap(),
                                &extract_to_data_dir,
                            ) {
                                Ok(_) => {
                                    if let Some(ui) = ui_handle.upgrade() {
                                        println!("Extracted files correctly");
                                        ui.set_footer(SharedString::from(
                                            "Successfully extracted files",
                                        ));
                                        ui.set_progress(0.5);
                                    }

                                    println!("Now copying over to target directory");

                                    let log_path =
                                        PathBuf::from(&extract_to_data_dir).join("existing.txt");
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
                                        &extract_to_data_dir,
                                        Path::new(""),
                                        overwrite,
                                        &log_path,
                                        symlink,
                                    ) {
                                        Ok(_) => {
                                            if let Some(ui) = ui_handle.upgrade() {
                                                println!("Copied files correctly");
                                                ui.set_footer(SharedString::from(
                                                    "Successfully copied files",
                                                ));
                                                ui.set_progress(1.0);

                                                let title = &game_path
                                                    .file_name()
                                                    .unwrap()
                                                    .to_string_lossy();

                                                let temp_path =
                                                    &extract_to_data_dir.to_str().unwrap();

                                                let path_profile = &game_path.to_string_lossy();

                                                match profile_manager::save_data(
                                                    title,
                                                    temp_path,
                                                    path_profile,
                                                ) {
                                                    Ok(_) => {}
                                                    Err(e) => {
                                                        if let Some(ui) = ui_handle.upgrade() {
                                                            println!("Failed to save data: {}", e);
                                                            let error = format!("Error: {}", e);
                                                        }
                                                    }
                                                }

                                                profile_manager::reload_profiles(&ui_copy).unwrap();
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
            } else if let Some(ui) = ui_handle.upgrade() {
                ui.set_footer(SharedString::from("No archive selected"));
            }
        });
    }

    {
        let ui_copy = Arc::clone(&ui);

        ui.on_restore(move |name| {
            let ini = match Ini::load_from_file(PathBuf::from(format!("profiles/{}.ini", name))) {
                Ok(ini) => ini,
                Err(e) => {
                    ui_copy.set_footer(SharedString::from(format!("Error: {}", e)));
                    return;
                }
            };
            let path_to_profile = if let Some(section) = ini.section(Some("profile")) {
                section.get("temp_path").unwrap_or("").to_string()
            } else {
                String::from("")
            };

            let profile = PathBuf::from(path_to_profile.to_string());
            println!("Restoring profile: {}", profile.display());
            match file_manager::restore(&profile) {
                Ok(_) => {
                    profile_manager::reload_profiles(&ui_copy).unwrap();
                    println!("Restored profile: {}", profile.display());
                }
                Err(e) => {
                    println!("Failed to restore profile: {}", e);
                }
            }
        });
    }

    {
        let ui_copy = Arc::clone(&ui);

        ui.on_reload_profiles(move || {
            match profile_manager::reload_profiles(&ui_copy) {
                Ok(profiles_model_rc) => {
                    ui_copy.set_profiles(profiles_model_rc);
                }
                Err(e) => {
                    println!("Failed to reload profiles: {}", e);
                }
            };
        });
    }

    {
        let ui_copy = Arc::clone(&ui);

        ui.on_update_profile(move |title, temp_path, profile_path| {
            println!("Data: {}, {}, {}", title, temp_path, profile_path);
            profile_manager::save_data(&title, &temp_path, &profile_path).unwrap();
        });
    }
    ui.run()?;

    Ok(())
}
