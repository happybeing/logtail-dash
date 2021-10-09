///! Application logic
///!
///! Edit src/custom/app.rs to create a customised fork of logtail-dash
use std::fs::File;
use std::io::{Error, ErrorKind, Write};

use linemux::MuxedLines;
use std::collections::HashMap;
use structopt::StructOpt;

use crate::custom::opt::Opt;
use crate::shared::util::StatefulList;

pub static DEBUG_WINDOW_NAME: &str = "Debug Window";

pub struct App {
	pub opt: Opt,
	pub dash_state: DashState,
	pub monitors: HashMap<String, LogMonitor>,
	pub logfile_with_focus: String,
	pub logfiles: MuxedLines,
	pub logfile_names: Vec<String>,
}

impl App {
	pub async fn new() -> Result<App, std::io::Error> {
		let mut opt = Opt::from_args();

		if opt.files.is_empty() {
			println!("{}: no logfile(s) specified.", Opt::clap().get_name());
			println!(
				"Try '{} --help' for more information.",
				Opt::clap().get_name()
			);
			return Err(Error::new(ErrorKind::Other, "missing logfiles"));
		}

		let mut dash_state = DashState::new();
		let mut monitors: HashMap<String, LogMonitor> = HashMap::new();
		let mut logfiles = MuxedLines::new()?;
		let mut name_for_focus = String::new();
		let mut logfile_names = Vec::<String>::new();

		println!("Loading {} files...", opt.files.len());
		for f in &opt.files {
			println!("file: {}", f);
			let mut monitor = LogMonitor::new(f.to_string(), opt.lines_max);
			let f = match logfiles.add_file(&f).await {
				Ok(canonical_path) => canonical_path.to_string_lossy().to_string(),
				Err(e) => {
					println!("ERROR: {}", e);
					println!(
						"Note: it is ok for the file not to exist, but the file's parent directory must exist."
					);
					return Err(e);
				}
			};

			if opt.ignore_existing {
				logfile_names.push(f.clone());
				monitors.insert(f.clone(), monitor);
			} else {
				match monitor.load_logfile() {
					Ok(()) => {
						logfile_names.push(f.clone());
						monitors.insert(f.clone(), monitor);
					}
					Err(e) => {
						println!("...failed: {}", e);
						return Err(e);
					}
				}
			}

			if name_for_focus.is_empty() {
				name_for_focus = f.clone();
			}

		}

		let mut app = App {
			opt,
			dash_state,
			monitors,
			logfile_with_focus: name_for_focus.clone(),
			logfiles,
			logfile_names,
		};
		app.set_logfile_focus(&name_for_focus);
		Ok(app)
	}

	pub fn get_monitor_for_file_path(&mut self, logfile: &String) -> Option<(&mut LogMonitor)> {
		let mut index = 0;
		let mut monitor_for_path = None;
		for (monitor_file, mut monitor) in self.monitors.iter_mut() {
			if monitor_file.eq(logfile) {
				monitor_for_path = Some(monitor);
				break;
			}
			use std::env::current_dir;
			use std::path::Path;
			if let Ok(current_dir) = current_dir() {
				let logfile_path = Path::new(logfile.as_str());
				if current_dir.join(monitor_file).eq(&logfile_path) {
					monitor_for_path = Some(monitor);
					break;
				}
			}
			index += 1;
		}
		return monitor_for_path;
	}

	pub fn get_monitor_with_focus(&mut self) -> Option<(&mut LogMonitor)> {
		match (&mut self.monitors).get_mut(&self.logfile_with_focus) {
			Some(mut monitor) => Some(monitor),
			None => None,
		}
	}

	pub fn set_logfile_focus(&mut self, logfile_name: &String) {
		match self.get_monitor_with_focus() {
			Some(fading_monitor) => {
				fading_monitor.has_focus = false;
				self.logfile_with_focus = String::new();
			}
			None => (),
		}

		if logfile_name == DEBUG_WINDOW_NAME {
			self.dash_state.debug_window_has_focus = true;
			self.logfile_with_focus = logfile_name.clone();
			return;
		} else {
			self.dash_state.debug_window_has_focus = false;
		}

		if let Some(focus_monitor) = (&mut self.monitors).get_mut(logfile_name) {
			focus_monitor.has_focus = true;
			self.logfile_with_focus = logfile_name.clone();
		};
	}

	pub fn change_focus_next(&mut self) {
		let mut next_i = 0;
		for (i, name) in self.logfile_names.iter().enumerate() {
			if name == &self.logfile_with_focus {
				if i < self.logfile_names.len() - 1 {
					next_i = i + 1;
				}
				break;
			}
		}

		if false	// In case --debug_window is implemented (see vdash)
			&& next_i == 0 
			&& self.opt.debug_window 
			&& self.logfile_with_focus != DEBUG_WINDOW_NAME
		{
			self.set_logfile_focus(&DEBUG_WINDOW_NAME.to_string());
			return;
		}

		let new_focus_name = &self.logfile_names[next_i].to_string();
		self.set_logfile_focus(&new_focus_name);
	}

