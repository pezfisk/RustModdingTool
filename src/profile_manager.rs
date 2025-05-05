use crate::{AppWindow, ProfileData};
use ini::Ini;
use slint::{ModelRc, VecModel};
use std::path::Path;
use std::rc::Rc;
use std::{
    error::Error,
    fs::{self},
    path::PathBuf,
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
    let profile = PathBuf::from("profiles");
    let mut data = Ini::new();

    let cover_image = get_cover_image(title).unwrap();
    data.with_section(Some("profile"))
        .set("title", title)
        .set("temp_path", temp_path)
        .set("path_profile", path_profile)
        .set("cover_image", "")
        .set("year", "");

    if !profile.exists() {
        fs::create_dir(PathBuf::from("profiles"))?;
    }

    data.write_to_file(PathBuf::from(format!("profiles/{}.ini", title)))?;

    println!("Saving data");
    Ok(())
}

fn get_cover_image(title: &str) -> Result<String, Box<dyn Error>> {
    Ok("notfound.png".into())
    // TODO -> Figure out how to get the cover image from steamgriddb
}

pub fn reload_profiles(ui: &Rc<AppWindow>) -> Result<(), Box<dyn Error>> {
    let mut profiles = Vec::new();
    let profile_path = PathBuf::from("profiles");
    if !profile_path.exists() {
        let _ = fs::create_dir_all(&profile_path);
    }

    for entry in std::fs::read_dir(PathBuf::from("profiles"))? {
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

                println!("Profile {:?}", profile_data.temp_path);

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
