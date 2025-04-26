use std::{
    env,
    error::Error,
    fs::{self, OpenOptions, read_to_string},
    io::prelude::*,
    path::Path,
    path::PathBuf,
};

pub fn copy_to_dir(
    copy_to: &Path,
    profile: &PathBuf,
    start_point: &Path,
    overwrite: bool,
    log_path: &PathBuf,
    symlink: bool,
) -> Result<(), Box<dyn Error>> {
    let mut walk_dir = PathBuf::new();
    match env::current_dir() {
        Ok(path) => {
            walk_dir = path.join(&profile).join("extracted").join(start_point);
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
            copy_to_dir(
                &dst_path,
                &profile,
                &new_walk_dir,
                overwrite,
                &log_path,
                symlink,
            )?;
        } else if metadata.is_file() {
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

                let copy_bak_to = split_path(&dst_path, profile).unwrap();
                println!(
                    "Backing up file: '{}' -> '{}'",
                    dst_path.display(),
                    copy_bak_to.display()
                );
                create_bak_folders(&dst_path, &copy_bak_to, profile, &PathBuf::new())?;

                if !overwrite {
                    continue;
                }
            }

            if !symlink {
                println!(
                    "Copying file: '{}' -> '{}'",
                    src_path.display(),
                    dst_path.display()
                );

                println!("Copying file");
                fs::copy(&src_path, &dst_path)?;
            } else {
                println!(
                    "Creating symlink to file: '{}' -> '{}'",
                    src_path.display(),
                    dst_path.display()
                );

                if dst_path.exists() {
                    println!("Removing old copy/symlink");
                    fs::remove_file(&dst_path)?;
                }
                create_symlink(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}

#[cfg(target_family = "unix")]
fn create_symlink(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    std::os::unix::fs::symlink(src, dst)?;
    Ok(())
}

#[cfg(target_family = "windows")]
fn create_symlink(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    if !src.is_dir() {
        std::os::windows::fs::symlink_file(src, dst)?;
    }
    Ok(())
}

pub fn restore(profile: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!(
        "Restoring profile ({}) to original state",
        profile.display()
    );
    let profile_existing = read_to_string(&profile.join("existing.txt"))?.lines();

    Ok(())
}

fn split_path(path: &PathBuf, profile: &PathBuf) -> Option<PathBuf> {
    let path_str = path.to_string_lossy();
    let profile_filename = profile.file_name()?;
    let where_to = match path_str.split_once(profile_filename.to_str()?) {
        Some((_, where_to)) => where_to,
        None => return None,
    };

    let path_dir = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            println!("Failed to get current directory (split_path): {}", e);
            PathBuf::new()
        }
    };
    let complete_path = PathBuf::from(format!(
        "{}/{}bak{}",
        path_dir.display(),
        profile.display(),
        where_to
    ));

    Some(PathBuf::from(complete_path))
}

fn create_bak_folders(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::copy(&src, &dst)?;

    Ok(())
}
