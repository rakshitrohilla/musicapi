use std::io::Cursor;

use anyhow::Result;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::api::{SCTrack, SoundCloudClient};

pub struct AudioPlayer {
	device_sink: MixerDeviceSink,
	player: Player,
	pub queue: Vec<SCTrack>,
	pub queue_index: i32,
	pub current_track: Option<SCTrack>,
	pub is_playing: bool,
}

impl AudioPlayer {
	pub fn new() -> Result<Self> {
		let mut device_sink = DeviceSinkBuilder::open_default_sink()?;
		device_sink.log_on_drop(false);
		let player = Player::connect_new(device_sink.mixer());
		player.pause();
		Ok(Self {
			device_sink,
			player,
			queue: Vec::new(),
			queue_index: -1,
			current_track: None,
			is_playing: false,
		})
	}

	pub async fn play_track(&mut self, track: &SCTrack, client: &SoundCloudClient) -> Result<()> {
		let url = client.get_stream_url(track).await?;

		// Download audio data.
		let response = reqwest::get(&url).await?;
		let bytes = response.bytes().await?;

		self.player.stop();

		let cursor = Cursor::new(bytes.to_vec());
		let source = Decoder::try_from(cursor)?;
		self.player.append(source);
		self.player.play();

		self.current_track = Some(track.clone());
		self.is_playing = true;
		Ok(())
	}

	pub fn toggle_pause(&mut self) {
		if self.is_playing {
			self.player.pause();
			self.is_playing = false;
		} else {
			self.player.play();
			self.is_playing = true;
		}
	}

	pub fn set_volume(&self, volume: f32) {
		self.player.set_volume(volume.clamp(0.0, 1.0));
	}

	pub fn get_volume(&self) -> f32 {
		self.player.volume()
	}

	pub fn is_empty(&self) -> bool {
		self.player.empty()
	}

	pub async fn play_from_queue(&mut self, index: usize, client: &SoundCloudClient) -> Result<()> {
		if index < self.queue.len() {
			let track = self.queue[index].clone();
			self.queue_index = index as i32;
			self.play_track(&track, client).await?;
		}
		Ok(())
	}

	pub async fn next(&mut self, client: &SoundCloudClient) -> Result<()> {
		let next = self.queue_index + 1;
		if (next as usize) < self.queue.len() {
			self.play_from_queue(next as usize, client).await?;
		}
		Ok(())
	}

	pub async fn prev(&mut self, client: &SoundCloudClient) -> Result<()> {
		let prev = self.queue_index - 1;
		if prev >= 0 {
			self.play_from_queue(prev as usize, client).await?;
		}
		Ok(())
	}
}
