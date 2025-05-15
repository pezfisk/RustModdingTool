use crate::{profile_manager, AppWindow, ProfileData};
use dirs::data_dir;
use ini::Ini;
use slint::{Image, ModelRc, VecModel};
use std::{
    error::Error,
    fs::{self},
    io::Cursor,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use reqwest::get;

use image::io::Reader as ImageReader;

use steamgriddb_api::query_parameters::QueryType::Grid;
use steamgriddb_api::Client;
use tokio::runtime::Runtime;

pub fn save_data<'a>(
    title: &'a str,
    temp_path: &str,
    path_profile: &str,
    //cover_image: &str,
    //year: &str,
) -> Result<(), Box<dyn Error>> {
    let data_dir = data_dir().unwrap_or_else(|| {
        println!("Failed to get data directory");
        PathBuf::new()
    });

    let mut profile = {
        let mut path = data_dir;
        path.push("oxide/profiles");
        path
    };

    let mut data = Ini::new();

    //let cover_image = get_cover_image(title)?;
    data.with_section(Some("profile"))
        .set("title", title)
        .set("temp_path", temp_path)
        .set("path_profile", path_profile)
        .set("cover_image", "")
        .set("year", "");

    if !profile.exists() {
        fs::create_dir(&profile)?;
    }

    profile.push(PathBuf::from(format!("{}.ini", title)));

    data.write_to_file(&profile)?;

    println!("Saving data");
    Ok(())
}

pub fn reload_profiles(ui: &Arc<AppWindow>) -> Result<ModelRc<ProfileData>, Box<dyn Error>> {
    let mut profiles = Vec::new();

    let data_dir = data_dir().unwrap_or_else(|| {
        println!("Failed to get data directory");
        PathBuf::new()
    });

    let profile_path = {
        let mut path = data_dir;
        path.push("oxide/profiles");
        path
    };

    println!("profile_path: {}", profile_path.display());

    if !profile_path.exists() {
        let _ = fs::create_dir_all(&profile_path);
    }

    for entry in fs::read_dir(&profile_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Ok(conf) = Ini::load_from_file(&path) {
            if let Some(section) = conf.section(Some("profile")) {
                let title = section.get("title").unwrap_or("Unknown").to_string();
                let try_image = section.get("cover_image").unwrap_or("Unknown").to_string();
                let cover_image = load_cover_image(try_image, title.clone()).unwrap();

                let profile_data = ProfileData {
                    cover_image,
                    title: title.into(),
                    year: section.get("year").unwrap_or("Unknown").to_string().into(),
                    path_to_profile: section
                        .get("path_profile")
                        .unwrap_or("Not found?")
                        .to_string()
                        .into(),
                    temp_path: section
                        .get("temp_path")
                        .unwrap_or("Not found?")
                        .to_string()
                        .into(),
                    name: section.get("name").unwrap_or("Unknown").to_string().into(),
                };

                profiles.push(profile_data);
            }
        }
    }

    println!("Profiles: {}", profiles.len());

    let profiles_model = Rc::new(VecModel::from(profiles));
    let profiles_model_rc = ModelRc::from(profiles_model);

    Ok(profiles_model_rc)
}

fn load_cover_image(path: String, title: String) -> Result<Image, Box<dyn Error>> {
    println!("Trying image in ini file: {}", path);
    if let Ok(image) = slint::Image::load_from_path(&PathBuf::from(&path)) {
        return Ok(image);
    }

    println!("Trying image with same name: {}", title);

    let data_dir = data_dir().unwrap_or_else(|| {
        println!("Failed to get data directory");
        PathBuf::new()
    });

    let profile = {
        let mut path = data_dir;
        let profile_path = PathBuf::from(format!("oxide/profiles/{}.png", title));
        path.push(profile_path);
        path
    };

    match slint::Image::load_from_path(&PathBuf::from(&profile)) {
        Ok(image) => {
            return Ok(image);
        }
        Err(..) => {
            let rt = Runtime::new()?;

            println!("Trying image from steamgriddb: {}", title);
            match rt.block_on(profile_manager::get_cover_image(&title)) {
                Ok(..) => {
                    return Ok(slint::Image::load_from_path(&PathBuf::from(&profile))?);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }

    println!("Falling back to notfound.png");
    if let Ok(image) = slint::Image::load_from_path(&PathBuf::from("notfound.png")) {
        Ok(image)
    } else {
        Err("Failed to load image".into())
    }
}

pub async fn get_cover_image(title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client_key = Client::new("");

    let data_dir = data_dir().unwrap_or_else(|| {
        println!("Failed to get data directory");
        PathBuf::new()
    });

    let profile = {
        let mut path = data_dir;
        let profile_path = PathBuf::from(format!("oxide/profiles/{}.png", title));
        path.push(profile_path);
        path
    };

    println!("profile: {}", profile.display());

    if !profile.exists() {
        let client = client_key;
        let games = client.search(title).await?;
        let first_game = games.iter().next().ok_or("No games found")?;

        if first_game.name != title {
            return Err(format!("Game not found: {}", title).into());
        }

        assert_eq!(title, first_game.name);
        let images = client.get_images_for_id(first_game.id, &Grid(None)).await?;
        let first_image = images.iter().next().ok_or("No images found")?;

        println!("Getting image from: {:?}", first_image.url);

        let response = get(&first_image.url).await?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download image: HTTP status code {}",
                response.status()
            )
            .into());
        }

        let image_bytes = response.bytes().await?;

        println!("Image data downloaded.");

        let cursor = Cursor::new(image_bytes);
        let reader = ImageReader::new(cursor).with_guessed_format()?;

        let img = reader.decode()?;

        println!("Image decoded successfully.");

        img.save(profile)?;

        println!("Image saved");
    }
    Ok(())
}
