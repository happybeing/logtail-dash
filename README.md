[![crates.io](http://meritbadge.herokuapp.com/logtail)](https://crates.io/crates/logtail)

# Terminal Dashboard for Monitoring Log Files

`logtail` displays one or more log files in the terminal in the manner of `tail -f`.

Features of `logtail`:
- it displays more than one logfile, one above the other or side-by-side
- the display updates as each logfile grows
- use tab or arrow keys to navigate and scroll
- you can fork **logtail-dash** to create customised views of your logfile data

`logtail` is written in Rust and uses [tui-rs](https://github.com/fdehau/tui-rs) to create the terminal UI, and [linemux](https://github.com/jmagnuson/linemux) to monitor the logfiles.

## Operating Systems
- **Linux:** works on Ubuntu.
- **Windows:** works on Windows 10.
- **MacOS:** let me know what happens!

## Install from crates.io
1 Install **Rust** via https://doc.rust-lang.org/cargo/getting-started/installation.html

2 **Linux (Ubuntu)**

    sudo apt-get install build-essential

3a. **Linux/MacOS** install **logtail:**

    cargo install logtail
    logtail --help

3b. **Windows** install **logtail-crossterm:**

To build on Windows you must use the 'nightly' compiler until the 'itarget' feature becomes part of 'stable', so install Rust nightly using `rustup`:

    rustup toolchain install nightly
    
To build `logtail-crossterm` on Windows, clone logtail-dash, build with `+nightly` and use the binary it creates under `./taget/release`:

    git clone https://github.com/happybeing/logtail-dash
    cd logtail-dash
    cargo +nightly build -Z features=itarget --bin logtail-crossterm --release --no-default-features

    ./target/release/logtail-crossterm --help

Note: `vdash` is a fork of `logtail` that provides a dashboard for SAFE Network Vaults (see [vdash](https://github.com/happybeing/vdash)).

## Usage

In the terminal type the command and the paths of one or more logfiles you want to monitor. For example:

    logtail /var/log/auth.log /var/log/kern.log

When the dashboard is active, pressing 'v' or 'h' switches between horizontal and vertical arrangments (when viewing more than one logfile).

For more information:

    logtail --help

## Customised Logfile Dashboards

If you want to use the core functionality of logtail-dash to create
a customised terminal display based on real time updates to files,
you can do this by creating a fork and customising the files in src/custom:

`src/custom/opt.rs`:  command line options and usage

`src/custom/app.rs`:  application logic (e.g. parsing logfiles to `dashboard state)

`src/custom/ui.rs `:    dashboard display and keyboard/mouse interface

Example: `vdash` is a fork of `logtail` that provides a dashboard for SAFE Network Vaults (see [vdash](https://github.com/happybeing/vdash)).

## Build
### Get pre-requisites
1. **Get Rust:** see: https://doc.rust-lang.org/cargo/getting-started/installation.html

### Get code
```
git clone https://github.com/happybeing/logtail-dash
cd logtail-dash
```

### Build

#### Linux / MacOS
Builds logtail which uses the termion backend (see [tui-rs](https://github.com/fdehau/tui-rs)).
Note: MacOS is untested
```
cargo build --bin logtail --features="termion" --release
```

#### Windows 10
Builds logtail-crossterm which uses the crossterm backend (see [tui-rs](https://github.com/fdehau/tui-rs)), with the intention to support Windows.

NOT working on Windows yet, this is being worked on at the moment. Help with testing appreciated.
```
cargo build --bin logtail-crossterm --features="crossterm" --release
```

### Quick Test
Here's a couple of useful commands to build and run `logtail` to monitor a couple of Linux logfiles.

Open two terminals and in one run logtail-dash with:
```
cargo run --bin logtail --features="termion"  /var/log/auth.log /var/log/kern.log
```

In a second terminal you can affect the first logfile by trying and failing to 'su root':
```
su root </dev/null
```

You can use any logfiles for this basic level of testing. Here are some to try:

    /var/log/syslog
    /var/log/auth.log 
    /var/log/lastlog
    /var/log/dmesg
    /var/log/kern.log
    /var/log/boot.log

## LICENSE

Everything is GPL3.0 unless otherwise stated. Any contributions are accepted on the condition they conform to this license.

See also ./LICENSE