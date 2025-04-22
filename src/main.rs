#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use rfd::FileDialog;
use slint::SharedString;
use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
};

fn main() -> Result<(), Box<dyn Error>> {
    let window = AppWindow::new()?;
    let ui = Rc::new(window);

    let ui_handle = ui.as_weak();

    println!("Hello, world!");

    // let archive_path = Rc::new(RefCell::new(PathBuf::new()));
    let extract_to = Rc::new(String::from(".temp/"));

    {
        // let archive_path = archive_path.clone();
        // let extract_to = extract_to.clone();
        let ui_copy = Rc::clone(&ui);

        ui.on_request_archive_path(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                ui_copy.set_archive_path(SharedString::from(path.to_str().unwrap()));
                // *archive_path.borrow_mut() = path;
            }
        });
    }

    {
        let ui_copy = Rc::clone(&ui);

        ui.on_request_game_path(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                ui_copy.set_game_path(SharedString::from(path.to_str().unwrap()));
            }
        });
    }

    {
        let extract_to = extract_to.clone();
        let ui_copy = Rc::clone(&ui);
        ui.on_mod(move || {
            fs::remove_dir_all(match env::current_dir() {
                Ok(path) => path.join(".temp/"),
                Err(e) => {
                    println!("Failed to get current directory: {}", e);
                    PathBuf::new()
                }
            });

            let path = PathBuf::from(ui_copy.get_archive_path().to_string());
            let game_path = PathBuf::from(ui_copy.get_game_path().to_string());
            let exts = ["zip", "rar", "7z"];
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

                                    println!("Path: {}", game_path.display());

                                    copy_to_dir(&game_path, Path::new(""));
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
        let result = match extension.to_str().unwrap() {
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

    let file = fs::File::open(archive_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = Path::new(extract_to).join(outpath);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            // if let Some(p) = outpath.parent() {
            //    if !p.exists() {
            //        fs::create_dir_all(p).unwrap();
            //    }
            //}

            println!("Creating file at: {:?}", outpath);

            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }

            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    println!("Extracted to ({})", extract_to);

    Ok(())
}

use unrar::Archive;
fn extract_rar(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
    println!("Extracting RAR ({}) to ({})", archive_path, extract_to);

    let mut archive = Archive::new(archive_path).open_for_processing().unwrap();

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

fn copy_to_dir(extract_to: &Path, start_point: &Path) -> Result<(), Box<dyn Error>> {
    let mut walk_dir = PathBuf::new();
    match env::current_dir() {
        Ok(path) => {
            walk_dir.push(path);
            walk_dir.push(".temp/");
            walk_dir.push(start_point);
        }
        Err(e) => {
            println!("Failed to get current directory: {}", e);
        }
    }
    println!(
        "Copying files from {:?} to ({:?})",
        walk_dir.display(),
        extract_to
    );

    for entry_result in fs::read_dir(&*walk_dir)? {
        let entry = entry_result?;
        let src_path = entry.path();
        let filename = entry.file_name();
        let dst_path = extract_to.join(&filename);

        //if dst_path.exists() {
        //    println!("File already exists at: {}", dst_path.display());
        //    continue;
        //}

        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            println!(
                "Creating directory and copying contents: '{}' -> '{}'",
                src_path.display(),
                dst_path.display()
            );

            let new_walk_dir = walk_dir.join(&filename);

            fs::create_dir(&dst_path);
            copy_to_dir(&dst_path, &new_walk_dir)?;
        } else if metadata.is_file() {
            println!(
                "Copying file: '{}' -> '{}'",
                src_path.display(),
                dst_path.display()
            );

            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
