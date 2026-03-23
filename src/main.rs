mod alsa;
mod curve;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rme-cli")]
#[command(about = "Control RME Babyface Pro audio interface")]
#[command(after_help = "Run 'rme-cli <command> --help' for more information on a command.")]
#[command(long_about = "Control RME Babyface Pro audio interface

OUTPUT CHANNELS
  main, headphones
    volume  0-100 (integer)

INPUT CHANNELS
  mic1, mic2
    gain      0-65 dB (integer)
    phantom   on/off  — 48V phantom power
    pad       on/off

  line1, line2
    gain      0-9 dB (0.5 dB steps, e.g. 3.5)
    sensitivity  high (+4dBu) / low (-10dBV)

EXAMPLES
  rme-cli get main
  rme-cli set main 75%
  rme-cli set headphones +10%
  rme-cli get mic1 gain
  rme-cli set mic1 gain 32
  rme-cli set mic1 phantom on
  rme-cli set mic2 pad off
  rme-cli get line1 sensitivity
  rme-cli set line1 sensitivity high
  rme-cli set line2 gain 3.5")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read a channel value or input parameter
    #[command(after_help = "OUTPUTS
  main, headphones

INPUTS
  mic1, mic2, line1, line2

INPUT PARAMS (mic1, mic2)
  gain      0-65 dB
  phantom   on/off   48V phantom power
  pad       on/off

INPUT PARAMS (line1, line2)
  gain        0-9 dB in 0.5 steps (e.g. 3.5)
  sensitivity high (+4dBu) / low (-10dBV)")]
    Get {
        /// Output channel or input channel
        channel: String,
        /// Parameter to read (input channels only)
        parameter: Option<String>,
    },
    /// Write a channel value or input parameter
    #[command(after_help = "OUTPUTS
  main, headphones
    value  0-100%, relative: +5% / -5%

INPUTS
  mic1, mic2, line1, line2

INPUT PARAMS (mic1, mic2)
  gain      0-65 dB
  phantom   on/off   48V phantom power
  pad       on/off

INPUT PARAMS (line1, line2)
  gain        0-9 dB in 0.5 steps (e.g. 3.5)
  sensitivity high (+4dBu) / low (-10dBV)")]
    Set {
        /// Output channel or input channel
        channel: String,
        /// Output: volume (e.g. 75%, +10%, -5%)  —  Input: parameter name
        #[arg(allow_hyphen_values = true)]
        parameter_or_value: String,
        /// Parameter value (e.g. 32, 3.5, on, off, high, low)
        #[arg(allow_hyphen_values = true)]
        value: Option<String>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let card = alsa::find_babyface_card()?;

    match cli.command {
        Commands::Get { channel, parameter } => {
            let ch = channel.to_lowercase();
            match ch.as_str() {
                "main" | "main-out" | "speakers" => {
                    let raw = alsa::get_volume(&card, alsa::MAIN_OUT_LEFT)?;
                    println!("{:.0}%", curve::raw_to_percent(raw));
                }
                "headphones" | "hp" | "phones" => {
                    let raw = alsa::get_volume(&card, alsa::HEADPHONES_LEFT)?;
                    println!("{:.0}%", curve::raw_to_percent(raw));
                }
                "mic1" | "mic2" | "line1" | "line2" => {
                    let ctrl = parameter.ok_or_else(|| format!(
                        "Control required for '{}'. Use: {}",
                        ch, available_controls(ch.as_str())
                    ))?;
                    handle_input_get(&card, ch.as_str(), &ctrl)?;
                }
                _ => return Err(format!(
                    "Unknown channel '{}'. Use: main, headphones, mic1, mic2, line1, line2",
                    channel
                )),
            }
        }
        Commands::Set { channel, parameter_or_value, value } => {
            let ch = channel.to_lowercase();
            match ch.as_str() {
                "main" | "main-out" | "speakers" => {
                    let new_percent = parse_volume_value(&card, alsa::MAIN_OUT_LEFT, &parameter_or_value)?;
                    let new_raw = curve::percent_to_raw(new_percent);
                    alsa::set_volume(&card, alsa::MAIN_OUT_LEFT, new_raw)?;
                    alsa::set_volume(&card, alsa::MAIN_OUT_RIGHT, new_raw)?;
                    println!("{:.0}%", new_percent);
                }
                "headphones" | "hp" | "phones" => {
                    let new_percent = parse_volume_value(&card, alsa::HEADPHONES_LEFT, &parameter_or_value)?;
                    let new_raw = curve::percent_to_raw(new_percent);
                    alsa::set_volume(&card, alsa::HEADPHONES_LEFT, new_raw)?;
                    alsa::set_volume(&card, alsa::HEADPHONES_RIGHT, new_raw)?;
                    println!("{:.0}%", new_percent);
                }
                "mic1" | "mic2" | "line1" | "line2" => {
                    let val = value.ok_or_else(|| format!(
                        "Value required: rme-cli set {} {} <value>",
                        ch, parameter_or_value
                    ))?;
                    handle_input_set(&card, ch.as_str(), &parameter_or_value, &val)?;
                }
                _ => return Err(format!(
                    "Unknown channel '{}'. Use: main, headphones, mic1, mic2, line1, line2",
                    channel
                )),
            }
        }
    }

    Ok(())
}

