use confy::ConfyError;
use serde::{Deserialize, Serialize};

pub const CONFIG_NAME: &str = "save-it";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ui_lang: String,
    pub source_lang: String,
    pub format_standard: FormatStandard,
    pub custom_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui_lang: "en".to_string(),
            source_lang: "en".to_string(),
            format_standard: FormatStandard::Default,
            custom_format: "CUSTOM FORMAT".to_string(),
        }
    }
}

impl Config {
    pub fn get_config() -> Self {
        let res: Result<Config, ConfyError> = confy::load(CONFIG_NAME, None);

        res.unwrap_or_else(|e| {
            if let ConfyError::BadTomlData(_) = e {
                let default = Config::default();

                confy::store(CONFIG_NAME, None, default).expect("Error resetting config");
                Self::get_config()
            } else {
                panic!("Error loading config: {}", &e)
            }
        })
    }

    pub fn save(&self) {
        confy::store(CONFIG_NAME, None, self).expect("Error saving config");
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FormatStandard {
    Default,
    Custom,
}
