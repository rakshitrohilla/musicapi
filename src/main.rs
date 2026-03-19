mod api;
mod app;
mod player;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use app::{App, InputMode};
use crossterm::{
	event::{self, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

fn get_client_id() -> String {
	// Check env var first, then config file, then prompt.
	if let Ok(id) = std::env::var("SC_CLIENT_ID") {
		return id;
	}

	if let Some(config_dir) = dirs::config_dir() {
		let config_path = config_dir.join("scplayer").join("client_id");
		if let Ok(id) = std::fs::read_to_string(&config_path) {
			let id = id.trim().to_string();
			if !id.is_empty() {
				return id;
			}
		}
	}

	eprintln!("Enter your SoundCloud client_id:");
	let mut input = String::new();
	io::stdin()
		.read_line(&mut input)
		.expect("Failed to read input");
	let id = input.trim().to_string();

	// Save for next time.
	if let Some(config_dir) = dirs::config_dir() {
		let dir = config_dir.join("scplayer");
		let _ = std::fs::create_dir_all(&dir);
		let _ = std::fs::write(dir.join("client_id"), &id);
	}

	id
}

#[tokio::main]
async fn main() -> Result<()> {
	let client_id = get_client_id();

	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;

	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	let mut app = App::new(client_id)?;

	let result = run_app(&mut terminal, &mut app).await;

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;

	if let Err(e) = result {
		eprintln!("Error: {}", e);
	}

	Ok(())
}

async fn run_app(
	terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
	app: &mut App,
) -> Result<()> {
	loop {
		terminal.draw(|f| ui::draw(f, app))?;

		if event::poll(Duration::from_millis(100))? {
			if let Event::Key(key) = event::read()? {
				if key.kind != KeyEventKind::Press {
					continue;
				}

				match &app.input_mode {
					InputMode::Editing => match key.code {
						KeyCode::Enter => {
							app.input_mode = InputMode::Normal;
							app.search().await?;
						}
						KeyCode::Esc => {
							app.input_mode = InputMode::Normal;
						}
						KeyCode::Char(c) => {
							app.search_input.push(c);
						}
						KeyCode::Backspace => {
							app.search_input.pop();
						}
						_ => {}
					},
					InputMode::Normal => match key.code {
						KeyCode::Char('q') => {
							app.should_quit = true;
						}
						KeyCode::Char('/') => {
							app.input_mode = InputMode::Editing;
							app.search_input.clear();
						}
						KeyCode::Char('j') | KeyCode::Down => {
							app.move_selection(1);
						}
						KeyCode::Char('k') | KeyCode::Up => {
							app.move_selection(-1);
						}
						KeyCode::Char('g') => {
							app.selected_track = 0;
						}
						KeyCode::Char('G') => {
							if !app.tracks.is_empty() {
								app.selected_track = app.tracks.len() - 1;
							}
						}
						KeyCode::Enter => {
							app.play_selected().await?;
						}
						KeyCode::Char(' ') => {
							app.player.toggle_pause();
						}
						KeyCode::Char('n') => {
							app.player.next(&app.client).await?;
						}
						KeyCode::Char('p') => {
							app.player.prev(&app.client).await?;
						}
						KeyCode::Char('+') | KeyCode::Char('=') => {
							let v = (app.player.get_volume() + 0.05).min(1.0);
							app.player.set_volume(v);
						}
						KeyCode::Char('-') => {
							let v = (app.player.get_volume() - 0.05).max(0.0);
							app.player.set_volume(v);
						}
						_ => {}
					},
				}

				if app.should_quit {
					return Ok(());
				}
			}
		}

		// Auto-next when track ends.
		if app.player.is_playing && app.player.is_empty() {
			app.player.next(&app.client).await?;
		}
	}
}