	pub fn change_focus_previous(&mut self) {
		let len = self.logfile_names.len();
		let mut previous_i = len - 1;
		let mut log = String::from("change_previous()           \r\n");

		for (i, name) in self.logfile_names.iter().enumerate() {
			log = format!("{}loop name: {}           \r\n", log, name).to_string();
			if name == &self.logfile_with_focus {
				if i > 0 {
					previous_i = i - 1;
				}
				break;
			}
		}
		if false	// In case --debug_window is implemented (see vdash)
			&& self.opt.debug_window
			&& previous_i == len - 1
			&& self.logfile_with_focus != DEBUG_WINDOW_NAME
		{
			self.set_logfile_focus(&DEBUG_WINDOW_NAME.to_string());
			return;
		}
		let new_focus_name = &self.logfile_names[previous_i].to_string();
		self.set_logfile_focus(&new_focus_name);
	}

	pub fn handle_arrow_up(&mut self) {
		if let Some(monitor) = self.get_monitor_with_focus() {
			do_bracketed_next_previous(&mut monitor.content, false);
		} else if self.opt.debug_window {
			do_bracketed_next_previous(&mut self.dash_state.debug_window_list, false);
		}
	}

	pub fn handle_arrow_down(&mut self) {
		if let Some(monitor) = self.get_monitor_with_focus() {
			do_bracketed_next_previous(&mut monitor.content, true);
		} else if self.opt.debug_window {
			do_bracketed_next_previous(&mut self.dash_state.debug_window_list, true);
		}
	}
}

/// Move selection forward or back without wrapping at start or end
fn do_bracketed_next_previous(list: &mut StatefulList<String>, next: bool) {
	if (next) {
		if let Some(selected) = list.state.selected() {
			if selected != list.items.len() - 1 {
				list.next();
			}
		} else {
			list.previous();
		}
	} else {
		if let Some(selected) = list.state.selected() {
			if selected != 0 {
				list.previous();
			}
		} else {
			list.previous();
		}
	}
}

pub struct LogMonitor {
	pub index: usize,
	pub content: StatefulList<String>,
	pub has_focus: bool,
	pub logfile: String,

	max_content: usize, // Limit number of lines in content
}

use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT_MONITOR: AtomicUsize = AtomicUsize::new(0);

impl LogMonitor {
	pub fn new(f: String, max_lines: usize) -> LogMonitor {
		let index = NEXT_MONITOR.fetch_add(1, Ordering::Relaxed);
		LogMonitor {
			index,
			has_focus: false,
			logfile: f,
			max_content: max_lines,
			content: StatefulList::with_items(vec![]),
		}
	}

	pub fn load_logfile(&mut self) -> std::io::Result<()> {
		use std::io::{BufRead, BufReader};

		let f = File::open(self.logfile.to_string());
		let f = match f {
			Ok(file) => file,
			Err(_e) => return Ok(()), // It's ok for a logfile not to exist yet
		};

		let f = BufReader::new(f);

		for line in f.lines() {
			let line = line.expect("Unable to read line");
			self.process_line(&line);
		}

		Ok(())
	}

	pub fn process_line(&mut self, text: &str) {
		// TODO parse and update metrics
		self.append_to_content(text);
	}

	pub fn append_to_content(&mut self, text: &str) {
		self.content.items.push(text.to_string());
		let len = self.content.items.len();
		if len > self.max_content {
			self.content.items = self.content.items.split_off(len - self.max_content);
		} else {
			self.content.state.select(Some(len - 1));
		}
	}

	fn _reset_metrics(&mut self) {}
}

pub enum DashViewMain {
	DashHorizontal,
	DashVertical,
}

pub struct DashState {
	pub main_view: DashViewMain,
	pub debug_window: bool,
	pub debug_window_has_focus: bool,
	max_debug_window: usize,

	// For --debug-window option
	pub debug_window_list: StatefulList<String>,

	// For DashViewMain::DashVertical
	dash_vertical: DashVertical,
}

impl DashState {
	pub fn new() -> DashState {
		DashState {
			main_view: DashViewMain::DashHorizontal,
			dash_vertical: DashVertical::new(),
			debug_window: false,
			debug_window_has_focus: false,
			debug_window_list: StatefulList::new(),
			max_debug_window: 100,
		}
	}

	pub fn _debug_window(&mut self, text: &str) {
		self.debug_window_list.items.push(text.to_string());
		let len = self.debug_window_list.items.len();

		if len > self.max_debug_window {
			self.debug_window_list.items = self
				.debug_window_list
				.items
				.split_off(len - self.max_debug_window);
		} else {
			self.debug_window_list.state.select(Some(len - 1));
		}
	}
}

pub struct DashVertical {
	active_view: usize,
}

impl DashVertical {
	pub fn new() -> Self {
		DashVertical { active_view: 0 }
	}
}
