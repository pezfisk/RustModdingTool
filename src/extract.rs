use std::{error::Error, fs, io, path::Path};

pub fn extract_file(archive_path: &str, extract_to: &str) -> Result<(), Box<dyn Error>> {
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