fn available_controls(channel: &str) -> &'static str {
    match channel {
        "mic1" | "mic2" => "gain, phantom, pad",
        "line1" | "line2" => "gain, sensitivity",
        _ => "gain",
    }
}

fn input_gain_control(channel: &str) -> Result<&'static str, String> {
    match channel {
        "mic1" => Ok(alsa::MIC1_GAIN),
        "mic2" => Ok(alsa::MIC2_GAIN),
        "line1" => Ok(alsa::LINE1_GAIN),
        "line2" => Ok(alsa::LINE2_GAIN),
        _ => Err(format!("Unknown input channel: {}", channel)),
    }
}

fn phantom_control(channel: &str) -> Result<&'static str, String> {
    match channel {
        "mic1" => Ok(alsa::MIC1_PHANTOM),
        "mic2" => Ok(alsa::MIC2_PHANTOM),
        _ => Err(format!("'phantom' is not available for '{}'", channel)),
    }
}

fn pad_control(channel: &str) -> Result<&'static str, String> {
    match channel {
        "mic1" => Ok(alsa::MIC1_PAD),
        "mic2" => Ok(alsa::MIC2_PAD),
        _ => Err(format!("'pad' is not available for '{}'", channel)),
    }
}

fn sensitivity_control(channel: &str) -> Result<&'static str, String> {
    match channel {
        "line1" => Ok(alsa::LINE1_SENS),
        "line2" => Ok(alsa::LINE2_SENS),
        _ => Err(format!("'sensitivity' is not available for '{}'", channel)),
    }
}

fn handle_input_get(card: &str, channel: &str, control: &str) -> Result<(), String> {
    match control.to_lowercase().as_str() {
        "gain" => {
            let ctrl = input_gain_control(channel)?;
            let raw = alsa::get_volume(card, ctrl)?;
            if channel.starts_with("line") {
                println!("{}", raw as f64 / 2.0);
            } else {
                println!("{}", raw);
            }
        }
        "phantom" | "48v" => {
            let ctrl = phantom_control(channel)?;
            let state = alsa::get_switch(card, ctrl)?;
            println!("{}", if state { "on" } else { "off" });
        }
        "pad" => {
            let ctrl = pad_control(channel)?;
            let state = alsa::get_switch(card, ctrl)?;
            println!("{}", if state { "on" } else { "off" });
        }
        "sensitivity" | "sens" => {
            let ctrl = sensitivity_control(channel)?;
            let val = alsa::get_sensitivity(card, ctrl)?;
            println!("{}", val);
        }
        _ => return Err(format!(
            "Unknown control '{}' for '{}'. Use: {}",
            control, channel, available_controls(channel)
        )),
    }
    Ok(())
}

fn handle_input_set(card: &str, channel: &str, control: &str, value: &str) -> Result<(), String> {
    match control.to_lowercase().as_str() {
        "gain" => {
            let ctrl = input_gain_control(channel)?;
            let gain = parse_gain(value, channel)?;

            alsa::set_volume(card, ctrl, gain)?;

            // Setting input gain knocks the RME card in a weird state resulting in a volume change.
            // Nudge outputs to reset them to current volume.
            nudge_volume(card, alsa::MAIN_OUT_LEFT)?;
            nudge_volume(card, alsa::MAIN_OUT_RIGHT)?;
            nudge_volume(card, alsa::HEADPHONES_LEFT)?;
            nudge_volume(card, alsa::HEADPHONES_RIGHT)?;

            if channel.starts_with("line") {
                println!("{} dB", gain as f64 / 2.0);
            } else {
                println!("{} dB", gain);
            }
        }
        "phantom" | "48v" => {
            let ctrl = phantom_control(channel)?;
            let state = parse_switch_value(value)?;
            alsa::set_switch(card, ctrl, state)?;
            println!("{}", if state { "on" } else { "off" });
        }
        "pad" => {
            let ctrl = pad_control(channel)?;
            let state = parse_switch_value(value)?;
            alsa::set_switch(card, ctrl, state)?;
            println!("{}", if state { "on" } else { "off" });
        }
        "sensitivity" | "sens" => {
            let ctrl = sensitivity_control(channel)?;
            let sens = parse_sensitivity_value(value)?;
            alsa::set_sensitivity(card, ctrl, sens)?;
            println!("{}", sens);
        }
        _ => return Err(format!(
            "Unknown control '{}' for '{}'. Use: {}",
            control, channel, available_controls(channel)
        )),
    }
    Ok(())
}

