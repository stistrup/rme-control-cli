# RME Babyface CLI
### Simple cli wrapper for controling RME Babyface Pro

A CLI for controlling the RME Babyface Pro audio interface on Linux using the controls exposed in ALSA. I've perpously left out the controls available for the Babyface as this is what i use. But if any of you use this tool, and would want those extra routing etc, create an issue and i'll probably include it.

Why use this instead of just useing the alsa controls? Well convenience is one, but there are some translation that makes this nicer to use. Volume 0-100, exponantiontially translated to better mimic how volume is percieved, translate the line inputs gain to dB instead of the 18 half dB steps (3.5 instead of 7 for instance).  

But maybe the most important is fixing a bug within the current drivers: When changing gain on any input, the headphones and main outputs changes the volume and gets stuck in a weird state, where you need to "nudge" those controls to make them jump back. (This can be verified in alsa mixer, set volume to loud, change gain, and hear volume go way down, nudge the volume again and it's back). This "nudging" is included in the CLI by changing volume by minimum amount and back. Not super pretty way of solving it, but couldn't think of anything better. If you have main output set to 100% and adjust gain, you will hear some "studdering", but to me it's an acceptable side-effect to not have to manually readjust main output and headphones after adjusting input gain. 

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
