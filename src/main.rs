#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use rfd::FileDialog;
use slint::SharedString;
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{self, prelude::*},
    path::{Path, PathBuf},
    rc::Rc,
};

fn main() -> Result<(), Box<dyn Error>> {
    let window = AppWindow::new()?;
    let ui = Rc::new(window);

    let ui_handle = ui.as_weak();

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
            let exts = ["zip", "rar", "7z"];

            let _ = fs::remove_dir_all(match env::current_dir() {
                Ok(path) => path.join(&extract_to),
                Err(e) => {
                    println!("Failed to get current directory: {}", e);
                    PathBuf::new()
                }
            });

            println!("Path: '{}'", path.display());
            if path.exists() {
                for entry in fs::read_dir(&*path).unwrap() {
                    let entry = entry.unwrap();
                    println!("Entry: {}", entry.path().display());
                    let path = entry.path();

                    if let Some(extension) = path.extension() {
                        if exts.contains(&extension.to_str().unwrap()) {
                            let _result = match extract_file(
                                &path.to_str().unwrap().to_string(),
                                &extract_to,
                            ) {
                                Ok(_) => {
                                    if let Some(ui) = ui_handle.upgrade() {
                                        println!("Extracted files correctly");
                                        ui.set_footer(SharedString::from(
                                            "Succesfully extracted files",
                                        ));
                                    }

                                    println!("Now copying over to target directory");

                                    let log_path = PathBuf::from(&extract_to).join("existing.txt");
                                    println!("Path: {}", game_path.display());

                                    match copy_to_dir(
                                        &game_path,
                                        &PathBuf::from(&extract_to).join("extracted"),
                                        Path::new(""),
                                        overwrite,
                                        &log_path,
                                    ) {
                                        Ok(_) => {
                                            if let Some(ui) = ui_handle.upgrade() {
                                                println!("Copied files correctly");
                                                ui.set_footer(SharedString::from(
                                                    "Succesfully copied files",
                                                ));
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

    ui.run()?;

    Ok(())
}

fn extract_file(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
    if let Some(extension) = Path::new(&archive_path).extension() {
        let extract_to = format!("{}/extracted", extract_to);
        let _result = match extension.to_str().unwrap() {
            "zip" => extract_zip(&archive_path, &extract_to),
            "rar" => extract_rar(&archive_path, &extract_to),
            "7z" => extract_7z(&archive_path, &extract_to),
            _ => {
                println!("Not supported");
                Ok(())
            }
        };
    }

    Ok(())
}

use zip::ZipArchive;
fn extract_zip(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
    println!("Extracting zip ({}) to ({})", archive_path, extract_to);

    let file = fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = Path::new(extract_to).join(outpath);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            // if let Some(p) = outpath.parent() {
            //    if !p.exists() {
            //        fs::create_dir_all(p).unwrap();
            //    }
            //}

            println!("Creating file at: {:?}", outpath);

            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("Extracted to ({})", extract_to);

    Ok(())
}

use unrar::Archive;
fn extract_rar(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
    println!("Extracting RAR ({}) to ({})", archive_path, extract_to);

    let mut archive = Archive::new(archive_path).open_for_processing()?;

    while let Some(header) = archive.read_header().unwrap() {
        println!(
            "Creating file at: .temp/{}",
            header.entry().filename.to_string_lossy()
        );
        archive = if header.entry().is_file() {
            header.extract_with_base(extract_to)?
        } else {
            header.skip()?
        };
    }

    println!("Extracted to ({})", extract_to);
    Ok(())
}

fn extract_7z(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
    println!("Extracting 7z ({}) to ({})", archive_path, extract_to);
    sevenz_rust2::decompress_file(archive_path, extract_to).expect("Failed to extract 7z");
    Ok(())
}

fn copy_to_dir(
    copy_to: &Path,
    profile: &PathBuf,
    start_point: &Path,
    overwrite: bool,
    log_path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut walk_dir = PathBuf::new();
    match env::current_dir() {
        Ok(path) => {
            walk_dir = path.join(&profile).join(start_point);
            //walk_dir.push(path);
            //walk_dir.push(".temp/");
            //walk_dir.push(start_point);
        }
        Err(e) => {
            eprintln!("Failed to get current directory: {}", e);
        }
    }
    println!(
        "Copying files from {:?} to ({:?})",
        walk_dir.display(),
        copy_to
    );
    for entry_result in fs::read_dir(&*walk_dir)? {
        let entry = entry_result?;
        let src_path = entry.path();
        let filename = entry.file_name();
        let dst_path = copy_to.join(&filename);
        let metadata = entry.metadata()?;

        println!(
            "Entry is directory: {} ({})",
            metadata.is_dir(),
            src_path.display()
        );
        if metadata.is_dir() {
            println!(
                "Creating directory and copying contents: '{}' -> '{}'",
                src_path.display(),
                dst_path.display()
            );

            let new_walk_dir = walk_dir.join(&filename);
            match fs::create_dir(&dst_path) {
                Ok(_) => {
                    println!("Created directory: {}", dst_path.display());
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {
                        println!("Directory already exists: {}", dst_path.display());
                    }
                    _ => {
                        println!("Failed to create directory: {}", e);
                        return Err(e.into());
                    }
                },
            }
            copy_to_dir(&dst_path, &profile, &new_walk_dir, overwrite, &log_path)?;
        } else if metadata.is_file() {
            println!(
                "Copying file: '{}' -> '{}'",
                src_path.display(),
                dst_path.display()
            );

            if dst_path.exists() {
                println!(
                    "File already exists, writing to log: {}",
                    dst_path.display()
                );

                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&log_path)?;

                let path_str = format!("{}\n", &dst_path.to_string_lossy());

                file.write_all(path_str.as_bytes())?;

                if !overwrite {
                    continue;
                }
            }

            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
