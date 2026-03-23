use regex::Regex;
use std::fs;
use std::process::Command;

pub fn find_babyface_card() -> Result<String, String> {
    let cards_path = "/proc/asound/cards";
    let cards = fs::read_to_string(cards_path)
        .map_err(|e| format!("Failed to read {}: {}", cards_path, e))?;

    for (i, line) in cards.lines().enumerate() {
        if i % 2 == 0 {
            if let Some(card_index) = line.split_whitespace().next() {
                let usb_id_path = format!("/proc/asound/card{}/usbid", card_index);
                if let Ok(usb_id) = fs::read_to_string(&usb_id_path) {
                    // RME Babyface Pro (class compliant)
                    if usb_id.trim() == "2a39:3fb0" {
                        return Ok(card_index.to_string());
                    }
                }
            }
        }
    }

    Err("No RME Babyface Pro found".to_string())
}

pub fn get_volume(card_index: &str, control_name: &str) -> Result<i32, String> {
    let output = Command::new("amixer")
        .args(["-c", card_index, "get", control_name])
        .output()
        .map_err(|e| format!("Failed to execute amixer: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_value_from_amixer_output(&stdout)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn set_volume(card_index: &str, control_name: &str, volume: i32) -> Result<(), String> {
    let output = Command::new("amixer")
        .args(["-c", card_index, "set", control_name, &volume.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute amixer: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn parse_value_from_amixer_output(output: &str) -> Result<i32, String> {
    let re = Regex::new(r"(?m)^\s*Mono:( Playback)? (\d+)").unwrap();

    match re.captures(output) {
        Some(caps) => {
            let volume_str = caps.get(caps.len() - 1).unwrap().as_str();
            volume_str
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse volume: {}", e))
        }
        None => Err(format!("Could not parse volume from amixer output:\n{}", output)),
    }
}

pub const MAIN_OUT_LEFT: &str = "Main-Out AN1";
pub const MAIN_OUT_RIGHT: &str = "Main-Out AN2";

pub const HEADPHONES_LEFT: &str = "Main-Out PH3";
pub const HEADPHONES_RIGHT: &str = "Main-Out PH4";

pub const MIC1_GAIN: &str = "Mic-AN1 Gain";
pub const MIC1_PHANTOM: &str = "Mic-AN1 48V";
pub const MIC1_PAD: &str = "Mic-AN1 PAD";

pub const MIC2_GAIN: &str = "Mic-AN2 Gain";
pub const MIC2_PHANTOM: &str = "Mic-AN2 48V";
pub const MIC2_PAD: &str = "Mic-AN2 PAD";

pub const LINE1_GAIN: &str = "Line-IN3 Gain";
pub const LINE1_SENS: &str = "Line-IN3 Sens.";

pub const LINE2_GAIN: &str = "Line-IN4 Gain";
pub const LINE2_SENS: &str = "Line-IN4 Sens.";

pub const SENS_HIGH: &str = "+4dBu";
pub const SENS_LOW: &str = "-10dBV";

pub fn get_switch(card_index: &str, control_name: &str) -> Result<bool, String> {
    let output = Command::new("amixer")
        .args(["-c", card_index, "get", control_name])
        .output()
        .map_err(|e| format!("Failed to execute amixer: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Mono:") {
                return Ok(line.contains("[on]"));
            }
        }
        Err(format!("Could not parse switch state for '{}'", control_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn set_switch(card_index: &str, control_name: &str, state: bool) -> Result<(), String> {
    let state_str = if state { "on" } else { "off" };
    let output = Command::new("amixer")
        .args(["-c", card_index, "set", control_name, state_str])
        .output()
        .map_err(|e| format!("Failed to execute amixer: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn get_sensitivity(card_index: &str, control_name: &str) -> Result<String, String> {
    let output = Command::new("amixer")
        .args(["-c", card_index, "get", control_name])
        .output()
        .map_err(|e| format!("Failed to execute amixer: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim_start().starts_with("Item0:") {
                let val = line.split(':').nth(1)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('\'')
                    .to_string();
                return Ok(val);
            }
        }
        Err(format!("Could not parse sensitivity for '{}'", control_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn set_sensitivity(card_index: &str, control_name: &str, value: &str) -> Result<(), String> {
    let output = Command::new("amixer")
        .args(["-c", card_index, "sset", control_name, "--", value])
        .output()
        .map_err(|e| format!("Failed to execute amixer: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