/// Write a nudged value first, then the real value, to force the hardware to re-latch.
/// Read the current raw value and write it back with a one-step nudge first,
/// forcing the RME hardware to re-latch after an input gain change resets it.
/// Parse a gain value (dB) into a raw ALSA integer.
/// Mic inputs: 1 dB steps (raw == dB). Line inputs: 0.5 dB steps (raw == dB * 2).
/// Accepts both "3.5" and "3,5".
fn parse_gain(value: &str, channel: &str) -> Result<i32, String> {
    let db: f64 = value.replace(',', ".")
        .parse()
        .map_err(|_| format!("Invalid gain value '{}'. Use a number (e.g. 32 or 3.5)", value))?;
    if channel.starts_with("line") {
        let raw = (db * 2.0).round() as i32;
        if raw < 0 || raw > 18 {
            return Err(format!("Line gain must be between 0 and 9 dB (got {})", db));
        }
        Ok(raw)
    } else {
        let raw = db.round() as i32;
        if raw < 0 || raw > 65 {
            return Err(format!("Mic gain must be between 0 and 65 dB (got {})", db));
        }
        Ok(raw)
    }
}

fn nudge_volume(card: &str, control: &str) -> Result<(), String> {
    let raw = alsa::get_volume(card, control)?;
    let nudge = if raw > 0 { raw - 1 } else { 1 };
    alsa::set_volume(card, control, nudge)?;
    alsa::set_volume(card, control, raw)
}

fn parse_switch_value(value: &str) -> Result<bool, String> {
    match value.to_lowercase().as_str() {
        "on" | "1" | "true" | "yes" => Ok(true),
        "off" | "0" | "false" | "no" => Ok(false),
        _ => Err(format!("Invalid switch value '{}'. Use: on, off", value)),
    }
}

fn parse_sensitivity_value(value: &str) -> Result<&'static str, String> {
    match value.to_lowercase().as_str() {
        "+4dbu" | "high" | "hi" => Ok(alsa::SENS_HIGH),
        "-10dbv" | "low" | "lo" => Ok(alsa::SENS_LOW),
        _ => Err(format!(
            "Invalid sensitivity '{}'. Use: +4dBu/high or -10dBV/low",
            value
        )),
    }
}

/// Parse a volume value string and return the new percentage.
/// Supports: "50%" (absolute), "+5%" (relative up), "-5%" (relative down)
/// Parse a volume value string and return the new percentage (integer, 0–100).
/// Supports: "50%" (absolute), "+5%" (relative up), "-5%" (relative down)
fn parse_volume_value(card: &str, control: &str, value: &str) -> Result<f64, String> {
    let value = value.trim().trim_end_matches('%');

    let parse_int = |s: &str| -> Result<i32, String> {
        s.parse::<i32>()
            .map_err(|_| format!("Volume must be a whole number (got '{}')", s))
    };

    let validate = |v: i32| -> Result<f64, String> {
        if v < 0 || v > 100 {
            Err(format!("Volume must be between 0 and 100 (got {})", v))
        } else {
            Ok(v as f64)
        }
    };

    if let Some(delta) = value.strip_prefix('+') {
        let delta = parse_int(delta)?;
        let current_raw = alsa::get_volume(card, control)?;
        let current = curve::raw_to_percent(current_raw).round() as i32;
        Ok((current + delta).clamp(0, 100) as f64)
    } else if let Some(delta) = value.strip_prefix('-') {
        let delta = parse_int(delta)?;
        let current_raw = alsa::get_volume(card, control)?;
        let current = curve::raw_to_percent(current_raw).round() as i32;
        Ok((current - delta).clamp(0, 100) as f64)
    } else {
        validate(parse_int(value)?)
    }
}
