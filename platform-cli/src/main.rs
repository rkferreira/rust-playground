use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use platform_cli::*;
use std::env;

const ENABLED_TEMPLATES: [&str; 1] = ["module_echo"];
const TEMPLATE_SUFFIX: &str = ".hbs";

// Init functionality
fn init() {
    show_header();
    println!("I will ask you some questions to get started.\n");

    // Get system name
    let system_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is your system name?")
        .allow_empty(false)
        .interact()
        .unwrap();

    // Configure tags
    println!("\nLets configure --> tags <-- for {} system.", system_name);
    println!("Please input each tag value for corresponding tag name.\n");
    let tags: tags::Configure = tags::Configure::questions();

    // Configure module echo
    println!(
        "\nLets configure --> module echo <-- for {} system.",
        system_name
    );
    println!("Please input each value for corresponding echo module variables.\n");
    let module_echo: module_echo::Configure = module_echo::Configure::questions();

    // Put all together on a System
    let result = System::new(system_name.clone(), tags, module_echo);

    let json = format_output(&result).unwrap();
    let _ = match write_output(system_name, json) {
        Ok(_) => println!("\nSystem configuration saved successfully.\n"),
        Err(e) => panic!("\nError saving configuration file!\n\n {}", e),
    };
}

fn plan() {
    show_header();

    println!("--> You are on PLAN phase <--\n");

    // Check if terraform binary is present
    if let Some(_) = is_terraform_present() {
        println!("    ::terraform is present\n\n");
    }

    // Get system name
    let system_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is your system name?")
        .allow_empty(false)
        .interact()
        .unwrap();

    let json =
        read_config_file(system_name.clone()).expect("\nError reading configuration file.\n\n");

    for template in ENABLED_TEMPLATES.iter() {
        let template = &*(format!("{}{}", template, TEMPLATE_SUFFIX));
        if let Ok(_) = render_template(&*(system_name.clone()), template, &json) {
            println!("\nTemplate {} rendered successfully.\n", template);
        } else {
            println!("\nError rendering template {}.\n", template);
        }
    }

    if let Ok(_) = terraform_plan(&*system_name) {
        println!("\nTerraform plan executed successfully.\n");
    } else {
        println!("\nError executing terraform plan.\n");
    }
}

fn apply() {
    show_header();

    println!("--> You are on APPLY phase <--\n");

    // Check if terraform binary is present
    if let Some(_) = is_terraform_present() {
        println!("    ::terraform is present\n\n");
    }

    // Get system name
    let system_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is your system name?")
        .allow_empty(false)
        .interact()
        .unwrap();

    if let Ok(_) = terraform_show(&*system_name) {
        println!("\nTerraform show executed successfully.\n\n");
    } else {
        println!("\nError executing terraform show.\n");
    }

    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to apply?")
        .interact()
        .unwrap();

    if confirmation {
        if let Ok(_) = terraform_apply(&*system_name) {
            println!("\nTerraform apply executed successfully.\n");
        } else {
            println!("\nError executing terraform apply.\n");
        }
    } else {
        println!("\nTerraform apply cancelled.\n");
    }
}

fn show_header() {
    print!("{}[2J", 27 as char);
    println!("=====================================");
    println!("Welcome to awesome Platform CLI tool!");
    println!("=====================================\n\n");
}

// Main program
fn main() {
    let args: Vec<String> = env::args().collect();
    let command: String;

    // Check if command is provided
    if args.len() == 2 {
        let cmd = usage(args[1].clone());
        match cmd {
            Ok(_) => command = cmd.unwrap(),
            Err(e) => {
                println!("Error: {}", e.unwrap());
                return;
            }
        }
    } else {
        println!("No arguments provided.\n\n  Usage: system [init|plan|apply] \n");
        return;
    }

    match &*command {
        "init" => init(),
        "plan" => plan(),
        "apply" => apply(),
        _ => println!("Invalid command provided."),
    }
}
