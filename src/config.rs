use actix_files as fs;
use actix_web::web;
use ron::de::from_reader;
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    static_dirs: Vec<String>,
}

fn load_config() -> Config {
    let conf_file = File::open("./config.ron").expect("Failed opening file");
    match from_reader(&conf_file) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn config_statics(cfg: &mut web::ServiceConfig) {
    let conf = load_config();
    for dr in conf.static_dirs {
        let srv_dir = format!("/{}", dr);
        let from_dir = format!("{}", dr);
        cfg.service(fs::Files::new(&srv_dir, &from_dir).show_files_listing());
    }
}
