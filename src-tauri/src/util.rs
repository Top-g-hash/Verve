mod calculator;
mod icons;
mod preferences;
mod search;

extern crate directories;
extern crate plist;

use auto_launch::AutoLaunchBuilder;
use calculator::calculate;
use directories::ProjectDirs;
use std::{process::Command, time::Instant};
use std::collections::HashMap;
use std::fs;

pub use icons::convert_all_app_icons_to_png;
pub use preferences::create_preferences_if_missing;
pub use search::{search, similarity_sort};

pub enum ResultType {
    Applications = 1,
    Files = 2,
    Calculation = 3,
}
#[tauri::command]
// Function to extract `Name`, `Icon`, and `Exec` fields from .desktop files
fn extract_desktop_entry(file_path: &str, selection: u8) -> Option<String> {
    // Read the contents of the file
    if let Ok(contents) = fs::read_to_string(file_path) {
        let mut app_details = HashMap::new();
        
        // Parse each line for relevant details
        for line in contents.lines() {
            if line.starts_with("Name=") {
                app_details.insert("Name", line["Name=".len()..].to_string());
            } else if line.starts_with("Exec=") {
                app_details.insert("Exec", line["Exec=".len()..].to_string());
            } else if line.starts_with("Icon=") {
                app_details.insert("Icon", line["Icon=".len()..].to_string());
            }
        }

        // Select the requested field based on the selection parameter
        match selection {
            1 => app_details.get("Name").cloned(),
            2 => app_details.get("Icon").cloned(),
            3 => app_details.get("Exec").cloned(),
            _ => None,
        }
    } else {
        None
    }
}
fn extract_names_from_desktop_entries(file_paths: Vec<String>) -> Vec<String> {
    let mut names = Vec::new();

    for file_path in file_paths {
        if let Some(name) = extract_desktop_entry(&file_path, 1) {
            names.push(name); // Add the name to the vector if it exists
        }
    }

    names
}

#[tauri::command]
pub async fn handle_input(input: String) -> (Vec<String>, f32, i32) {
    let mut result: Vec<String>;
    let mut result_type: ResultType;
    let start_time = Instant::now();
    if !input.starts_with("/") {
    result = search(
        input.as_str(),
        vec![
            "/var/lib/snapd/desktop/applications/",
            "/usr/share/applications/",
            "~/.local/share/applications/",
            "/var/lib/flatpak/exports/share/applications/"
        ],
        Some(".desktop"),  // Change this to search for .desktop files
        Some(1),           // Limit to 1 result (or adjust as needed)
    );
     result = extract_names_from_desktop_entries(result);
    similarity_sort(&mut result, input.as_str());
   
    result_type = ResultType::Applications;
} else {
        result = search(
            input.trim_start_matches("/"),
            vec!["/home"],
            None,
            Some(10000),
        );
        println!("{:?}", result);
        result_type = ResultType::Files;
    }
    if result.len() == 0 {
        let calculation_result = calculate(input.as_str());
        if calculation_result != "" {
            result.push(calculation_result);
            result_type = ResultType::Calculation;
        }
    }
    let time_taken = start_time.elapsed().as_secs_f32();
    return (result, time_taken, result_type as i32);
}

#[tauri::command]
pub fn get_icon(app_name: &str) -> String {
    if let Some(proj_dirs) = ProjectDirs::from("com", "parth jadhav", "verve") {
        let icon_dir = proj_dirs.config_dir().join("appIcons");
        let icon_path = icon_dir.join(app_name.to_owned() + &".png");
        if icon_path.exists() {
            return icon_path.to_str().unwrap().to_owned();
        }
        return String::from("");
    }
    return String::from("");
}

#[tauri::command]
pub fn open_command(path: &str) {
    Command::new("open")
        .arg(path.trim())
        .spawn()
        .expect("failed to execute process");
}

#[tauri::command]
pub fn launch_on_login(enable: bool) -> bool {
    let auto = AutoLaunchBuilder::new()
        .set_app_name("verve")
        .set_app_path("/Applications/verve.app")
        .build()
        .unwrap();

    if enable {
        match auto.enable() {
            Ok(_) => return true,
            Err(_) => {
                println!("Failed");
                false
            }
        }
    } else {
        match auto.disable() {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
}
