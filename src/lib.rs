use std::collections::{HashMap, HashSet};
use std::ops::Index;
use std::path::PathBuf;

use ggez::audio::{SoundData, SoundSource, Source};
use ggez::graphics::{self, Image};
use serde::{Deserialize, Serialize};

use character::{CharacterName, Characters, Instance, InstanceName, StateName};
use interface::{Button, Render, RenderText, TextBox};

pub mod game;
pub mod lexer;
pub mod parser;
pub mod interface;
pub mod character;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FlagName(pub String);

#[derive(Debug)]
pub enum Command {
	/// Changes the state of an instance.
	Change(InstanceName, StateName),
	/// Displays text associated with a character.
	Dialogue(Option<CharacterName>, String),
	/// Presents the user with a list of options and jumps to a label
	/// depending on the option that is chosen.
	Diverge(Vec<(String, Label)>),
	/// Jumps to a label if the flag has been set.
	If(FlagName, Label),
	/// Sets a flag.
	Flag(FlagName),
	/// Removes a flag if it has been set.
	Unflag(FlagName),
	/// Makes an instance visible.
	Show(InstanceName),
	/// Makes an instance invisible.
	Hide(InstanceName),
	/// Sets the position of an instance.
	Position(InstanceName, (f32, f32)),
	/// Kills an instance.
	Kill(InstanceName),
	/// Creates an instance of a character onto the screen at a specified position.
	/// If no instance name is specified, the character name is used.
	Spawn(CharacterName, StateName, (f32, f32), Option<InstanceName>),
	/// Sets the background image.
	Stage(PathBuf),
	/// Jumps directly to a label.
	Jump(Label),
	/// Sets the currently playing music. Music audio is repeated.
	Music(PathBuf),
	/// Plays a sound effect.
	Sound(PathBuf),
	/// Waits for user interaction before continuing.
	Pause,
}

