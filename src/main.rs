#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod database;
mod ui;

use crate::database::establish_connection;
use crate::ui::open_gui;

use tracing::{info, Level};

const DATABASE_NAME: &str = "sources.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup logging
    let subscriber = tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(false)
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    // setup database
    info!("Executing database migrations...");
    let mut conn = establish_connection()
        .await
        .expect("Error connection to database.");
    sqlx::migrate!("./migrations")
        .run(&mut conn)
        .await
        .expect("Error executing database migrations.");

    info!("Opening GUI");
    open_gui().expect("Error opening GUI");

    Ok(())
}
