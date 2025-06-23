#![windows_subsystem = "windows"]

use chrono::Local;
use chrono::Timelike;
use notify_rust::Notification;
use std::{sync::mpsc, time::Duration};
use tray_item::{IconSource, TrayItem};
use directories::{ProjectDirs};
use std::fs::File;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Copy)]
enum Message {
    Quit,
    SaveSettings(Settings),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Settings {
    minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings{minutes: 15}
    }
}

fn load_settings() -> Settings {
    // Load config
    if let Some(proj_dirs) = ProjectDirs::from("ch", "otr",  "stretechy") {
        let config_dir = proj_dirs.config_dir();
        let file_content = match File::open(config_dir.join("settings.toml")) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                content
            }
            Err(_) => {
                // If the file doesn't exist, return default settings
                eprintln!("Error loading file: {}", config_dir.join("settings.toml").display());
                return Settings::default();
            }
        };

        match toml::from_str(&file_content) {
            Ok(settings) => {
                return settings;
            }
            Err(e) => {
                eprintln!("Error parsing settings file: {}", e);
                return Settings::default();
            }
        }
    }
    // For simplicity, we return default settings.
    Settings::default()
}

fn save_settings(s: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("ch", "otr",  "stretechy") {
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        let mut file = File::create(config_dir.join("settings.toml"))?;
        let content = toml::to_string(&s)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    } else {
        Err("Could not determine project directories".into())
    }
}

fn main() {
    

    let mut settings = load_settings();

    let mut tray = TrayItem::new(
        "Stretch timer",
        IconSource::Resource("aa-exe-icon"),
    ).unwrap();

    let next_tracker = tray.inner_mut().add_label_with_id("Next ").unwrap();

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let settings_15_tx = tx.clone();
    tray.add_menu_item("Stretch every 15 minutes", move || {
        settings_15_tx.send(Message::SaveSettings(
            Settings { minutes: 15 }
        )).unwrap();
    }).unwrap();

    let settings_20_tx = tx.clone();
    tray.add_menu_item("Stretch every 20 minutes", move || {
        settings_20_tx.send(Message::SaveSettings(
            Settings { minutes: 20 }
        )).unwrap();
    }).unwrap();

    let settings_30_tx = tx.clone();
    tray.add_menu_item("Stretch every 30 minutes", move || {
        settings_30_tx.send(Message::SaveSettings(
            Settings { minutes: 30 }
        )).unwrap();
    }).unwrap();

    let settings_60_tx = tx.clone();
    tray.add_menu_item("Stretch every hour", move || {
        settings_60_tx.send(Message::SaveSettings(
            Settings { minutes: 60 }
        )).unwrap();
    }).unwrap();

    tray.inner_mut().add_separator().unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    }).unwrap();



    loop {
        let minutes = settings.minutes;
        let seconds = minutes * 60;
        let now = Local::now().time();
        let seconds_in_quarter_hour = now.num_seconds_from_midnight() % seconds;
        let next_stretch_in_seconds: u64 = seconds as u64 - seconds_in_quarter_hour as u64;
        let next_stretch_time =
            (now + Duration::from_secs(next_stretch_in_seconds)).format("%H:%M");
        let next_stretch_text = format!("Next stretch at {next_stretch_time}");

        tray.inner_mut()
            .set_menu_item_label(&next_stretch_text, next_tracker)
            .unwrap();
        // We sleep unless we get a kill signal.
        match rx.recv_timeout(Duration::from_secs(next_stretch_in_seconds)) {  
            // Sleep was interrupted
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }

            Ok(Message::SaveSettings(new_settings)) => {
                settings = new_settings;
                if let Err(e) = save_settings(&settings) {
                    eprintln!("Error saving settings: {}", e);
                } else {
                    println!("Settings saved successfully.");
                }
            }

            Err(_) => {
                Notification::new()
                .appname("Stretchy")
                .summary("Time for a stretch!")
                .body("It's time to stretch your back, shoulders and stuffens.")
                .icon("firefox")
                .show()
                .unwrap();
            }
        }
    }
}
