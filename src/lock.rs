
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

use super::MyAppState;

#[derive(Deserialize)]
pub(crate) struct Invite {
    secret: String,
}

pub(crate) fn login_save(
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

pub(crate) fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Found().header("location", "/unlock").finish()
}

pub(crate) fn login() -> HttpResponse {
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
