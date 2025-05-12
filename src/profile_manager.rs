use crate::{AppWindow, ProfileData};
use dirs::data_dir;
use ini::Ini;
use slint::{ModelRc, VecModel};
use std::{
    error::Error,
    fs::{self},
    path::{Path, PathBuf},
    rc::Rc,
};
//use steamgriddb_api::Client;
//use steamgriddb_api::query_parameters::QueryType::Grid;

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

    let cover_image = get_cover_image(title)?;
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

fn get_cover_image(title: &str) -> Result<String, Box<dyn Error>> {
    Ok("notfound.png".into())
    // TODO -> Figure out how to get the cover image from steamgriddb
}

pub fn reload_profiles(ui: &Rc<AppWindow>) -> Result<(), Box<dyn Error>> {
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

    for entry in std::fs::read_dir(profile_path)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Ok(conf) = Ini::load_from_file(&path) {
            if let Some(section) = conf.section(Some("profile")) {
                let title = section.get("title").unwrap_or("Unknown").to_string();
                let try_image = section.get("cover_image");
                let cover_image =
                    match slint::Image::load_from_path(&PathBuf::from(try_image.unwrap())) {
                        Ok(image) => image,
                        Err(_) => slint::Image::load_from_path(&PathBuf::from(&format!(
                            "profiles/{}.png",
                            title
                        )))
                        .unwrap_or_else(|_| {
                            slint::Image::load_from_path(Path::new("notfound.png"))
                                .unwrap_or_default()
                        }),
                    };

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
    ui.set_profiles(profiles_model_rc);

    Ok(())
}
