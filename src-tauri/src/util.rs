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
use std::io;
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
fn extract_desktop_entry(file_path: &str, selection: u8) -> Option<String> {
    // Read the contents of the file
    if let Ok(contents) = fs::read_to_string(file_path) {
        // Parse each line and return early when the requested field is found
        for line in contents.lines() {
            match selection {
                1 if line.starts_with("Name=") => {
                    return Some(line["Name=".len()..].to_string());
                }
                2 if line.starts_with("Icon=") => {
                    return Some(line["Icon=".len()..].to_string());
                }
                3 if line.starts_with("Exec=") => {
                    return Some(line["Exec=".len()..].to_string());
                }
                _ => continue,
            }
        }
    }
    None
}


#[tauri::command]
pub async fn extract_name_from_desktop_entry(file_path: String) -> String {
    if let Some(name) = extract_desktop_entry(&file_path, 1) {
        println!("Extracted names: {}", name);
        return name; // Return the name if it exists
        
    }

    "".to_string() // Return "Unknown" if no name is found
}
#[tauri::command]
pub async fn extract_icon_from_desktop_entry(file_path: String) -> String {
    // Attempt to extract the icon from the desktop entry
    if let Some(icon) = extract_desktop_entry(&file_path, 2) { 
        println!("Extracted icon: {}", icon);
        return icon; // Return the icon if it exists
    }

    // Return "Unknown" if no icon is found
    "Unknown".to_string()
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
    
    //  result = extract_names_from_desktop_entries(result);
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
pub async fn execute_desktop_file(desktop_file_path: &str) -> Result<(), String> {
    // Log the command being executed for debugging purposes
    println!("Executing command from desktop file: {}", desktop_file_path);

    // Extract the command from the specified line (e.g., Exec line)
    let command = extract_desktop_entry(desktop_file_path, 3)
        .ok_or_else(|| "Failed to get command from the desktop entry".to_string())?;

    // Split command and its arguments (if any)
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or_else(|| "No program found".to_string())?;
    let args: Vec<&str> = parts.collect();

    // Execute the command directly, instead of using xdg-open
    let mut child = Command::new(program)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    // Wait for the command to finish
    child.wait()
        .map_err(|e| format!("Command execution failed: {}", e))?;

    Ok(())
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
