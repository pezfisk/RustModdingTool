use std::{env, fs};

fn main() {
    let profile = env::var("PROFILE").unwrap();
    match profile.as_str() {
        "release" => {
            fs::copy("notfound.png", "target/release/notfound.png").unwrap();
        }

        "debug" => {
            fs::copy("notfound.png", "target/debug/notfound.png").unwrap();
        }

        _ => {
            panic!("This shouldn't happen, cargo profile: {}", profile);
        }
    }

    let mut config = slint_build::CompilerConfiguration::new().with_style("qt".into());
    match std::env::consts::OS {
        "windows" => {
            config = config.with_style("fluent".into());
        }

        "linux" => {
            config = config.with_style("cosmic".into());
        }

        "macos" => {
            config = config.with_style("cupertino".into());
        }

        _ => {
            config = config.with_style("qt".into());
        }
    }

    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
