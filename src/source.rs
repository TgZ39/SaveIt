use crate::config::{Config, FormatStandard};
use chrono::{Local, NaiveDate};
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct Source {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub author: String,
    pub published_date: NaiveDate,
    pub viewed_date: NaiveDate,
    pub published_date_unknown: bool,
    pub comment: String,
}

impl Source {
    pub fn format(&self, standard: &FormatStandard) -> String {
        match standard {
            FormatStandard::Default => {
                let mut out = String::new();

                out.push_str(format!("[{}]", self.id).as_str());

                match self.author.is_empty() {
                    true => out.push_str(" Unbekannt"),
                    false => out.push_str(format!(" {}", self.author).as_str()),
                }

                if !self.published_date_unknown {
                    out.push_str(format!(" ({})", self.published_date.format("%Y")).as_str());
                }

                out.push_str(
                    format!(
                        ": {} URL: {} [Stand: {}]",
                        self.title,
                        self.url,
                        self.viewed_date.format("%d. %m. %Y")
                    )
                    .as_str(),
                );

                out
            }
            FormatStandard::Custom => {
                let config = Config::get_config();

                let mut out = config.custom_format;

                out = out.replace("{INDEX}", &self.id.to_string());
                out = out.replace("{TITLE}", &self.title);
                out = out.replace("{URL}", &self.url);
                out = out.replace("{AUTHOR}", &self.author);
                out = out.replace("{P_DATE}", &self.published_date.format("%Y").to_string());
                out = out.replace(
                    "{V_DATE}",
                    &self.viewed_date.format("%d. %m. %Y").to_string(),
                );

                out
            }
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
            viewed_date: chrono::NaiveDate::from(Local::now().naive_local()),    // current date
            published_date_unknown: false,
            comment: String::new(),
        }
    }
}
