use handlebars::Handlebars;
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fs;

pub mod module_echo;
pub mod tags;

const CONFIG_FILE_WRITE_PATH: &str = "src";
const CONFIG_FILE_NAME: &str = ".platform-config.json";
const TERRAFORM_CACHE_DIR: &str = ".cache";
const TERRAFORM_PLAN_FILE: &str = "tfplan.tfout";

static TEMPLATE_DIR: Dir = include_dir!("./src/templates");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct System {
    pub name: String,
    pub tags: tags::Configure,
    pub module_echo: module_echo::Configure,
}

impl System {
    pub fn new(name: String, tags: tags::Configure, module_echo: module_echo::Configure) -> System {
        System {
            name,
            tags,
            module_echo,
        }
    }
}

pub fn usage(command: String) -> Result<String, Option<String>> {
    let command = &*command;

    match command {
        "init" => {
            println!(" --> init: Initialize a new system configuration");
            return Ok(String::from("init"));
        }
        "plan" => {
            println!(" --> plan: Plan the system configuration");
            return Ok(String::from("plan"));
        }
        "apply" => {
            println!(" --> apply: Apply the system configuration");
            return Ok(String::from("apply"));
        }
        _ => {
            println!("Usage: system [init|plan|apply]");
            return Err(Some(String::from("Invalid command")));
        }
    }
}

pub fn format_output(system: &System) -> Result<String, Option<String>> {
    let serialized = serde_json::to_string(&system).unwrap();
    Ok(serialized)
}

pub fn write_output(folder_name: String, json: String) -> std::io::Result<()> {
    let file_folder = format!("{}/{}", CONFIG_FILE_WRITE_PATH, folder_name);
    let cache_folder = format!("{}/{}", file_folder, TERRAFORM_CACHE_DIR);
    let file_path = format!("{}/{}", file_folder, CONFIG_FILE_NAME);
    fs::create_dir_all(cache_folder)?;
    std::fs::write(file_path, json)?;
    Ok(())
}

pub fn read_config_file(system_name: String) -> Result<String, Box<dyn StdError>> {
    let file_path = format!(
        "{}/{}/{}",
        CONFIG_FILE_WRITE_PATH, system_name, CONFIG_FILE_NAME
    );
    let contents = fs::read_to_string(file_path)?;
    Ok(contents)
}

pub fn render_template(system_name: &str, template_name: &str, json: &str) -> std::io::Result<()> {
    let json: serde_json::Value = serde_json::from_str(json).expect("Error parsing JSON");
    let template_file = TEMPLATE_DIR
        .get_file(template_name)
        .expect("Template not found");
    let template_content = template_file
        .contents_utf8()
        .expect("Error reading template file");
    let mut handlebars = Handlebars::new();
    let _ = handlebars.register_template_string("template", template_content);
    let rendered = handlebars
        .render("template", &json)
        .expect("Error rendering template");
    let output_file = format!(
        "{}/{}/{}/{}",
        CONFIG_FILE_WRITE_PATH, system_name, TERRAFORM_CACHE_DIR, "main.tf"
    );
    write_rendered_template_file(&output_file, &rendered)
}

fn write_rendered_template_file(file_path: &str, rendered: &str) -> std::io::Result<()> {
    println!("\nWriting rendered template to {}", file_path);
    std::fs::write(file_path, rendered)?;
    Ok(())
}

pub fn is_terraform_present() -> Option<bool> {
    let _output = std::process::Command::new("terraform")
        .arg("--version")
        .output()
        .expect("\n\nFailed to execute terraform --version command\n Be sure to have terraform installed and in your PATH\n\n");
    Some(true)
}

pub fn terraform_plan(system_name: &str) -> std::io::Result<()> {
    println!("\nRunning terraform plan...");
    println!("\nInitializing terraform...");
    let _output = std::process::Command::new("terraform")
        .arg("init")
        .current_dir(format!(
            "{}/{}/{}",
            CONFIG_FILE_WRITE_PATH, system_name, TERRAFORM_CACHE_DIR
        ))
        .output()
        .expect("Failed to execute terraform init command");
    let tfoutput = format!("-out={}", TERRAFORM_PLAN_FILE);
    println!("\nPlanning terraform...");
    let output = std::process::Command::new("terraform")
        .args(["plan", tfoutput.as_str()])
        .current_dir(format!(
            "{}/{}/{}",
            CONFIG_FILE_WRITE_PATH, system_name, TERRAFORM_CACHE_DIR
        ))
        .output()
        .expect("Failed to execute terraform plan command");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub fn terraform_show(system_name: &str) -> std::io::Result<()> {
    println!("\nShowing terraform plan...");
    let output = std::process::Command::new("terraform")
        .args(["show", TERRAFORM_PLAN_FILE])
        .current_dir(format!(
            "{}/{}/{}",
            CONFIG_FILE_WRITE_PATH, system_name, TERRAFORM_CACHE_DIR
        ))
        .output()
        .expect("\nFailed to execute terraform show command\n\n");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub fn terraform_apply(system_name: &str) -> std::io::Result<()> {
    println!("\nPlease confirm below plan!\n");
    println!("\nApplying terraform...");
    let output = std::process::Command::new("terraform")
        .args(["apply", TERRAFORM_PLAN_FILE])
        .current_dir(format!(
            "{}/{}/{}",
            CONFIG_FILE_WRITE_PATH, system_name, TERRAFORM_CACHE_DIR
        ))
        .output()
        .expect("Failed to execute terraform apply command");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON_TEST: &'static str = r#"{"name":"test-system","tags":{"team":"tetam-tes","system_name":"system-test"},"module_echo":{"foo":99,"bar":"super-bar"}}"#;

    #[test]
    fn test_usage() {
        let command = String::from("init");
        let result = usage(command);
        assert_eq!(result, Ok(String::from("init")));
    }

    #[test]
    fn test_format_output() {
        let json: System = serde_json::from_str(&JSON_TEST).expect("Error parsing JSON");
        let result = format_output(&json);
        assert_eq!(
            result,
            Ok(String::from(
               "{\"name\":\"test-system\",\"tags\":{\"team\":\"tetam-tes\",\"system_name\":\"system-test\"},\"module_echo\":{\"foo\":99,\"bar\":\"super-bar\"}}"
            ))
        );
    }
}
