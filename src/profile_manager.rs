use ini::Ini;
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use steamgriddb_api::Client;
use steamgriddb_api::query_parameters::QueryType::Grid;

pub fn save_data(
    title: &str,
    temp_path: &str,
    path_profile: &str,
    //cover_image: &str,
    //year: &str,
) -> Result<(), Box<dyn Error>> {
    let mut data = Ini::new();

    data.with_section(Some("profile"))
        .set("title", title)
        .set("temp_path", temp_path)
        .set("year", "")
        .set("path_profile", path_profile)
        .set("cover_image", "");

    fs::create_dir(PathBuf::from("profiles"));
    data.write_to_file(PathBuf::from(format!("profiles/{}.ini", title)))?;

    println!("Saving data");
    Ok(())
}

fn get_cover_image(title: &str) -> Result<String, Box<dyn Error>> {
    Ok("notfound.png".into())
}
