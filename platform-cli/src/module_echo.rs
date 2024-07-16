use dialoguer::{theme::ColorfulTheme, Input};
use serde::{Deserialize, Serialize};

const DEFAULT_FOO: usize = 5;
const DEFAULT_BAR: &str = "hello";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configure {
    pub foo: usize,
    pub bar: String,
}

impl Default for Configure {
    fn default() -> Configure {
        Configure {
            foo: DEFAULT_FOO,
            bar: DEFAULT_BAR.to_string(),
        }
    }
}

impl Configure {
    pub fn questions() -> Self {
        let foo: usize = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("echo::foo <integer> =")
            .default(DEFAULT_FOO)
            .interact()
            .unwrap();
        let bar: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("echo::bar <string> =")
            .default(DEFAULT_BAR.to_string())
            .interact()
            .unwrap();
        Configure { foo, bar }
    }
}
