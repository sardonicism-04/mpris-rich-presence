# MPRIS RP
A program made to run in the background and update a Discord rich presence with data from MPRIS-compatible players.

## Usage
The most basic way to run the program is to simply invoke it with `mpris-rp`. However, this does block the terminal. As such, the best way to run it is using a process manager such as `systemd`.

An example systemd service file:
```ini
[Unit]
Description=Discord Rich Presence for MPRIS players

[Service]
Type=simple
ExecStart=%h/.cargo/bin/mpris-rp
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

## Installation
Requires the latest Rust compiler:
```sh
cargo install --git https://github.com/sardonicism-04/mpris-rp --branch main mpris-rp
```

Or, building from source:
```sh
git clone https://github.com/sardonicism-04/mpris-rp.git # via HTTPS
git clone git@github.com:sardonicism-04/mpris-rp.git     # via SSH

cd mpris-rp

cargo build
```