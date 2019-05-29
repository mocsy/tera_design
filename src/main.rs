#![forbid(unsafe_code)]

#[macro_use]
extern crate tera;
#[macro_use]
extern crate log;

use std::collections::HashMap;

use actix_files as fs;
use actix_web::http::StatusCode;
use actix_web::{
    error, get, guard, middleware, web, App, Error, FromRequest, HttpRequest, HttpResponse,
    HttpServer,
};

// store tera template in application state
// #[get("{any:.*}")]
fn templates(
    req: HttpRequest,
    tmpl: web::Data<std::sync::Mutex<tera::Tera>>
) -> Result<HttpResponse, Error> {
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
        let ctx = get_context(&file).unwrap();
        tmpl.render(&file, &ctx).map_err(|e| {
            error::ErrorInternalServerError(format!(
                "Template error: {} with context: {:?}",
                e, &ctx
            ))
        })?
    } else {
        tmpl.render("404.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        let tera = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));

        App::new()
            .data(std::sync::Mutex::new(tera))
            .wrap(middleware::Logger::default()) // enable logger
            .service(favicon)
            .service(fs::Files::new("/css", "css").show_files_listing())
            .service(fs::Files::new("/js", "js").show_files_listing())
            .service(fs::Files::new("/vendor", "vendor").show_files_listing())
            .service(fs::Files::new("/img", "img").show_files_listing())
            .service(web::resource("{any:.*}").route(web::get().to(templates)))
    })
    .bind("127.0.0.1:8080")?
    .run()
}

fn get_context(file: &str) -> Result<HashMap<String, String>, Error> {
    let mut fl = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/").to_owned();
    fl.push_str(file);
    let ctx_file = if let Some(file_ext) = std::path::Path::new(&fl)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
    {
        fl.replace(file_ext, "ctx")
    } else {
        let mut s = fl.to_owned();
        s.push_str(".ctx");
        s
    };
    if std::path::Path::new(&ctx_file).exists() {
        let file = std::fs::File::open(ctx_file)?;
        let reader = std::io::BufReader::new(file);
        let json: HashMap<String, String> = serde_json::from_reader(reader)?;
        return Ok(json);
    }
    Ok(HashMap::new())
}

/// favicon handler
#[get("/favicon")]
fn favicon() -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}
