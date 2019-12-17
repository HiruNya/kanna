use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use ggez::graphics;
use serde::Deserialize;

use crate::Script;

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
#[serde(transparent)]
pub struct CharacterName(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct InstanceName(pub String);

#[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
#[serde(transparent)]
pub struct StateName(pub String);

/// A state represents a possible character image.
#[derive(Debug, Deserialize, Clone)]
pub struct CharacterState {
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

impl CharacterState {
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
	/// Character which this instance belongs to.
	pub character: CharacterName,
	/// Position of the image centre in pixels.
	/// This determines the centre of rotation and scaling.
	pub centre_position: (f32, f32),
	/// Image that this instance draws to the screen.
	pub image: graphics::Image,
	/// Position on the screen in pixels.
	pub position: (f32, f32),
	/// Amount the image is scaled by.
	pub scale: (f32, f32),
	/// Whether the instance is visible.
	pub visible: bool,
}

impl Instance {
	/// Creates a new instance.
	pub fn new(script: &Script, character: CharacterName, state: &StateName, position: (f32, f32)) -> Self {
		let state = &script.characters[(&character, state)];
		let image = script.images.get(&state.image).unwrap_or_else(||
			panic!("Image at path: {:?}, is not loaded", &state.image)).clone();
		let centre_position = state.centre_position.map(|(x, y)| (x as f32, y as f32))
			.unwrap_or_else(|| (image.width() as f32 / 2.0, image.height() as f32 / 2.0));
		Instance { character, centre_position, image, position, scale: state.scale, visible: true }
	}

	/// Draws the instance to the screen.
	pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		let (centre_x, centre_y) = self.centre_position;
		let offset_x = centre_x / self.image.width() as f32;
		let offset_y = centre_y / self.image.height() as f32;

		let (scale_x, scale_y) = self.scale;
		let (position_x, position_y) = self.position;
		let draw_params = graphics::DrawParam::new()
			.dest([position_x, position_y])
			.offset([offset_x, offset_y])
			.scale([scale_x, scale_y]);
		graphics::draw(ctx, &self.image, draw_params)
	}
}

/// Holds all the current instances.
#[derive(Debug, Default)]
pub struct Stage(pub HashMap<InstanceName, Instance>);

impl Stage {
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
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Characters(pub HashMap<CharacterName, HashMap<StateName, CharacterState>>);

impl Characters {
	/// Adds a character with a map of its states.
	pub fn insert(&mut self, name: CharacterName, states: HashMap<StateName, CharacterState>) {
		let Characters(characters) = self;
		characters.insert(name, states);
	}
}

impl Index<(&CharacterName, &StateName)> for Characters {
	type Output = CharacterState;

	fn index(&self, (character, state): (&CharacterName, &StateName)) -> &Self::Output {
		let Characters(characters) = self;
		characters.get(character).unwrap_or_else(|| panic!("Character: {:?}, does not exist in map", character))
			.get(state).unwrap_or_else(|| panic!("State: {:?}, does not exist for character: {:?}", state, character))
	}
}
