use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index, IndexMut, Range};
use std::path::PathBuf;

use ggez::audio::{SoundData, SoundSource, Source};
use ggez::graphics::{self, Image};

use animation::{Animation, AnimationDeclaration, AnimationMap, AnimationState, ChangeAnimation, HideAnimation, InstanceParameter, KillAnimation, PositionAnimation, ShowAnimation, SpawnAnimation};

pub mod animation;
pub mod game;
pub mod parser;
pub mod lexer;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CharacterName(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct InstanceName(pub String);

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct StateName(pub String);

#[derive(Debug)]
pub enum Command {
	/// Changes the state of an instance.
	Change(InstanceName, StateName, Option<AnimationDeclaration>),
	/// Displays text associated with a character.
	Dialogue(Option<CharacterName>, String),
	/// Presents the user with a list of options and jumps to a label
	/// depending on the option that is chosen.
	Diverge(Vec<(String, Label)>),
	/// Makes an instance visible.
	Show(InstanceName, Option<AnimationDeclaration>),
	/// Makes an instance invisible.
	Hide(InstanceName, Option<AnimationDeclaration>),
	/// Sets the position of an instance.
	Position(InstanceName, (f32, f32), Option<AnimationDeclaration>),
	/// Kills an instance.
	Kill(InstanceName, Option<AnimationDeclaration>),
	/// Creates an instance of a character onto the screen at a specified position.
	/// If no instance name is specified, the character name is used.
	Spawn(CharacterName, StateName, (f32, f32), Option<InstanceName>, Option<AnimationDeclaration>),
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
			Command::Change(instance, state, animation) => {
				let instance = &mut render.stage[instance];
				if let Some(animation) = animation {
					let change_animation = ChangeAnimation::new(animation.arguments.clone(), &instance.character, script, state);
					let animation = script.animations.change.get(&animation.name).unwrap_or_else(|| panic!("Error finding animation: {}", animation.name))
						.initialise(change_animation);
					instance.add_animation(animation);
				} else {
					*instance = Instance::new(script, instance.character.clone(),
						state, instance.position);
				}
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
			Command::Show(instance, animation) => {
				if let Some(animation) = animation {
					let animation_producer = script.animations.show.get(&animation.name).unwrap_or_else(|| panic!("Error finding animation named: {}", animation.name));
					render.stage[instance].add_animation(animation_producer.initialise(ShowAnimation { arguments: animation.arguments.clone(), view_dimensions: (settings.width, settings.height) }) as Box<_>)
				} else {
					render.stage[instance].visible = true
				}
			}
			Command::Hide(instance, animation) => {
				if let Some(animation) = animation {
					let animation_producer = script.animations.hide.get(&animation.name).unwrap_or_else(|| panic!("Error finding animation named: {}", animation.name));
					render.stage[instance].add_animation(animation_producer.initialise(HideAnimation { arguments: animation.arguments.clone(), view_dimensions: (settings.width, settings.height)  }) as Box<_>)
				} else {
					render.stage[instance].visible = false
				}
			}
			Command::Position(instance, position, animation) => {
				if let Some(animation) = animation {
					let position_animation = PositionAnimation {
						destination: *position,
						arguments: animation.arguments.clone(),
					};
					let animation = script.animations.position.get(&animation.name)
						.unwrap_or_else(|| panic!("Error finding animation named `{}`", animation.name))
						.initialise(position_animation);
					render.stage[instance].add_animation(animation);
				} else {
					render.stage[instance].position = *position;
				}
			}
			Command::Kill(instance, animation) => {
				if let Some(animation) = animation {
					let animation = script.animations.kill.get(&animation.name)
						.unwrap_or_else(|| panic!("Error finding animation: {}", animation.name))
						.initialise(KillAnimation{ arguments: animation.arguments.clone(), view_dimensions: (settings.width, settings.height)  });
					render.stage[instance].add_animation(animation);
					render.stage[instance].tbk = true;
				} else {
					render.stage.remove(instance)
				}
			}
			Command::Spawn(character, state, position, instance_name, animation) => {
				let CharacterName(character_name) = character;
				let instance = Instance::new(script, character.clone(), state, *position);
				let instance_name = instance_name.clone().unwrap_or_else(||
					InstanceName(character_name.clone()));
				render.stage.spawn(instance_name.clone(), instance);
				if let Some(animation) = animation {
					let animation = script.animations.spawn.get(&animation.name)
						.unwrap_or_else(|| panic!("Error finding animation named `{}`", animation.name))
						.initialise(SpawnAnimation{ arguments: animation.arguments.clone(), view_dimensions: (settings.width, settings.height)  });
					render.stage[&instance_name].add_animation(animation);
				}
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
	pub characters: Characters,
	pub commands: Vec<Command>,
	pub labels: HashMap<Label, Target>,
	pub images: HashMap<PathBuf, Image>,
	pub audio: HashMap<PathBuf, SoundData>,
	pub animations: AnimationMap,
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
	pub stage: Stage,
	pub character: Option<TextBox>,
	pub text: Option<TextBox>,
	pub branches: Vec<(Button, Label)>,
	pub shadow_bars: [graphics::Rect; 2],
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

/// A state represents a possible character image.
#[derive(Clone, Debug)]
pub struct State {
	/// Path to the image.
	pub image: PathBuf,
	/// Centre of the image in pixels.
	/// This is used when the image sets its position, is scaled, or is rotated.
	/// If no position is specified then the pixel centre of the image is used.
	pub centre_position: Option<(u16, u16)>,
	/// Amount this image is to be scaled by.
	/// Default is `(1.0, 1.0)` (normal size).
	pub scale: (f32, f32),
}

impl State {
	/// Creates a new state from the path to the image.
	pub fn new<P: Into<PathBuf>>(path: P) -> Self {
		Self {
			image: path.into(),
			centre_position: None,
			scale: (1.0, 1.0),
		}
	}

	/// Sets the centre of the image in pixels.
	pub fn centre_position(mut self, (x, y): (u16, u16)) -> Self {
		self.centre_position = Some((x, y));
		self
	}

	/// Sets the scaling of the image.
	pub fn scale(mut self, (x, y): (f32, f32)) -> Self {
		self.scale = (x, y);
		self
	}
}

/// A character that has been spawned onto the screen.
#[derive(Debug)]
pub struct Instance {
	/// An animation acting on the instance.
	pub animation: Option<Box<dyn Animation<InstanceParameter>>>,
	/// Character which this instance belongs to.
	pub character: CharacterName,
	/// Position of the image centre in pixels.
	/// This determines the centre of rotation and scaling.
	pub centre_position: (f32, f32),
	/// Image that this instance draws to the screen.
	pub image: Image,
	/// Position on the screen in pixels.
	pub position: (f32, f32),
	/// Amount the image is scaled by.
	pub scale: (f32, f32),
	/// Whether the instance is visible.
	pub visible: bool,
	/// The colour of the image.
	pub colour: [f32; 4],
	/// 'To Be Killed' - Whether this instance should be removed after the animation finished.
	pub tbk: bool,
}

impl Instance {
	/// Creates a new instance.
	fn new(script: &Script, character: CharacterName, state: &StateName, position: (f32, f32)) -> Self {
		let state = &script.characters[(&character, state)];
		let image = script.images.get(&state.image).unwrap_or_else(||
			panic!("Image at path: {:?}, is not loaded", &state.image)).clone();
		let centre_position = state.centre_position.map(|(x, y)| (x as f32, y as f32))
			.unwrap_or_else(|| (image.width() as f32 / 2.0, image.height() as f32 / 2.0));
		Instance { animation: None, character, centre_position, colour: [1.0; 4], image, position, scale: state.scale, visible: true, tbk: false }
	}

	/// The instance progresses any animation it contains.
	fn update(&mut self, ctx: &mut ggez::Context) {
		if self.animation.is_some() {
			let mut parameters = self.create_parameter();
			match self.animation.as_mut().unwrap().update(&mut parameters, ctx) {
				AnimationState::Continue => self.update_with_parameter(parameters),
				AnimationState::Finished => {
					self.animation.take().unwrap().finish(&mut parameters);
					self.update_with_parameter(parameters);
				}
			}
		}
	}

	/// Draws the instance to the screen.
	fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		let (centre_x, centre_y) = self.centre_position;
		let offset_x = centre_x / self.image.width() as f32;
		let offset_y = centre_y / self.image.height() as f32;

		let (scale_x, scale_y) = self.scale;
		let (position_x, position_y) = self.position;
		let draw_params = graphics::DrawParam::new()
			.dest([position_x, position_y])
			.offset([offset_x, offset_y])
			.scale([scale_x, scale_y])
			.color(self.colour.into());
		graphics::draw(ctx, &self.image, draw_params)
	}

	/// Adds an animation onto the Instance.
	/// If an animation is already present, it is finished before the new one is applied.
	fn add_animation(&mut self, animation: Box<dyn Animation<InstanceParameter>>) {
		if let Some(old_animation) = self.animation.replace(animation) {
			let mut parameters = self.create_parameter();
			old_animation.finish(&mut parameters);
			self.update_with_parameter(parameters);
		}
	}

	/// Finish any animation the Instance has.
	fn finish_animation(&mut self) {
		if let Some(animation) = self.animation.take() {
			let mut parameters = self.create_parameter();
			animation.finish(&mut parameters);
			self.update_with_parameter(parameters);
		}
	}

	/// Creates a parameter struct that will be given to the animation.
	fn create_parameter(&self) -> InstanceParameter {
		InstanceParameter {
			centre_position: self.centre_position,
			image: self.image.clone(),
			position: self.position,
			scale: self.scale,
			visible: self.visible,
			colour: self.colour,
		}
	}

	/// Uses a parameter to update the Instance's own values.
	fn update_with_parameter(&mut self, parameters: InstanceParameter) {
		self.centre_position = parameters.centre_position;
		self.image = parameters.image;
		self.position = parameters.position;
		self.scale = parameters.scale;
		self.visible = parameters.visible;
		self.colour = parameters.colour;
	}
}

/// Holds all the current instances.
#[derive(Debug, Default)]
pub struct Stage(pub HashMap<InstanceName, Instance>);

impl Stage {
	/// Runs all the animations that have been applied onto the instances.
	pub fn update(&mut self, ctx: &mut ggez::Context) {
		let Stage(stage) = self;
		stage.values_mut().for_each(|instance| instance.update(ctx))
	}

	/// Draws all the instances it contains.
	pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		let Stage(stage) = self;
		stage.values().filter(|instance| instance.visible)
			.map(|instance| instance.draw(ctx)).collect()
	}

	/// Spawns a new instance onto the stage.
	pub fn spawn(&mut self, name: InstanceName, instance: Instance) {
		let Stage(stage) = self;
		stage.insert(name, instance);
	}

	/// Removes an instance from the stage.
	pub fn remove(&mut self, name: &InstanceName) {
		let Stage(stage) = self;
		stage.remove(name);
	}

	/// Finishes any animations that are currently on the instances.
	pub fn finish_animation(&mut self) {
		let Stage(stage) = self;
		stage.retain(|_, instance| {
			instance.finish_animation();
			!instance.tbk
		})
	}
}

impl Index<&InstanceName> for Stage {
	type Output = Instance;

	fn index(&self, index: &InstanceName) -> &Self::Output {
		let Stage(stage) = self;
		stage.get(index).unwrap_or_else(||
			panic!("Instance: {:?}, does not exist in stage", index))
	}
}

impl IndexMut<&InstanceName> for Stage {
	fn index_mut(&mut self, index: &InstanceName) -> &mut Self::Output {
		let Stage(stage) = self;
		stage.get_mut(index).unwrap_or_else(||
			panic!("Instance: {:?}, does not exist in stage", index))
	}
}

/// Holds all the characters and their respective states.
#[derive(Debug, Default)]
pub struct Characters(pub HashMap<CharacterName, HashMap<StateName, State>>);

impl Characters {
	/// Adds a character with a map of its states.
	pub fn insert(&mut self, name: CharacterName, states: HashMap<StateName, State>) {
		let Characters(characters) = self;
		characters.insert(name, states);
	}
}

impl Index<(&CharacterName, &StateName)> for Characters {
	type Output = State;

	fn index(&self, (character, state): (&CharacterName, &StateName)) -> &Self::Output {
		let Characters(characters) = self;
		characters.get(character).unwrap_or_else(|| panic!("Character: {:?}, does not exist in map", character))
			.get(state).unwrap_or_else(|| panic!("State: {:?}, does not exist for character: {:?}", state, character))
	}
}
