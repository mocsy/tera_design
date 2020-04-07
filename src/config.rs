use actix_files as fs;
use actix_web::web;
use ron::de::from_reader;
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Config {
    #[serde(default = "default_static_dir")]
    pub static_dirs: Vec<String>,
    // #[serde(default = "default_template_dir")]
    // pub template_dir: String,
    #[serde(default = "default_port")]
    pub bind_port: u32,
    pub version: Option<String>,
}
fn default_static_dir() -> Vec<String> {
    vec!["static".to_owned()]
}
// fn default_template_dir() -> String {
//     String::new()
// }
fn default_port() -> u32 {
    8080
}

pub(crate) fn load_config() -> Config {
    if let Ok(conf_file) = File::open("./config.ron") {
        match from_reader(&conf_file) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load config: {}, defaulting...", e);
                Config {
                    static_dirs: vec!["static".to_owned()],
                    // template_dir: "templates".to_owned(),
                    bind_port: 8080,
                    version: None,
                }
            }
        }
    } else {
        Config {
            static_dirs: vec!["static".to_owned()],
            // template_dir: "templates".to_owned(),
            bind_port: 8080,
            version: None,
        }
    }
}

pub(crate) fn config_statics(cfg: &mut web::ServiceConfig) {
    let conf = load_config();
    for dr in conf.static_dirs {
        let srv_dir = format!("/{}", dr);
        let from_dir = dr;
        cfg.service(fs::Files::new(&srv_dir, &from_dir).show_files_listing());
    }
}
