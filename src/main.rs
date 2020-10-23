use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};

use dotenv::dotenv;
use log::*;
use std::env;

use serde::Deserialize;
use sqlx::SqlitePool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // The sqlx query! macro uses environment value at compile time,
    // So you need to export it anyway before compilation, dotenv is not enough.
    dotenv().ok();
    env_logger::init();

    let db_pool = create_pool().await;
    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            // We allow the visitor to see an index of the images at `/images`.
            .service(web::resource("/clients").route(web::get().to(list_clients)))
            .service(Files::new("/images", "static/images/").show_files_listing())
            .service(web::resource("/").route(web::post().to(handle_form)))
            // Serve a tree of static files at the web root and specify the index file.
            // Note that the root path should always be defined as the last item. The paths are
            // resolved in the order they are defined. If this would be placed before the `/images`
            // path then the service for the static images would never be reached.
            .service(Files::new("/", "./static/root/").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn create_pool() -> SqlitePool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    SqlitePool::new(&database_url)
        .await
        .expect("Couldn't create sqlite pool!")
}

#[derive(Deserialize)]
struct ClientForm {
    name: String,
}

async fn handle_form(pool: web::Data<SqlitePool>, form: web::Form<ClientForm>) -> HttpResponse {
    let mut conn = pool
        .acquire()
        .await
        .expect("Couldn't acquire connection from the pool");

    sqlx::query!(
        r#"
INSERT INTO clients ( name )
VALUES ( ?1 )
        "#,
        form.name
    )
    .execute(&mut conn)
    .await
    .expect("Couldn't execute sql query!");

    info!("Added a client: {}", form.name);
    HttpResponse::Ok().json(format!("Thanks, {}", form.name))
}

async fn list_clients(pool: web::Data<SqlitePool>) -> HttpResponse {
    let mut conn = pool
        .acquire()
        .await
        .expect("Couldn't acquire connection from the pool");

    let clients = sqlx::query!(
        r#"
SELECT id, name
FROM clients
ORDER BY id
        "#
    )
    .fetch_all(&mut conn)
    .await
    .expect("Couldn't execute sql query!");

    let mut message: String = "Clients are: ".to_string();
    for client in clients {
        message = format!("{}{}. {}; ", message, client.id, client.name);
    }

    HttpResponse::Ok().json(message)
}
