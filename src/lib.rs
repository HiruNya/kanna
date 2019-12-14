use std::ops::{Deref, DerefMut, Index, Range};

use ggez::graphics;

pub mod game;

#[derive(Debug)]
pub struct Character(pub String);

#[derive(Debug)]
pub enum Command {
	Dialogue(Character, String),
}

impl Command {
	pub fn execute(&self, _: &mut ScriptState, render: &mut Render, settings: &Settings) {
		match self {
			Command::Dialogue(_, string) => {
				let text = RenderText::new(string.clone(), settings.foreground_colour);
				render.text = Some(TextBox::new(text, (8.0, 8.0), settings.background_colour)
					.size((settings.width - 16.0, settings.height - 16.0)).padding(8.0));
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

#[derive(Debug)]
pub struct Script {
	pub commands: Vec<Command>,
}

impl Index<Target> for Script {
	type Output = Command;

	fn index(&self, Target(index): Target) -> &Self::Output {
		&self.commands[index]
	}
}

#[derive(Debug, Default)]
pub struct ScriptState {
	pub target: Target,
}

#[derive(Debug, Default)]
pub struct Render {
	pub text: Option<TextBox>,
}

#[derive(Debug)]
pub struct RenderText {
	pub string: String,
	pub slice: Range<usize>,
	pub colour: [f32; 4],
}

impl RenderText {
	pub fn new(string: String, colour: [f32; 4]) -> Self {
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
	pub colour: [f32; 4],
	pub position: (f32, f32),
	pub size: Option<(f32, f32)>,
	pub padding: f32,
}

impl TextBox {
	pub fn new(text: RenderText, position: (f32, f32), colour: [f32; 4]) -> Self {
		TextBox { text, colour, position, size: None, padding: 0.0 }
	}

	pub fn size(mut self, size: (f32, f32)) -> Self {
		self.size = Some(size);
		self
	}

	pub fn padding(mut self, padding: f32) -> Self {
		self.padding = padding;
		self
	}

	pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
		let fragment = self.text.fragment();
		let (x, y) = self.position;
		match self.size {
			Some((width, height)) => {
				let rectangle = [x, y, width, height].into();
				let text_box = graphics::Mesh::new_rectangle(ctx,
					graphics::DrawMode::fill(), rectangle, self.colour.into())?;
				graphics::draw(ctx, &text_box, graphics::DrawParam::new())?;

				let bounds = [width - 2.0 * self.padding, height - 2.0 * self.padding];
				graphics::draw(ctx, graphics::Text::new(fragment).set_bounds(bounds,
					graphics::Align::Left), ([x + self.padding, y + self.padding], ))
			}
			None => {
				let text = graphics::Text::new(fragment);
				let (width, height) = text.dimensions(ctx);
				let width = width as f32 + 2.0 * self.padding;
				let height = height as f32 + 2.0 * self.padding;

				let rectangle = [x, y, width, height].into();
				let text_box = graphics::Mesh::new_rectangle(ctx,
					graphics::DrawMode::fill(), rectangle, self.colour.into())?;
				graphics::draw(ctx, &text_box, graphics::DrawParam::new())?;
				graphics::draw(ctx, &text, ([x + self.padding, y + self.padding], ))
			}
		}
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
pub struct Settings {
	/// The width of the game window.
	pub width: f32,
	/// The height of the game window.
	pub height: f32,
	/// The rate at which characters are displayed.
	pub text_speed: u32,
	/// The colour of background elements such as text boxes.
	pub background_colour: [f32; 4],
	/// The colour of foreground elements such as text.
	pub foreground_colour: [f32; 4],
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			width: 640.0,
			height: 480.0,
			text_speed: 32,
			background_colour: [0.8, 0.8, 0.8, 0.8],
			foreground_colour: [0.0, 0.0, 0.0, 1.0],
		}
	}
}
