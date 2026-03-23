# RME Control CLI
### Simple cli wrapper for controling RME Babyface Pro

A CLI for controlling the RME Babyface Pro audio interface on Linux.

#### Usage
```
rme-cli get <channel> [parameter]
rme-cli set <channel> <parameter> <value>
```

Output channels: `main`, `headphones`
```
Set in 0-100 (with optional '%')

rme-cli get main
rme-cli set main 75%
rme-cli set headphones +10
```
> 0-100 percentage is translated with a exponential curve so percievable volume matches percentage as good as i could get it. 

**Mic inputs:** `mic1`, `mic2`.  
```
Parameters: 
gain      0-64 dB
phantom   on/off
pad       on/off

Example:
rme-cli set mic1 gain 32
rme-cli set mic1 phantom on
rme-cli set mic2 pad off
```


**Line inputs**: `line1`, `line2`  
Parameters: `gain`, `sensitivity`
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
```
cargo build --release
cp target/release/rme-cli ~/.local/bin/
```
