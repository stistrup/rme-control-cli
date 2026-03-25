# RME Babyface CLI
### Simple cli wrapper for controling RME Babyface Pro

A CLI for controlling the RME Babyface Pro audio interface on Linux using the controls exposed in ALSA. I've perpously left out some controls available for the Babyface only done the ones i use. If you use this tool, and would want those extra routing remaining 8 inputs etc, create an issue and i'll probably include it.

Might seem unnecessary to do a CLI for something thats available as ALSA controls. But point of me doing it was:
- Finds the card for you
- Nicer commands
- Translates main output volumes to percieved volume which makes it nicer to use
- Translates line input gain to represent dB (0-9) instead of those halv db steps (0-18). You can still set half db steps with "3.5" for example
- Compensates for bug where adjusting gain changes volume and keeps it there until you adjust it. The workaround nudges the volume back and forth with smallest step possible so it goes back to where you had it before adjusting gain.
> If you don't know what bug i'm talking about: open alsa mixer, set main out to over 70%, adjust input gain of any of the first 4 inputs and you'll hear volume goes down until you touch both main outs. Applies to headphones too.

#### Usage
```
rme-cli get <channel> [parameter]
rme-cli set <channel> <parameter> <value>
```

Output channels: `main`, `headphones`
```
Set in 0-100 (with optional '%')
Either in absolute or with +/-

rme-cli get main
rme-cli set main 75%
rme-cli set headphones +10
```

**Mic inputs:** `mic1`, `mic2`.  
```
Parameters: 
gain      0-65 dB
phantom   on/off
pad       on/off

Example:
rme-cli set mic1 gain 32
rme-cli set mic1 phantom on
rme-cli set mic2 pad off
```


**Line inputs**: `line1`, `line2`  
```
Parameters:
gain          0-9 dB (supports 0.5 incraments)
sensitivity   low/high (-10dBv/+4dBu)

Example:
rme-cli set line1 gain 3.5
rme-cli set line1 sensitivity high
rme-cli set line2 sensitivity low
```

## Installation
Requires rust installed
```
cargo build --release
cp target/release/rme-cli ~/.local/bin/
```
