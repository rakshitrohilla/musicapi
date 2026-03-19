use crate::api::SCTrack;
use crate::api::SoundCloudClient;
use crate::player::AudioPlayer;
use anyhow::Result;

pub enum InputMode {
	Normal,
	Editing,
}

pub enum Focus {
	TrackList,
}

pub struct App {
	pub client: SoundCloudClient,
	pub player: AudioPlayer,
	pub tracks: Vec<SCTrack>,
	pub selected_track: usize,
	pub search_input: String,
	pub input_mode: InputMode,
	pub focus: Focus,
	pub should_quit: bool,
	pub is_searching: bool,
}

impl App {
	pub fn new(client_id: String) -> Result<Self> {
		Ok(Self {
			client: SoundCloudClient::new(client_id),
			player: AudioPlayer::new()?,
			tracks: Vec::new(),
			selected_track: 0,
			search_input: String::new(),
			input_mode: InputMode::Normal,
			focus: Focus::TrackList,
			should_quit: false,
			is_searching: false,
		})
	}

	pub async fn search(&mut self) -> Result<()> {
		if self.search_input.is_empty() {
			return Ok(());
		}

		self.is_searching = true;
		match self.client.search(&self.search_input, 30).await {
			Ok(tracks) => {
				self.tracks = tracks;
				self.selected_track = 0;
			}
			Err(e) => {
				eprintln!("Search error: {}", e);
			}
		}
		self.is_searching = false;
		Ok(())
	}

	pub async fn play_selected(&mut self) -> Result<()> {
		if self.tracks.is_empty() {
			return Ok(());
		}

		let track = self.tracks[self.selected_track].clone();
		self.player.queue = self.tracks.clone();
		self.player.queue_index = self.selected_track as i32;
		self.player.play_track(&track, &self.client).await?;
		Ok(())
	}

	pub fn move_selection(&mut self, delta: i32) {
		if self.tracks.is_empty() {
			return;
		}

		let new = self.selected_track as i32 + delta;
		self.selected_track = new.clamp(0, self.tracks.len() as i32 - 1) as usize;
	}
}
