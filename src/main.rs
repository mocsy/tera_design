#![forbid(unsafe_code)]

#[macro_use]
extern crate tera;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use actix_files as fs;
use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    error, get, middleware, web, App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer,
};

// store tera template in application state
// #[get("{any:.*}")]
fn templates(
    req: HttpRequest,
    tmpl: web::Data<std::sync::Mutex<tera::Tera>>,
    id: Identity,
    state: web::Data<std::sync::Mutex<MyAppState>>,
) -> Result<HttpResponse, Error> {
    let state = state.lock().unwrap();
    if !state.secret.is_empty() {
        if id.identity().is_none() {
            return Ok(HttpResponse::Found().header("location", "/unlock").finish());
        }
    }
    let mut tmpl = tmpl.lock().unwrap();
    if let Err(e) = tmpl.full_reload() {
        println!("Error during template reload: {:?}", e);
    }
    let s = if let Ok(pth) = web::Path::<String>::extract(&req) {
        println!("fn templates: {}", &pth);
        let file = if pth.is_empty() {
            "index.html".to_owned()
        } else {
            if let Some(_) = std::path::Path::new(&pth.to_owned())
                .extension()
                .and_then(std::ffi::OsStr::to_str)
            {
                pth.to_owned()
            } else {
                let mut fl = pth.to_owned();
                fl.push_str(".html");
                fl
            }
        };
        match get_context(&file) {
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
struct MyAppState {
    // note: it's just an invite code, not a password
    secret: String,
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,tera_design=info");
    env_logger::init();

    let version = env!("CARGO_PKG_VERSION");
    info!(
        "Tera design {} dev server listening on http://127.0.0.1:8080",
        version
    );
    HttpServer::new(|| {
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
            .data(std::sync::Mutex::new(tera))
            .data(std::sync::Mutex::new(state))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth")
                    .secure(false),
            ))
            .wrap(middleware::Logger::default()) // enable logger
            .service(
                web::resource("/unlock")
                    .route(web::post().to(login_save))
                    .route(web::get().to(login)),
            )
            .service(web::resource("/lock").to(logout))
            .service(favicon)
            .service(fs::Files::new("/css", "css").show_files_listing())
            .service(fs::Files::new("/js", "js").show_files_listing())
            .service(fs::Files::new("/vendor", "vendor").show_files_listing())
            .service(fs::Files::new("/img", "img").show_files_listing())
            .service(fs::Files::new("/fonts", "fonts").show_files_listing())
            .service(web::resource("{any:.*}").route(web::get().to(templates)))
    })
    .bind("127.0.0.1:8080")?
    .run()
}

fn get_context(file: &str) -> Result<serde_json::Value, Error> {
    let mut fl = "templates/".to_owned();
    fl.push_str(file);
    let ctx_file = if let Some(file_ext) = std::path::Path::new(&fl)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
    {
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
    // Ok(serde_json::from_str("{}").unwrap())
    Ok(final_ctx)
}

/// favicon handler
#[get("/favicon")]
fn favicon() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[derive(Deserialize)]
struct Invite {
    secret: String,
}

fn login_save(
    id: Identity,
    invinf: web::Form<Invite>,
    state: web::Data<std::sync::Mutex<MyAppState>>,
) -> HttpResponse {
    let state = state.lock().unwrap();
    println!("{:?} {:?}", invinf.secret, state.secret);
    if invinf.secret.eq(&state.secret) {
        id.remember("visitor".to_owned());
    }
    HttpResponse::Found().header("location", "/").finish()
}

fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Found().header("location", "/unlock").finish()
}

fn login() -> HttpResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <title>This site is invite only</title>
</head>
<body>
    <form method="post">
      <input type="text" name="secret" /><br/>
      <p><input type="submit" value="Submit Invite code"></p>
    </form>
</body>
</html>"#;
    HttpResponse::Ok().content_type("text/html").body(html)
}
