use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index, Range};
use std::path::PathBuf;

use ggez::audio::{SoundData, SoundSource, Source};
use ggez::graphics::{self, Image};

pub mod game;
pub mod parser;
pub mod lexer;

#[derive(Debug)]
pub struct Character(pub String);

#[derive(Debug)]
pub enum Command {
	/// Displays text associated with a character.
	Dialogue(Option<Character>, String),
	/// Presents the user with a list of options and jumps to a label
	/// depending on the option that is chosen.
	Diverge(Vec<(String, Label)>),
	/// Sets the background image.
	Stage(PathBuf),
	/// Jumps directly to a label.
	Jump(Label),
	/// Sets the currently playing music. Music audio is repeated.
	Music(PathBuf),
	/// Plays a sound effect.
	Sound(PathBuf),
}

impl Command {
	pub fn execute(&self, ctx: &mut ggez::Context, state: &mut ScriptState,
	               render: &mut Render, script: &Script, settings: &Settings) {
		match self {
			Command::Dialogue(character, string) => {
				let height = settings.height * settings.text_box_height - settings.interface_margin;
				let width = settings.width - 2.0 * settings.interface_margin;
				let size = (width, height - settings.interface_margin);
				let position = (settings.interface_margin, settings.height - height);
				let text = RenderText::empty(string.clone(), settings.foreground_colour);
				render.text = Some(TextBox::new(text, position, size,
					settings.background_colour).padding(settings.interface_margin));

				if let Some(Character(character)) = character {
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
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct Target(pub usize);

impl Target {
	pub fn advance(&mut self) {
		let Target(index) = self;
		*index += 1;
	}
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Label(pub String);

#[derive(Debug, Default)]
pub struct Script {
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
	pub music: Option<Source>,
	pub sounds: Vec<Source>,
}

#[derive(Debug, Default)]
pub struct Render {
	pub background: Option<Image>,
	pub character: Option<TextBox>,
	pub text: Option<TextBox>,
	pub branches: Vec<(Button, Label)>,
}

#[derive(Debug)]
pub struct RenderText {
	pub string: String,
	pub slice: Range<usize>,
	pub colour: [f32; 4],
}

impl RenderText {
	/// Creates a `RenderText` with all characters initially displayed.
	pub fn new(string: String, colour: [f32; 4]) -> Self {
		let slice = Range { start: 0, end: string.len() };
		RenderText { string, slice, colour }
	}

	/// Creates a `RenderText` with no characters initially displayed.
	pub fn empty(string: String, colour: [f32; 4]) -> Self {
		let slice = Range { start: 0, end: 0 };
		RenderText { string, slice, colour }
	}

	/// Adds an additional character to be rendered.
	/// Does nothing if the end of the string is already rendered.
	pub fn step(&mut self) {
		self.string[self.slice.end..].char_indices().skip(1)
			.next().map(|(index, _)| self.slice.end += index)
			.unwrap_or_else(|| self.finish());
	}

	/// Adds all remaining characters to be rendered.
	pub fn finish(&mut self) {
		self.slice.end = self.string.len();
	}

	/// Checks whether all the characters have been rendered.
	pub fn is_finished(&self) -> bool {
		self.slice.end == self.string.len()
	}

	pub fn fragment(&self) -> graphics::TextFragment {
		let string = self.string[self.slice.clone()].to_owned();
		graphics::TextFragment::new(string).color(self.colour.into())
	}
}

#[derive(Debug)]
pub struct TextBox {
	pub text: RenderText,
	pub position: (f32, f32),
	pub size: (f32, f32),
	pub colour: [f32; 4],
	pub padding: f32,
	pub alignment: graphics::Align,
}

impl TextBox {
	pub fn new(text: RenderText, position: (f32, f32), size: (f32, f32), colour: [f32; 4]) -> Self {
		TextBox { text, position, size, colour, padding: 0.0, alignment: graphics::Align::Left }
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.padding = padding;
		self
	}

	pub fn alignment(mut self, alignment: graphics::Align) -> Self {
		self.alignment = alignment;
		self
	}

	pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		let rectangle = self.rectangle();
		let fragment = self.text.fragment();
		let text_box = graphics::Mesh::new_rectangle(ctx,
			graphics::DrawMode::fill(), rectangle, self.colour.into())?;
		graphics::draw(ctx, &text_box, graphics::DrawParam::new())?;

		let bounds = [rectangle.w - 2.0 * self.padding, rectangle.h - 2.0 * self.padding];
		let text_position = ([rectangle.x + self.padding, rectangle.y + self.padding], );
		graphics::draw(ctx, graphics::Text::new(fragment)
			.set_bounds(bounds, self.alignment), text_position)
	}

	fn rectangle(&self) -> graphics::Rect {
		let (x, y) = self.position;
		let (width, height) = self.size;
		[x, y, width, height].into()
	}
}

impl Deref for TextBox {
	type Target = RenderText;

	fn deref(&self) -> &Self::Target {
		&self.text
	}
}

impl DerefMut for TextBox {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.text
	}
}

#[derive(Debug)]
pub struct Button {
	pub text: TextBox,
	pub default: [f32; 4],
	pub hover: [f32; 4],
}

impl Button {
	pub fn new(text: TextBox, default: [f32; 4], hover: [f32; 4]) -> Self {
		Button { text, default, hover }
	}

	pub fn update(&mut self, (x, y): (f32, f32)) {
		match self.text.rectangle().contains([x, y]) {
			false => self.text.colour = self.default,
			true => self.text.colour = self.hover,
		}
	}
}

impl Deref for Button {
	type Target = TextBox;

	fn deref(&self) -> &Self::Target {
		&self.text
	}
}

#[derive(Debug)]
pub struct Settings {
	/// Width of the game window.
	pub width: f32,
	/// Height of the game window.
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
	/// Volume of music that is played. The normal volume is `1.0`.
	pub music_volume: f32,
	/// Volume of sound effects that are played. The normal volume is `1.0`.
	pub sound_volume: f32,
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
			music_volume: 1.0,
			sound_volume: 1.0,
		}
	}
}
