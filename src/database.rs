use std::fs::create_dir_all;

use crate::source::Source;
use crate::ui::Application;

use directories::ProjectDirs;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Connection, Sqlite, SqliteConnection};
use tracing::*;

#[macro_export]
macro_rules! db_version {
    () => {
        format!("sources-{}.db", &env!("CARGO_PKG_VERSION")[0..3])
    };
}

pub async fn establish_connection() -> Result<SqliteConnection, sqlx::Error> {
    let db_path = ProjectDirs::from("com", "tgz39", "saveit")
        .unwrap()
        .data_dir()
        .to_owned();

    // create DB path if it doesn't exist
    if !&db_path.exists() {
        debug!("Creating database directories...");
        create_dir_all(&db_path).expect("Error creating database directories");
    }

    // DB path + DB name
    let db_loc = format!(
        "sqlite://{}/{}",
        &db_path.to_str().unwrap().to_owned(),
        db_version!()
    );

    // create DB file if it doesn't exist
    if !Sqlite::database_exists(&db_loc).await.unwrap_or(false) {
        debug!("Creating database {}", &db_loc);

        match Sqlite::create_database(&db_loc).await {
            Ok(_) => {
                debug!("Successfully created database")
            }
            Err(e) => {
                error!("Error creating database: {}", e)
            }
        }
    }

    // connect to DB
    debug!("Establishing connection to database {}...", &db_loc);
    SqliteConnection::connect(&db_loc).await
}

pub async fn insert_source(source: &Source) -> Result<(), sqlx::Error> {
    let mut conn = establish_connection().await?;

    debug!("Inserting source into database: {:#?}", &source);

    sqlx::query("INSERT INTO sources (title, url, author, published_date, viewed_date, published_date_unknown, comment) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(&source.title)
        .bind(&source.url)
        .bind(&source.author)
        .bind(source.published_date)
        .bind(source.viewed_date)
        .bind(source.published_date_unknown)
        .bind(&source.comment)
        .execute(&mut conn)
        .await?;

    Ok(())
}

pub async fn get_all_sources() -> Result<Vec<Source>, sqlx::Error> {
    let mut conn = establish_connection().await?;

    sqlx::query_as::<_, Source>("SELECT * FROM sources")
        .fetch_all(&mut conn)
        .await
}

pub async fn delete_source(id: i64) -> Result<(), sqlx::Error> {
    debug!("Deleting source: {}", id);

    let mut conn = establish_connection().await?;

    let res = sqlx::query("DELETE FROM sources WHERE id = $1")
        .bind(id)
        .execute(&mut conn)
        .await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn update_source(id: i64, source: &Source) -> Result<(), sqlx::Error> {
    debug!("Updating source: {} to {:#?}", id, &source);

    let mut conn = establish_connection().await?;

    let res = sqlx::query("UPDATE sources SET title = $1, url = $2, author = $3, published_date = $4, viewed_date = $5, published_date_unknown = $6, comment = $7 WHERE id = $8")
        .bind(&source.title)
        .bind(&source.url)
        .bind(&source.author)
        .bind(source.published_date)
        .bind(source.viewed_date)
        .bind(source.published_date_unknown)
        .bind(&source.comment)
        .bind(id)
        .execute(&mut conn)
        .await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

// async delete source
pub fn handle_delete_source(id: i64, app: &Application) {
    let source_cache = app.sources_cache.clone();

    tokio::task::spawn(async move {
        delete_source(id).await.expect("Error deleting source");

        // update source cache
        *source_cache.write().unwrap() = get_all_sources().await.expect("Error loading sources");
    });
}

// async update source
pub fn handle_update_source(id: i64, source: &Source, app: &Application) {
    let source = source.clone();
    let source_cache = app.sources_cache.clone();

    tokio::task::spawn(async move {
        update_source(id, &source)
            .await
            .expect("Error deleting source");

        // update source cache
        *source_cache.write().unwrap() = get_all_sources().await.expect("Error loading sources");
    });
}

// async save source
pub fn handle_source_save(app: &Application) {
    let source = app.get_source();
    let source_cache = app.sources_cache.clone();

    tokio::task::spawn(async move {
        insert_source(&source)
            .await
            .expect("Error inserting source in database");

        // update source cache
        *source_cache.write().unwrap() = get_all_sources().await.expect("Error loading sources");
    });
}
