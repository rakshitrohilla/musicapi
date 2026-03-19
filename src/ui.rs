
use ratatui::{
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
	Frame,
};

use crate::api::format_duration;
use crate::app::{App, Focus, InputMode};

const GREEN: Color = Color::Rgb(0, 200, 100);
const CYAN: Color = Color::Rgb(0, 220, 220);
const DIM: Color = Color::Rgb(80, 85, 95);
const YELLOW: Color = Color::Rgb(230, 200, 50);
const BG: Color = Color::Rgb(18, 20, 26);
const PANEL_BG: Color = Color::Rgb(24, 27, 33);

pub fn draw(f: &mut Frame, app: &App) {
	let size = f.area();

	// Background.
	let bg_block = Block::default().style(Style::default().bg(BG));
	f.render_widget(bg_block, size);

	let main_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(1),  // status bar
			Constraint::Min(5),     // main content
			Constraint::Length(3),  // now playing
		])
		.split(size);

	draw_status_bar(f, app, main_layout[0]);
	draw_main(f, app, main_layout[1]);
	draw_now_playing(f, app, main_layout[2]);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
	let status = if app.player.is_playing {
		"▶ playing"
	} else {
		"■ stopped"
	};
	let track_info = app
		.player
		.current_track
		.as_ref()
		.map(|t| format!("{} — {}", t.user.username, t.title))
		.unwrap_or_default();

	let line = Line::from(vec![
		Span::styled(
			" scplayer ",
			Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
		),
		Span::styled("│ ", Style::default().fg(DIM)),
		Span::styled(status, Style::default().fg(DIM)),
		Span::styled(" │ ", Style::default().fg(DIM)),
		Span::styled(track_info, Style::default().fg(CYAN)),
	]);
	let p = Paragraph::new(line).style(Style::default().bg(PANEL_BG));
	f.render_widget(p, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
		.split(area);
	draw_left_panel(f, app, layout[0]);
	draw_right_panel(f, app, layout[1]);
}

fn draw_left_panel(f: &mut Frame, app: &App, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(3), Constraint::Min(3)])
		.split(area);

	// Search bar.
	let search_border_color = if matches!(app.input_mode, InputMode::Editing) {
		GREEN
	} else {
		DIM
	};
	let input_text = if matches!(app.input_mode, InputMode::Editing) {
		format!("❯ {}█", app.search_input)
	} else if app.search_input.is_empty() {
		"❯ search soundcloud... (press /)".to_string()
	} else {
		format!("❯ {}", app.search_input)
	};
	let search = Paragraph::new(input_text)
		.style(Style::default().fg(Color::White).bg(PANEL_BG))
		.block(
			Block::default()
				.borders(Borders::ALL)
				.border_style(Style::default().fg(search_border_color))
				.title(Span::styled(" search ", Style::default().fg(DIM)))
				.style(Style::default().bg(PANEL_BG)),
		);
	f.render_widget(search, layout[0]);

	// Track list.
	let items: Vec<ListItem> = app
		.tracks
		.iter()
		.enumerate()
		.map(|(i, track)| {
			let is_current = app
				.player
				.current_track
				.as_ref()
				.map(|t| t.id == track.id)
				.unwrap_or(false);
			let num = format!("{:>3} ", i + 1);
			let dur = format_duration(track.duration);
			let title = &track.title;
			let artist = &track.user.username;
			let style = if is_current {
				Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
			} else {
				Style::default().fg(Color::White)
			};
			let indicator = if is_current { "▶ " } else { "  " };
			ListItem::new(Line::from(vec![
				Span::styled(indicator, Style::default().fg(GREEN)),
				Span::styled(num, Style::default().fg(DIM)),
				Span::styled(format!("{} ", title), style),
				Span::styled(format!("- {} ", artist), Style::default().fg(DIM)),
				Span::styled(dur, Style::default().fg(YELLOW)),
			]))
		})
		.collect();

	let highlight_style = if matches!(app.focus, Focus::TrackList) {
		Style::default().bg(Color::Rgb(30, 40, 35)).fg(GREEN)
	} else {
		Style::default().bg(Color::Rgb(30, 33, 38))
	};

	let list = List::new(items)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.border_style(Style::default().fg(if matches!(app.focus, Focus::TrackList) {
					GREEN
				} else {
					DIM
				}))
				.title(Span::styled(
					format!(" results ({}) ", app.tracks.len()),
					Style::default().fg(DIM),
				))
				.style(Style::default().bg(PANEL_BG)),
		)
		.highlight_style(highlight_style);

	let mut state = ListState::default();
	state.select(Some(app.selected_track));
	f.render_stateful_widget(list, layout[1], &mut state);
}

