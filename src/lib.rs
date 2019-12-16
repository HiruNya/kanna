use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index, Range};
use std::path::PathBuf;

use ggez::graphics::{self, Image};

pub mod game;

#[derive(Debug)]
pub struct Character(pub String);

#[derive(Debug)]
pub enum Command {
	/// Changes the state of an instance.
	/// Consists of the Instance and new State.
	Change(String, String),
	/// Displays text associated with a character.
	Dialogue(Character, String),
	/// Presents the user with a list of options and jumps to a label
	/// depending on the option that is chosen.
	Diverge(Vec<(String, Label)>),
	/// Makes an instance invisible.
	Hide(String),
	/// Kills an instance.
	Kill(String),
	/// Sets the position of an instance.
	Position(String, f32, f32),
	/// Makes an instance visible.
	Show(String),
	/// Creates an instance of a character onto the screen.
	/// Consists of the Character, State, Position, and Instance.
	Spawn(String, String, Option<(f32, f32)>, Option<String>),
	/// Sets the background image.
	Stage(PathBuf),
}

impl Command {
	pub fn execute(&self, _: &mut ScriptState, render: &mut Render, script: &Script, settings: &Settings) {
		match self {
			Command::Change(instance, state) => {
				let instance = render.stage.0.get_mut(instance).expect("Error getting instance.");
				let state = script.characters.get(&instance.character, state);
				let new_instance = Instance::new(instance.character.clone(), state, instance.position, script);
				*instance = new_instance;
			}
			Command::Dialogue(Character(character), string) => {
				let height = settings.height * settings.text_box_height - settings.interface_margin;
				let width = settings.width - 2.0 * settings.interface_margin;
				let size = (width, height - settings.interface_margin);
				let position = (settings.interface_margin, settings.height - height);
				let text = RenderText::empty(string.clone(), settings.foreground_colour);
				render.text = Some(TextBox::new(text, position, size,
					settings.background_colour).padding(settings.interface_margin));

				let character_height = settings.height * settings.character_name_height;
				let position = (settings.interface_margin, settings.height -
					(height + settings.interface_margin + character_height));
				let width = settings.width * settings.character_name_width - settings.interface_margin;
				let text = RenderText::new(character.clone(), settings.foreground_colour);
				render.character = Some(TextBox::new(text, position, (width, character_height),
					settings.background_colour).padding(settings.interface_margin))
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
			Command::Hide(instance) => {
				render.stage.0.get_mut(instance).expect("Error getting instance.").visible = false;
			}
			Command::Position(instance, x, y) => {
				let instance = render.stage.0.get_mut(instance).expect("Error getting instance.");
				instance.position = (*x, *y);
			}
			Command::Kill(instance) => {
				render.stage.0.remove(instance);
			}
			Command::Show(instance) => {
				render.stage.0.get_mut(instance).expect("Error getting instance.").visible = true;
			}
			Command::Spawn(character, state, position, instance_name) => {
				let position = position.unwrap_or((0., 0.));
				let instance = Instance::new(character.clone(), script.characters.get(character, state), position, script); 
				let character = instance_name.as_ref().unwrap_or_else(|| character).clone();
				render.stage.spawn(character, instance);
			}
			Command::Stage(path) => render.background = Some(script.images[path].clone()),
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

#[derive(Debug)]
pub struct Script {
	pub characters: Characters,
	pub commands: Vec<Command>,
	pub labels: HashMap<Label, Target>,
	pub images: HashMap<PathBuf, Image>,
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
}

#[derive(Debug, Default)]
pub struct Render {
	pub background: Option<Image>,
	pub stage: Stage,
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
		}
	}
}

/// A state represents a possible image of a character.
#[derive(Clone, Debug)]
pub struct State {
	/// The centre of the image (in pixels).
	/// This will be used when the image has it's position set, is scaled, or is rotated.
	/// Is (0, 0) by default (top-left corner of the image).
	pub centre_position: Option<(u16, u16)>,
	/// The path to the image.
	pub image: PathBuf,
	/// The amount this image is to be scaled by.
	/// (A value from 0 to 1).
	/// Is (1., 1.) by default (normal size).
	pub scale: Option<(f32, f32)>,
}
impl State {
	/// Create a new state, specifying the path to the image.
	pub fn new<P: Into<PathBuf>>(path: P) -> Self {
		Self {
			image: path.into(),
			centre_position: None,
			scale: None,
		}
	}
	/// Set the centre of the image (in pixels).
	pub fn centre_position(mut self, x: u16, y: u16) -> Self {
		self.centre_position = Some((x, y));
		self
	}
	/// Sets the scaling of the image.
	pub fn scale(mut self, x: f32, y: f32) -> Self {
		self.scale = Some((x, y));
		self
	}
}

/// A character that has been spawned onto the screen.
#[derive(Debug)]
pub struct Instance {
	/// The character which this instance belongs to.
	pub character: String,
	/// The position of the image's centre (in pixels).
	/// This determines the centre of rotation and scaling.
	pub centre_position: (f32, f32),
	/// The image that this instance draws to the screen.
	pub image: Image,
	/// The position on the screen (in pixels).
	pub position: (f32, f32),
	/// The amount the image is scaled by.
	pub scale: (f32, f32),
	/// Whether the instance is visible.
	pub visible: bool,
}
impl Instance {
	/// Creates a new instance.
	fn new(character: String, state: State, position: (f32, f32), script: &Script) -> Self {
		let image = script.images.get(&state.image)
			.unwrap_or_else(|| panic!("Could not find image at path: {:?}.", &state.image))
			.clone();
		let centre_position = state.centre_position.unwrap_or_else(||{
			let x = image.width() / 2;
			let y = image.height() / 2;
			(x, y)
		});
		let centre_position = (centre_position.0 as f32, centre_position.1 as f32);
		let scale = state.scale.unwrap_or((1., 1.));
		Instance {
			character,
			centre_position,
			image,
			position,
			scale,
			visible: true,
		}
	}
	/// Draws the instance to the screen.
	fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		let offset_x = self.centre_position.0 / (self.image.width() as f32);
		let offset_y = self.centre_position.1 / (self.image.height() as f32);
		let draw_params = graphics::DrawParam::new()
			.dest([self.position.0, self.position.1])
			.offset([offset_x, offset_y])
			.scale([self.scale.0, self.scale.1]);
		graphics::draw(ctx, &self.image, draw_params)
	}
}

/// Holds alls the instances.
#[derive(Debug, Default)]
pub struct Stage(pub HashMap<String, Instance>);
impl Stage {
	/// Draws all the instances it contains.
	pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		self.0.values()
			.filter(|v| v.visible)
			.map(|v| v.draw(ctx))
			.collect()
	}
	/// Spawns a new instance onto the stage.
	pub fn spawn(&mut self, name: String, instance: Instance) {
		self.0.insert(name, instance);
	}
}

