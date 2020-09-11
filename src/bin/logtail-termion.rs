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

#![recursion_limit = "512"] // Prevent select! macro blowing up

use std::collections::HashMap;
use std::io;

use linemux::MuxedLines;
use tokio::stream::StreamExt;

///! forks of logterm customise the files in src/custom
#[path = "../custom/mod.rs"]
pub mod custom;
use self::custom::app::{App, DashState, DashViewMain, LogMonitor};
use self::custom::opt::Opt;
use self::custom::ui::draw_dashboard;

///! logtail and its forks share code in src/
#[path = "../mod.rs"]
pub mod shared;
use crate::shared::util::StatefulList;
use shared::event::{Event, Events};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
	backend::Backend,
	backend::TermionBackend,
	layout::{Constraint, Corner, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, BorderType, Borders, List, ListItem, Widget},
	Frame, Terminal,
};

type TuiTerminal = tui::terminal::Terminal<
	TermionBackend<
		termion::screen::AlternateScreen<
			termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
		>,
	>,
>;

use std::io::{Error, ErrorKind};

use futures::{
	future::FutureExt, // for `.fuse()`
	pin_mut,
	select,
};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
	match terminal_main().await {
		Ok(()) => (),
		Err(e) => println!("{}", e),
	}
	Ok(())
}

async fn terminal_main() -> std::io::Result<()> {
	let mut app = match App::new().await {
		Ok(app) => app,
		Err(e) => {
			return Err(e);
		}
	};

	let events = Events::new();

	// Terminal initialization
	// info!("Intialising terminal (termion backend)");
	let stdout = io::stdout().into_raw_mode()?;
	let stdout = MouseTerminal::from(stdout);
	let stdout = AlternateScreen::from(stdout);
	let backend = TermionBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// Use futures of async functions to handle events
	// concurrently with logfile changes.
	// info!("Processing started");
	loop {
		let events_future = next_event(&events).fuse();
		let logfiles_future = app.logfiles.next().fuse();
		pin_mut!(events_future, logfiles_future);

		select! {
			(e) = events_future => {
				match e {
					Ok(Event::Input(input)) => {
						match input {
							Key::Char('q')|
							Key::Char('Q') => return Ok(()),
							Key::Char('h')|
							Key::Char('H') => app.dash_state.main_view = DashViewMain::DashHorizontal,
							Key::Char('v')|
							Key::Char('V') => app.dash_state.main_view = DashViewMain::DashVertical,
							Key::Down => app.handle_arrow_down(),
							Key::Up => app.handle_arrow_up(),
							Key::Right|
							Key::Char('\t') => app.change_focus_next(),
							Key::Left => app.change_focus_previous(),
								_ => {},
						}
					}

					Ok(Event::Tick) => {
						match terminal.draw(|f| draw_dashboard(f, &mut app.dash_state, &mut app.monitors)) {
							Ok(_) => {},
							Err(e) => {
								return Err(e);
							}
						};
					}

					Err(e) => {
						return Err(Error::new(ErrorKind::Other, "receive error"));
					}
				}
			},
			(line) = logfiles_future => {
				match line {
					Some(Ok(line)) => {
						app.dash_state._debug_window(format!("logfile: {}", line.line()).as_str());
						let source_str = line.source().to_str().unwrap();
						let source = String::from(source_str);

						match app.monitors.get_mut(&source) {
							Some(monitor) => {
								monitor.append_to_content(line.line())
							},
							None => (),
						}
					},
					Some(Err(e)) => {
						return Err(e)
					},
					None => (),
				}
			},
		}
	}
}

use std::sync::mpsc;

async fn next_event(events: &Events) -> Result<Event<Key>, mpsc::RecvError> {
	events.next()
}