fn draw_right_panel(f: &mut Frame, app: &App, area: Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
		.split(area);

	// Queue.
	let queue_items: Vec<ListItem> = app
		.player
		.queue
		.iter()
		.skip((app.player.queue_index + 1).max(0) as usize)
		.enumerate()
		.map(|(i, track)| {
			ListItem::new(Line::from(vec![
				Span::styled(format!("{:>2} ", i + 1), Style::default().fg(DIM)),
				Span::styled(&track.title, Style::default().fg(Color::White)),
				Span::styled(
					format!("  {}", format_duration(track.duration)),
					Style::default().fg(DIM),
				),
			]))
		})
		.collect();

	let queue_list = List::new(queue_items).block(
		Block::default()
			.borders(Borders::ALL)
			.border_style(Style::default().fg(DIM))
			.title(Span::styled(" queue ", Style::default().fg(DIM)))
			.style(Style::default().bg(PANEL_BG)),
	);
	f.render_widget(queue_list, layout[0]);

	// Now playing info.
	let info_lines = if let Some(track) = &app.player.current_track {
		vec![
			Line::from(""),
			Line::from(Span::styled("  ╔═══════════════╗", Style::default().fg(DIM))),
			Line::from(Span::styled("  ║  now playing  ║", Style::default().fg(GREEN))),
			Line::from(Span::styled("  ╚═══════════════╝", Style::default().fg(DIM))),
			Line::from(""),
			Line::from(Span::styled(
				format!("  {}", track.title),
				Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
			)),
			Line::from(Span::styled(
				format!("  {}", track.user.username),
				Style::default().fg(DIM),
			)),
			Line::from(Span::styled(
				format!("  {}", format_duration(track.duration)),
				Style::default().fg(YELLOW),
			)),
			Line::from(""),
			Line::from(Span::styled(
				format!("  vol: {}%", (app.player.get_volume() * 100.0) as u32),
				Style::default().fg(DIM),
			)),
		]
	} else {
		vec![
			Line::from(""),
			Line::from(Span::styled("  ╔═══════════════╗", Style::default().fg(DIM))),
			Line::from(Span::styled("  ║   music api   ║", Style::default().fg(GREEN))),
			Line::from(Span::styled("  ║  ▶ ■ ◀◀ ▶▶   ║", Style::default().fg(DIM))),
			Line::from(Span::styled("  ╚═══════════════╝", Style::default().fg(DIM))),
			Line::from(""),
			Line::from(Span::styled("  / search", Style::default().fg(DIM))),
			Line::from(Span::styled("  j/k navigate", Style::default().fg(DIM))),
			Line::from(Span::styled("  enter play", Style::default().fg(DIM))),
			Line::from(Span::styled("  space pause", Style::default().fg(DIM))),
			Line::from(Span::styled("  n/p next/prev", Style::default().fg(DIM))),
			Line::from(Span::styled("  +/- volume", Style::default().fg(DIM))),
			Line::from(Span::styled("  q quit", Style::default().fg(DIM))),
		]
	};

	let info = Paragraph::new(info_lines).block(
		Block::default()
			.borders(Borders::ALL)
			.border_style(Style::default().fg(DIM))
			.title(Span::styled(" info ", Style::default().fg(DIM)))
			.style(Style::default().bg(PANEL_BG)),
	);
	f.render_widget(info, layout[1]);
}

fn draw_now_playing(f: &mut Frame, app: &App, area: Rect) {
	let track_info = app
		.player
		.current_track
		.as_ref()
		.map(|t| format!("  {} — {}  ", t.title, t.user.username))
		.unwrap_or_else(|| "  no track selected  ".to_string());
	let status_icon = if app.player.is_playing { "▶" } else { "║║" };

	let line = Line::from(vec![
		Span::styled(format!(" {} ", status_icon), Style::default().fg(GREEN)),
		Span::styled(track_info, Style::default().fg(Color::White)),
		Span::styled(
			format!("  vol:{}%  ", (app.player.get_volume() * 100.0) as u32),
			Style::default().fg(DIM),
		),
	]);

	let bar = Paragraph::new(line).block(
		Block::default()
			.borders(Borders::TOP)
			.border_style(Style::default().fg(DIM))
			.style(Style::default().bg(PANEL_BG)),
	);
	f.render_widget(bar, area);
}