/// Holds all the characters and their respective states.
#[derive(Debug, Default)]
pub struct Characters(pub HashMap<String, HashMap<String, State>>);
impl Characters {
	/// Add a character with a hashmap of its states.
	pub fn add_character<S: Into<String>>(&mut self, name: S, states: HashMap<S, State>) {
		let states = states.into_iter()
			.map(|(k, v)| (k.into(), v))
			.collect::<HashMap<String, State>>();
		self.0.insert(name.into(), states);
	}
	/// Load all the images held by the states into the provided HashMap of images.
	pub fn load_images(&self, images: &mut HashMap<PathBuf, Image>, ctx: &mut ggez::Context) -> ggez::GameResult {
		let state_images = self.0.values()
			.map(|characters| characters.values())
			.flatten()
			.map(|state| Ok((state.image.clone(), Image::new(ctx, state.image.clone())?)))
			.collect::<ggez::GameResult<HashMap<PathBuf, Image>>>()?
			.into_iter();
		images.extend(state_images);
		Ok(())
	}
	/// Get a state by providing the name of the character it belonged to and the name of the state.
	pub fn get(&self, character: &String, state: &String) -> State {
		let state_map = self.0.get(character).unwrap_or_else(|| panic!("Could not find Character: {}", character));
		let state = state_map.get(state).unwrap_or_else(|| panic!("Could not find State: {}", state)).clone();
		state
	}
}