impl Command {
	pub fn execute(&self, ctx: &mut ggez::Context, state: &mut ScriptState,
	               render: &mut Render, script: &Script, settings: &Settings) {
		match self {
			Command::Change(instance, state) => {
				let instance = &mut render.stage[instance];
				*instance = Instance::new(script, instance.character.clone(),
					state, instance.position);
			}
			Command::Dialogue(character, string) => {
				let height = settings.height * settings.text_box_height - settings.interface_margin;
				let width = settings.width - 2.0 * settings.interface_margin;
				let size = (width, height - settings.interface_margin);
				let position = (settings.interface_margin, settings.height - height);
				let text = RenderText::empty(string.clone(), settings.foreground_colour);
				render.text = Some(TextBox::new(text, position, size,
					settings.background_colour).padding(settings.interface_margin));

				if let Some(CharacterName(character)) = character {
					let character_height = settings.height * settings.character_name_height;
					let position = (settings.interface_margin, settings.height -
						(height + settings.interface_margin + character_height));
					let width = settings.width * settings.character_name_width - settings.interface_margin;
					let text = RenderText::new(character.clone(), settings.foreground_colour);
					render.character = Some(TextBox::new(text, position, (width, character_height),
						settings.background_colour).padding(settings.interface_margin))
				}
			}
			Command::Diverge(branches) => {
				let button_height = settings.height * settings.branch_button_height;
				let button_width = settings.width * settings.branch_button_width;
				let position_x = (settings.width - button_width) / 2.0;

				let size = (button_width, button_height);
				let true_height = button_height + settings.interface_margin;
				let mut position_y = (settings.height - branches.len() as f32 * true_height) / 2.0;

				render.branches = branches.iter().map(|(string, label)| {
					let text = RenderText::new(string.clone(), settings.foreground_colour);
					let position = (position_x, position_y);
					position_y += true_height;

					(Button::new(TextBox::new(text, position, size, settings.background_colour)
						.alignment(graphics::Align::Center).padding(settings.interface_margin),
						settings.background_colour, settings.secondary_colour), label.clone())
				}).collect();
			}
			Command::If(flag, label) => if state.flags.contains(flag) {
				state.next_target = Some(script.labels[label].clone());
			}
			Command::Flag(flag) => { state.flags.insert(flag.clone()); }
			Command::Unflag(flag) => { state.flags.remove(flag); }
			Command::Show(instance) => render.stage[instance].visible = true,
			Command::Hide(instance) => render.stage[instance].visible = false,
			Command::Position(instance, position) => render.stage[instance].position = *position,
			Command::Kill(instance) => render.stage.remove(instance),
			Command::Spawn(character, state, position, instance_name) => {
				let CharacterName(character_name) = character;
				let instance = Instance::new(script, character.clone(), state, *position);
				render.stage.spawn(instance_name.clone().unwrap_or_else(||
					InstanceName(character_name.clone())), instance);
			}
			Command::Stage(path) => render.background = Some(script.images[path].clone()),
			Command::Jump(label) => state.next_target = Some(script.labels[label].clone()),
			Command::Music(path) => {
				let mut source = Source::from_data(ctx, script.audio[path].clone());
				source.iter_mut().for_each(|source| source.set_volume(settings.music_volume));
				source.iter_mut().for_each(|source| source.set_repeat(true));
				source.iter_mut().try_for_each(Source::play).unwrap();
				state.music = Some(source.unwrap());
			}
			Command::Sound(path) => {
				let mut source = Source::from_data(ctx, script.audio[path].clone());
				source.iter_mut().for_each(|source| source.set_volume(settings.sound_volume));
				source.iter_mut().try_for_each(Source::play).unwrap();
				state.music = Some(source.unwrap());
			}
			Command::Pause => (),
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct Target(pub usize);

impl Target {
	pub fn next(&self) -> Target {
		let Target(index) = self;
		Target(index + 1)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Label(pub String);

#[derive(Debug, Default)]
pub struct Script {
	pub characters: Characters,
	pub commands: Vec<Command>,
	pub labels: HashMap<Label, Target>,
	pub images: HashMap<PathBuf, Image>,
	pub audio: HashMap<PathBuf, SoundData>,
}

impl Index<&Target> for Script {
	type Output = Command;

	fn index(&self, Target(index): &Target) -> &Self::Output {
		&self.commands[*index]
	}
}

#[derive(Debug, Default)]
pub struct ScriptState {
	pub target: Target,
	pub next_target: Option<Target>,
	pub flags: HashSet<FlagName>,
	pub music: Option<Source>,
	pub sounds: Vec<Source>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct History {
	pub divergences: Vec<Label>,
	pub execution_count: usize,
}

#[derive(Debug, Clone)]
pub struct Settings {
	/// Width of the view.
	pub width: f32,
	/// Height of the view.
	pub height: f32,
	/// Rate at which characters are displayed.
	pub text_speed: u32,
	/// Colour of background elements such as text boxes.
	pub background_colour: [f32; 4],
	/// Colour of foreground elements such as text.
	pub foreground_colour: [f32; 4],
	/// Alternative colour for interface elements such as button hovers.
	pub secondary_colour: [f32; 4],
	/// Amount of pixels between interface elements and the game window.
	pub interface_margin: f32,
	/// Height of the main text box expressed as a multiplier of the window height.
	/// `0.5` is exactly half of the window height.
	pub text_box_height: f32,
	/// Width of the character name expressed as a multiplier of the window width.
	/// `0.5` is exactly half of the window width.
	pub character_name_width: f32,
	/// Height of the character name expressed as a multiplier of the window height.
	/// `0.1` is exactly one tenth of the window height.
	pub character_name_height: f32,
	/// Width of each branch button expressed as a multiplier of the window width.
	pub branch_button_width: f32,
	/// Height of each branch button expressed as a multiplier of the window height.
	pub branch_button_height: f32,
	/// Paths to look for resource files.
	pub resource_paths: Vec<String>,
	/// Path to save the game history.
	pub save_path: String,
	/// Volume of music that is played. The normal volume is `1.0`.
	pub music_volume: f32,
	/// Volume of sound effects that are played. The normal volume is `1.0`.
	pub sound_volume: f32,
	/// Enables developer mode features.
	pub developer: bool,
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			width: 640.0,
			height: 480.0,
			text_speed: 32,
			background_colour: [0.8, 0.8, 0.8, 0.8],
			foreground_colour: [0.0, 0.0, 0.0, 1.0],
			secondary_colour: [0.5, 0.5, 0.5, 1.0],
			interface_margin: 8.0,
			text_box_height: 0.25,
			character_name_width: 0.25,
			character_name_height: 0.08,
			branch_button_width: 0.3,
			branch_button_height: 0.1,
			resource_paths: Vec::new(),
			save_path: "/game.save".to_owned(),
			music_volume: 1.0,
			sound_volume: 1.0,
			developer: true,
		}
	}
}

