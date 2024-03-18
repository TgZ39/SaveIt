use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

const CONFIG_NAME: &str = "save-it";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ui_lang: String,
    pub source_lang: String,
    pub format_standard: FormatStandard,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui_lang: "en".to_string(),
            source_lang: "en".to_string(),
            format_standard: FormatStandard::Default,
        }
    }
}

impl Config {
    pub fn get_config() -> Self {
        confy::load(CONFIG_NAME, None).expect("Error loading config")
    }

    pub fn save(&self) {
        confy::store(CONFIG_NAME, None, self).expect("Error saving config");
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FormatStandard {
    Default,
    IEEE,
    APA,
}

impl Display for FormatStandard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatStandard::Default => write!(f, "Default"),
            FormatStandard::IEEE => write!(f, "IEEE"),
            FormatStandard::APA => write!(f, "APA"),
        }
    }
}
