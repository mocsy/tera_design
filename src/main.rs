#![forbid(unsafe_code)]

#[macro_use]
extern crate tera;
mod config;
mod lock;

use actix_files as fs;
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::{
    cookie::Key,
    error, get, middleware,
    web::{self, Data},
    App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer,
};
use log::{debug, error, info};
use serde::Deserialize;

// store tera template in application state
// #[get("{any:.*}")]
async fn templates(
    req: HttpRequest,
    tmpl: web::Data<std::sync::Mutex<tera::Tera>>,
    id: Option<Identity>,
    state: web::Data<std::sync::Mutex<MyAppState>>,
    cfg: web::Data<config::Config>,
) -> Result<HttpResponse, Error> {
    let state = state.lock().unwrap();
    if !state.secret.is_empty() && id.is_none() {
        return Ok(HttpResponse::Found()
            .append_header(("location", "/unlock"))
            .finish());
    }
    let mut tmpl = tmpl.lock().unwrap();
    if let Err(e) = tmpl.full_reload() {
        debug!("Error during template reload: {:?}", e);
    }
    let s = if let Ok(pth) = web::Path::<String>::extract(&req).into_inner() {
        debug!("fn templates: path {}", &pth);
        let file = if pth.is_empty() {
            "index.html".to_owned()
        } else if std::path::Path::new(&pth.to_owned()).extension().is_some() {
            pth.to_owned()
        } else {
            let mut fl = pth.to_owned();
            fl.push_str(".html");
            fl
        };

        match get_context(&file, &cfg) {
            Ok(ctx) => tmpl.render(&file, &ctx).map_err(|e| {
                error::ErrorInternalServerError(format!(
                    "Template error: {} with context: {:?}",
                    e, &ctx
                ))
            })?,
            Err(e) => {
                error!("{}", e);
                format!("Couldn't read context json: {}", e)
            }
        }
    } else {
        tmpl.render("404.html", &tera::Context::new())
            .map_err(|e| error::ErrorInternalServerError(format!("Template error: {}", e)))?
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[derive(Deserialize, Default)]
pub(crate) struct MyAppState {
    // note: it's just an invite code, not a password
    secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,tera_design=debug");
    env_logger::init();

    let version = env!("CARGO_PKG_VERSION");
    let cfg = config::load_config();
    let port = cfg.bind_port;
    let signing_key = Key::generate();

    info!(
        "Tera design {} dev server listening on http://127.0.0.1:{}",
        version, port
    );
    HttpServer::new(move || {
        let tera = compile_templates!("templates/**/*");

        let state = if std::path::Path::new("lockdown.json").exists() {
            if let Ok(file) = std::fs::File::open("lockdown.json") {
                let reader = std::io::BufReader::new(file);
                let json: MyAppState = serde_json::from_reader(reader).unwrap_or_default();
                json
            } else {
                MyAppState {
                    secret: String::new(),
                }
            }
        } else {
            MyAppState {
                secret: String::new(),
            }
        };

        App::new()
            .app_data(Data::new(std::sync::Mutex::new(tera)))
            .app_data(Data::new(std::sync::Mutex::new(state)))
            .app_data(Data::new(cfg.clone()))
            // Install the identity framework first.
            .wrap(IdentityMiddleware::default())
            // The identity system is built on top of sessions.
            // You must install the session middleware to leverage `actix-identity`.
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), signing_key.clone())
                    .cookie_secure(false)
                    .build(),
            )
            .wrap(middleware::Logger::default()) // enable logger
            .service(
                web::resource("/unlock")
                    .route(web::post().to(lock::login_save))
                    .route(web::get().to(lock::login)),
            )
            .service(web::resource("/lock").to(lock::logout))
            .service(favicon)
            .configure(config::config_statics)
            .service(web::resource("{any:.*}").route(web::get().to(templates)))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}

fn get_context(file: &str, _cfg: &config::Config) -> Result<serde_json::Value, Error> {
    // let template_dir = cfg.template_dir.clone();
    let template_dir = "templates/".to_owned();
    let tdir = std::path::Path::new(&template_dir);
    let file_path = tdir.join(file);
    let fl = file_path.to_str().unwrap().to_owned();
    debug!("Ctx: template file: {}", &fl);
    let ctx_file = if let Some(file_ext) = file_path.extension().and_then(std::ffi::OsStr::to_str) {
        fl.replace(file_ext, "json")
    } else {
        let mut s = fl.to_owned();
        s.push_str(".json");
        s
    };

    let mod_file = std::path::Path::new(&fl).parent().unwrap().join("mod.json");
    let mut final_ctx = if std::path::Path::new(&mod_file).exists() {
        let file = std::fs::File::open(mod_file)?;
        let reader = std::io::BufReader::new(file);
        let json: serde_json::Value = serde_json::from_reader(reader)?;
        json
    } else {
        let json: serde_json::Value = serde_json::from_str("{}")?;
        json
    };

    let local_ctx = if std::path::Path::new(&ctx_file).exists() {
        let file = std::fs::File::open(ctx_file)?;
        let reader = std::io::BufReader::new(file);
        let json: serde_json::Value = serde_json::from_reader(reader)?;
        json
    } else {
        let json: serde_json::Value = serde_json::from_str("{}")?;
        json
    };

    json_patch::merge(&mut final_ctx, &local_ctx);
    Ok(final_ctx)
}

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}
