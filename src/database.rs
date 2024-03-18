use std::fs::create_dir_all;

use chrono::{Local, NaiveDate};
use directories::ProjectDirs;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Connection, FromRow, Sqlite, SqliteConnection};
use tracing::*;
use crate::config::FormatStandard;

use crate::DATABASE_NAME;

#[derive(Debug, FromRow, Clone)]
pub struct Source {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub author: String,
    pub published_date: chrono::NaiveDate,
    pub viewed_date: chrono::NaiveDate,
}

impl Source {
    pub fn format(&self, standard: &FormatStandard) -> String { // TODO update this
        match standard {
            FormatStandard::Default => {

                let mut out = String::new();

                out.push_str(format!("[{}]", self.id).as_str());

                match self.author.len() == 0 {
                    true => { out.push_str(" Unbekannt") }
                    false => { out.push_str(format!(" {}", self.author ).as_str())}
                }

                if self.published_date != NaiveDate::MIN {
                    out.push_str(format!(" ({})", self.published_date.format("%Y")).as_str());
                }

                out.push_str(format!(": {} URL: {} [Stand: {}]", self.title, self.url, self.viewed_date.format("%d. %m. %Y")).as_str());

                return out;
            },
            FormatStandard::IEEE => { todo!() },
            FormatStandard::APA => { todo!() },
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self {
            id: -1,
            title: String::new(),
            author: String::new(),
            url: String::new(),
            published_date: chrono::NaiveDate::from(Local::now().naive_local()), // current date
            viewed_date: chrono::NaiveDate::from(Local::now().naive_local()), // current date
        }
    }
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
        DATABASE_NAME
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

    sqlx::query("INSERT INTO sources (title, url, author, published_date, viewed_date) VALUES ($1, $2, $3, $4, $5)")
        .bind(&source.title)
        .bind(&source.url)
        .bind(&source.author)
        .bind(source.published_date)
        .bind(source.viewed_date)
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

    let res = sqlx::query("UPDATE sources SET title = $1, url = $2, author = $3, published_date = $4, viewed_date = $5 WHERE id = $6")
        .bind(&source.title)
        .bind(&source.url)
        .bind(&source.author)
        .bind(source.published_date)
        .bind(source.viewed_date)
        .bind(id)
        .execute(&mut conn)
        .await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
