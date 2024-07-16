use dialoguer::{theme::ColorfulTheme, Input};
use serde::{Deserialize, Serialize};

const DEFAULT_TAG: &str = "dummy";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configure {
    pub team: String,
    pub system_name: String,
}

impl Default for Configure {
    fn default() -> Configure {
        Configure {
            team: DEFAULT_TAG.to_string(),
            system_name: DEFAULT_TAG.to_string(),
        }
    }
}

impl Configure {
    pub fn questions() -> Self {
        let team: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("tags::team <string> =")
            .default(DEFAULT_TAG.to_string())
            .interact()
            .unwrap();
        let system_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("tags::system_name <string> =")
            .default(DEFAULT_TAG.to_string())
            .interact()
            .unwrap();
        Configure { team, system_name }
    }
}
