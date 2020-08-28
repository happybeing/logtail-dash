//! This app monitors and logfiles and displays status in the terminal
//!
//! It is based on logtail-dash, which is a basic logfile dashboard
//! and also a framework for similar apps with customised dahsboard
//! displays.
//!
//! Custom apps based on logtail can be created by creating a
//! fork of logtail-dash and modifying the files in src/custom
//!
//! See README for more information.

#![recursion_limit = "256"] // Prevent select! macro blowing up

use linemux::MuxedLines;
use std::collections::HashMap;
use tokio::stream::StreamExt;

///! forks of logterm customise the files in src/custom
#[path = "../custom/mod.rs"]
pub mod custom;
use self::custom::app::{DashState, DashViewMain, LogMonitor};
use self::custom::opt::Opt;
use self::custom::ui::draw_dashboard;

///! logtail and its forks share code in src/
#[path = "../mod.rs"]
pub mod shared;
use crate::shared::util::StatefulList;

use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{
	error::Error,
	io::{stdout, Write},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Corner, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, BorderType, Borders, List, ListItem, Widget},
	Frame, Terminal,
};

use futures::{
	future::FutureExt, // for `.fuse()`
	pin_mut,
	select,
};

enum Event<I> {
	Input(I),
	Tick,
}

use structopt::StructOpt;

// RUSTFLAGS="-A unused" cargo run --bin logtail-crossterm --features="crossterm" /var/log/auth.log /var/log/dmesg
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
	// pub async fn main() -> std::io::Result<()> {
	let opt = Opt::from_args();

	if opt.files.is_empty() {
		println!("{}: no logfile(s) specified.", Opt::clap().get_name());
		println!(
			"Try '{} --help' for more information.",
			Opt::clap().get_name()
		);
		return Ok(());
	}

	let mut dash_state = DashState::new();
	let mut monitors: HashMap<String, LogMonitor> = HashMap::new();
	let mut logfiles = MuxedLines::new()?;

	println!("Loading...");
	for f in opt.files {
		let mut monitor = LogMonitor::new(f.to_string(), opt.lines_max);
		println!("{}", monitor.logfile);
		if opt.ignore_existing {
			monitors.insert(f.to_string(), monitor);
		} else {
			match monitor.load_logfile() {
				Ok(()) => {
					monitors.insert(f.to_string(), monitor);
				}
				Err(e) => {
					println!("...failed: {}", e);
					return Ok(());
				}
			}
		}
		match logfiles.add_file(&f).await {
			Ok(_) => {}
			Err(e) => {
				println!("ERROR: {}", e);
				println!("Note: it is ok for the file not to exist, but the file's parent directory must exist.");
				return Ok(());
			}
		}
	}

	// Terminal initialization
	enable_raw_mode()?;
	let mut stdout = stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	let rx = initialise_events(opt.tick_rate);
	terminal.clear()?;

	// Use futures of async functions to handle events
	// concurrently with logfile changes.
	loop {
		terminal.draw(|f| draw_dashboard(f, &dash_state, &mut monitors))?;
		let logfiles_future = logfiles.next().fuse();
		let events_future = next_event(&rx).fuse();
		pin_mut!(logfiles_future, events_future);

		select! {
		  (e) = events_future => {
			match e {
			  Ok(Event::Input(event)) => match event.code {
				KeyCode::Char('q')|
				KeyCode::Char('Q') => {
				  disable_raw_mode()?;
				  execute!(
					  terminal.backend_mut(),
					  LeaveAlternateScreen,
					  DisableMouseCapture
				  )?;
				  terminal.show_cursor()?;
				  break Ok(());
				},
				KeyCode::Char('h')|
				KeyCode::Char('H') => dash_state.main_view = DashViewMain::DashHorizontal,
				KeyCode::Char('v')|
				KeyCode::Char('V') => dash_state.main_view = DashViewMain::DashVertical,
				_ => {},
			  }

			  Ok(Event::Tick) => {
				// draw_dashboard(&mut f, &dash_state, &mut monitors).unwrap();
				// draw_dashboard(f, &dash_state, &mut monitors)?;
			  }

			  Err(error) => {
				println!("{}", error);
			  }
			}
		  },

		  (line) = logfiles_future => {
			match line {
			  Some(Ok(line)) => {
				let source_str = line.source().to_str().unwrap();
				let source = String::from(source_str);

				match monitors.get_mut(&source) {
				  None => (),
				  Some(monitor) => monitor.append_to_content(line.line())
				}
			  },
			  Some(Err(e)) => panic!("{}", e),
			  None => (),
			}
		  },
		}
	}
}
// type Tx = std::sync::mpsc::Sender<Event<crossterm::event::KeyEvent>>;
type Rx = std::sync::mpsc::Receiver<Event<crossterm::event::KeyEvent>>;

fn initialise_events(tick_rate: u64) -> Rx {
	let tick_rate = Duration::from_millis(tick_rate);
	let (tx, rx) = mpsc::channel(); // Setup input handling

	thread::spawn(move || {
		let mut last_tick = Instant::now();
		loop {
			// poll for tick rate duration, if no events, sent tick event.
			if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
				if let CEvent::Key(key) = event::read().unwrap() {
					tx.send(Event::Input(key)).unwrap();
				}
			}
			if last_tick.elapsed() >= tick_rate {
				tx.send(Event::Tick).unwrap(); // <-- PANICS HERE
				last_tick = Instant::now();
			}

			if last_tick.elapsed() >= tick_rate {
				match tx.send(Event::Tick) {
					Ok(()) => last_tick = Instant::now(),
					Err(e) => println!("send error: {}", e),
				}
			}
		}
	});
	rx
}

async fn next_event(rx: &Rx) -> Result<Event<crossterm::event::KeyEvent>, mpsc::RecvError> {
	rx.recv()
}
