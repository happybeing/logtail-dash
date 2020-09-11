///! Terminal based interface and dashboard
///!
///! Edit src/custom/ui.rs to create a customised fork of logtail-dash
use super::app::{DashState, DashViewMain, LogMonitor, DEBUG_WINDOW_NAME};
use std::collections::HashMap;

use tui::{
	backend::Backend,
	layout::{Constraint, Corner, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, BorderType, Borders, List, ListItem, Widget},
	Frame, Terminal,
};

pub fn draw_dashboard<B: Backend>(
	f: &mut Frame<B>,
	dash_state: &DashState,
	monitors: &mut HashMap<String, LogMonitor>,
) {
	match dash_state.main_view {
		DashViewMain::DashHorizontal => draw_dash_horizontal(f, dash_state, monitors),
		DashViewMain::DashVertical => draw_dash_vertical(f, dash_state, monitors),
	}
}

fn draw_dash_horizontal<B: Backend>(
	f: &mut Frame<B>,
	dash_state: &DashState,
	monitors: &mut HashMap<String, LogMonitor>,
) {
	let constraints = make_percentage_constraints(monitors.len());

	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints(constraints.as_ref())
		.split(size);

	let mut chunk = 0;
	for (logfile, mut monitor) in monitors.iter_mut() {
		draw_logfile(f, chunks[chunk], &logfile, &mut monitor);
		chunk += 1;
	}
}

fn draw_dash_vertical<B: Backend>(
	f: &mut Frame<B>,
	dash_state: &DashState,
	monitors: &mut HashMap<String, LogMonitor>,
) {
	let constraints = make_percentage_constraints(monitors.len());
	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(constraints.as_ref())
		.split(size);

	let mut chunk = 0;
	for (logfile, mut monitor) in monitors.iter_mut() {
		draw_logfile(f, chunks[chunk], &logfile, &mut monitor);
		chunk += 1;
	}
}

fn draw_logfile<B: Backend>(
	f: &mut Frame<B>,
	area: Rect,
	logfile: &String,
	monitor: &mut LogMonitor,
) {
	let highlight_style = match monitor.has_focus {
		true => Style::default()
			.bg(Color::LightGreen)
			.add_modifier(Modifier::BOLD),
		false => Style::default().add_modifier(Modifier::BOLD),
	};

	let items: Vec<ListItem> = monitor
		.content
		.items
		.iter()
		.map(|s| {
			ListItem::new(vec![Spans::from(s.clone())])
				.style(Style::default().fg(Color::Black).bg(Color::White))
		})
		.collect();

	let vault_log_title = format!("Vault Log ({})", logfile);

	let logfile_widget = List::new(items)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title(vault_log_title.clone()),
		)
		.highlight_style(highlight_style);

	f.render_stateful_widget(logfile_widget, area, &mut monitor.content.state);
}

fn draw_debug_window<B: Backend>(
	f: &mut Frame<B>,
	area: Rect,
	dash_state: &mut DashState,
	monitor: &mut LogMonitor,
) {
	let highlight_style = match dash_state.debug_window_has_focus {
		true => Style::default()
			.bg(Color::LightGreen)
			.add_modifier(Modifier::BOLD),
		false => Style::default().add_modifier(Modifier::BOLD),
	};

	let items: Vec<ListItem> = dash_state
		.debug_window_list
		.items
		.iter()
		.map(|s| {
			ListItem::new(vec![Spans::from(s.clone())])
				.style(Style::default().fg(Color::Black).bg(Color::White))
		})
		.collect();

	let debug_window_widget = List::new(items)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title(String::from(DEBUG_WINDOW_NAME)),
		)
		.highlight_style(highlight_style);

	f.render_stateful_widget(
		debug_window_widget,
		area,
		&mut dash_state.debug_window_list.state,
	);
}

fn old_draw_dash_horizontal<B: Backend>(
	f: &mut Frame<B>,
	dash_state: &DashState,
	monitors: &mut HashMap<String, LogMonitor>,
) {
	let constraints = make_percentage_constraints(monitors.len());

	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints(constraints.as_ref())
		.split(size);

	for (logfile, monitor) in monitors.iter_mut() {
		let len = monitor.content.items.len();
		if len > 0 {
			monitor
				.content
				.state
				.select(Some(monitor.content.items.len() - 1));
		}

		let items: Vec<ListItem> = monitor
			.content
			.items
			.iter()
			.map(|s| {
				ListItem::new(vec![Spans::from(s.clone())])
					.style(Style::default().fg(Color::Black).bg(Color::White))
			})
			.collect();

		let monitor_widget = List::new(items)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title(logfile.clone()),
			)
			.highlight_style(
				Style::default()
					.bg(Color::LightGreen)
					.add_modifier(Modifier::BOLD),
			);
		f.render_stateful_widget(
			monitor_widget,
			chunks[monitor.index],
			&mut monitor.content.state,
		);
	}
}

fn old_draw_dash_vertical<B: Backend>(
	f: &mut Frame<B>,
	dash_state: &DashState,
	monitors: &mut HashMap<String, LogMonitor>,
) {
	let constraints = make_percentage_constraints(monitors.len());
	let size = f.size();
	let chunks = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(constraints.as_ref())
		.split(size);

	for (logfile, monitor) in monitors.iter_mut() {
		monitor
			.content
			.state
			.select(Some(monitor.content.items.len() - 1));
		let items: Vec<ListItem> = monitor
			.content
			.items
			.iter()
			.map(|s| {
				ListItem::new(vec![Spans::from(s.clone())])
					.style(Style::default().fg(Color::Black).bg(Color::White))
			})
			.collect();

		let monitor_widget = List::new(items)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title(logfile.clone()),
			)
			.highlight_style(
				Style::default()
					.bg(Color::LightGreen)
					.add_modifier(Modifier::BOLD),
			);
		f.render_stateful_widget(
			monitor_widget,
			chunks[monitor.index],
			&mut monitor.content.state,
		);
	}
}

fn make_percentage_constraints(count: usize) -> Vec<Constraint> {
	let percent = if count > 0 { 100 / count as u16 } else { 0 };
	let mut constraints = Vec::new();
	let mut total_percent = 0;

	for i in 1..count + 1 {
		total_percent += percent;

		let next_percent = if i == count && total_percent < 100 {
			100 - total_percent
		} else {
			percent
		};

		constraints.push(Constraint::Percentage(next_percent));
	}
	constraints
}
