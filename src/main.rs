#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use rfd::FileDialog;
use slint::{SharedString, Weak};
use std::{
    cell::RefCell,
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

    let mut archive_path = Rc::new(RefCell::new(PathBuf::new()));
    let extract_to = Rc::new(String::from(".temp/"));
    let mut game_path = String::new();

    {
        let archive_path = archive_path.clone();
        let extract_to = extract_to.clone();
        let ui_copy = Rc::clone(&ui);

        ui.on_request_path(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                ui_copy.set_archive_path(SharedString::from(path.to_str().unwrap()));
                *archive_path.borrow_mut() = path;
            }
        });
    }

    {
        let archive_path = archive_path.clone();
        println!("{:?}", archive_path);
        let extract_to = extract_to.clone();
        let ui_copy = Rc::clone(&ui);
        ui.on_mod(move || {
            let path = PathBuf::from(ui_copy.get_archive_path().to_string());
            let exts = ["zip", "rar", "7z"];
            for entry in fs::read_dir(&*path).unwrap() {
                let entry = entry.unwrap();
                println!("Entry: {}", entry.path().display());
                let path = entry.path();

                if let Some(extension) = path.extension() {
                    if exts.contains(&extension.to_str().unwrap()) {
                        let result = extract_file(&path.to_str().unwrap().to_string(), &extract_to);
                        println!("Extracted files correctly?: {}", result.is_ok());
                    }
                }
            }
            let path_str = path.to_str().unwrap().to_string();

            match extract_file(&path_str, &extract_to) {
                Ok(_) => {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_dir_path(SharedString::from("Succesfully extracted files"));
                    }
                }

                Err(e) => {
                    if let Some(ui) = ui_handle.upgrade() {
                        let error = format!("Error: {}", e);
                        ui.set_dir_path(SharedString::from(error));
                    }
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

use sevenz_rust2::Archive as SevenZArchive;
fn extract_7z(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
    println!("Extracting 7z ({}) to ({})", archive_path, extract_to);
    sevenz_rust2::decompress_file(archive_path, extract_to).expect("Failed to extract 7z");
    Ok(())
}
