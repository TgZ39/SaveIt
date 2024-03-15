use crate::DATABASE_NAME;
use directories::ProjectDirs;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Connection, FromRow, Sqlite, SqliteConnection};
use std::fs::create_dir_all;
use tracing::*;

#[derive(Debug, FromRow)]
pub struct Source {
    pub url: String,
    pub author: String,
    pub date: chrono::NaiveDate,
}

impl Source {
    pub fn format(&self) -> String {
        format!(
            "- Author: {}, URL: {} ({})",
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

    if !&db_path.exists() {
        info!("Creating database directories...");
        create_dir_all(&db_path).expect("Error creating database directories.");
    }

    let db_loc = format!(
        "sqlite://{}/{}",
        &db_path.to_str().unwrap().to_owned(),
        DATABASE_NAME
    );

    // create database if it doesn't exist
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

    // create connection
    debug!("Establishing connection to database {}...", &db_loc);
    SqliteConnection::connect(&db_loc).await
}

pub async fn insert_source(
    conn: &mut SqliteConnection,
    source: &Source,
) -> Result<(), sqlx::Error> {
    info!("Inserting source into database: {:#?}", &source);

    sqlx::query("INSERT INTO sources (url, author, date) VALUES ($1, $2, $3)")
        .bind(&source.url)
        .bind(&source.author)
        .bind(source.date)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn get_all_sources(conn: &mut SqliteConnection) -> Result<Vec<Source>, sqlx::Error> {
    sqlx::query_as::<_, Source>("SELECT * FROM sources")
        .fetch_all(conn)
        .await
}
