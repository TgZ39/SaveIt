use crate::DATABASE_NAME;
use directories::ProjectDirs;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Connection, FromRow, Sqlite, SqliteConnection};
use std::fs::create_dir_all;
use tracing::*;

#[derive(Debug, FromRow, Clone)]
pub struct Source {
    pub id: i64,
    pub url: String,
    pub author: String,
    pub date: chrono::NaiveDate,
}

impl Source {
    pub fn format(&self) -> String {
        format!(
            "- [{}]: Author: {}, URL: {} ({})",
            self.id,
            self.author,
            self.url,
            self.date.format("%d. %m. %Y")
        )
    }
}

pub async fn establish_connection() -> Result<SqliteConnection, sqlx::Error> {
    let db_path = ProjectDirs::from("com", "tgz39", "saveit")
        .unwrap()
        .data_dir()
        .to_owned();

    // create DB path if it doesn't exist
    if !&db_path.exists() {
        info!("Creating database directories...");
        create_dir_all(&db_path).expect("Error creating database directories.");
    }

    // DB path + DB name
    let db_loc = format!(
        "sqlite://{}/{}",
        &db_path.to_str().unwrap().to_owned(),
        DATABASE_NAME
    );

    // create DB file if it doesn't exist
    if !Sqlite::database_exists(&db_loc).await.unwrap_or(false) {
        info!("Creating database {}", &db_loc);

        match Sqlite::create_database(&db_loc).await {
            Ok(_) => {
                info!("Successfully created database.")
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

    info!("Inserting source into database: {:#?}", &source);

    sqlx::query("INSERT INTO sources (url, author, date) VALUES ($1, $2, $3)")
        .bind(&source.url)
        .bind(&source.author)
        .bind(source.date)
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
    info!("Deleting source: {}", id);

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
