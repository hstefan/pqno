#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::sync::Mutex;

use rocket::fs::{FileServer, NamedFile};
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rusqlite::Connection;
use url::Url;

type Db = Mutex<Connection>;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreateRequest {
    slug: String,
    url: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CreateResponse {
    slug: String,
    url: String,
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").await.ok()
}

#[get("/links")]
fn list(db: &State<Db>) -> Json<HashMap<String, String>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT slug, url FROM links").unwrap();
    let map = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    Json(map)
}

#[post("/", data = "<req>")]
fn create(req: Json<CreateRequest>, db: &State<Db>) -> (Status, Json<CreateResponse>) {
    let parsed = Url::parse(&req.url).ok().filter(|u| matches!(u.scheme(), "http" | "https"));
    if parsed.is_none() {
        return (
            Status::UnprocessableEntity,
            Json(CreateResponse { slug: req.slug.clone(), url: req.url.clone() }),
        );
    }

    let conn = db.lock().unwrap();
    let inserted = conn
        .execute(
            "INSERT OR IGNORE INTO links (slug, url) VALUES (?1, ?2)",
            [&req.slug, &req.url],
        )
        .unwrap();

    if inserted == 0 {
        let existing_url: String = conn
            .query_row("SELECT url FROM links WHERE slug = ?1", [&req.slug], |row| {
                row.get(0)
            })
            .unwrap();
        return (
            Status::Conflict,
            Json(CreateResponse {
                slug: req.slug.clone(),
                url: existing_url,
            }),
        );
    }

    (
        Status::Created,
        Json(CreateResponse {
            slug: req.slug.clone(),
            url: req.url.clone(),
        }),
    )
}

#[get("/<slug>")]
fn redirect(slug: &str, db: &State<Db>) -> Result<Redirect, Status> {
    let conn = db.lock().unwrap();
    conn.query_row("SELECT url FROM links WHERE slug = ?1", [slug], |row| {
        row.get::<_, String>(0)
    })
    .map(|url| Redirect::to(url))
    .map_err(|_| Status::NotFound)
}

#[delete("/<slug>")]
fn delete(slug: &str, db: &State<Db>) -> Status {
    let conn = db.lock().unwrap();
    let deleted = conn
        .execute("DELETE FROM links WHERE slug = ?1", [slug])
        .unwrap();
    if deleted > 0 {
        Status::NoContent
    } else {
        Status::NotFound
    }
}

#[launch]
fn rocket() -> _ {
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "pqno.db".to_string());
    let conn = Connection::open(&db_path).expect("failed to open database");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS links (
            slug TEXT PRIMARY KEY,
            url  TEXT NOT NULL
        )",
    )
    .expect("failed to initialize database");

    rocket::build()
        .manage(Mutex::new(conn))
        .mount("/static", FileServer::from("static/"))
        .mount("/", routes![index, list, create, redirect, delete])
}
