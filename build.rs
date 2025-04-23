fn main() {
    let mut config = slint_build::CompilerConfiguration::new().with_style("cosmic".into());
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
