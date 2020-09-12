///! Command line options and usage
///!
///! Edit src/custom/opt.rs to create a customised fork of logtail-dash

static MAX_CONTENT: &str = "100";

pub use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
	about = "Monitor multiple logfiles in the terminal.\nUse tab or arrow keys to navigate and scroll."
)]
pub struct Opt {
	/// Maximum number of lines to keep for each logfile
	#[structopt(short = "l", long, default_value = "100")]
	pub lines_max: usize,

	/// Time between ticks in milliseconds
	#[structopt(short, long, default_value = "200")]
	pub tick_rate: u64,

	/// Ignore any existing logfile content
	#[structopt(short, long)]
	pub ignore_existing: bool,

	/// One or more logfiles to monitor
	#[structopt(name = "LOGFILE")]
	pub files: Vec<String>,

	// Show a debug window (not implemented in logtail)
	/// (Not implemented in logtail).
	#[structopt(short, long, hidden = true)]
	pub debug_window: bool,
}
