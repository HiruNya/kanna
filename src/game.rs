use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

use ggez::{self, Context, event, graphics, input};

use crate::{Characters, Command, History, Label, Render, Script, ScriptState, Settings, Target};

#[derive(Debug)]
pub struct GameState {
	script: Script,
	settings: Settings,
	history: History,
	state: ScriptState,
	render: Render,
	reload: bool,
}

impl GameState {
	pub fn load(ctx: &mut ggez::Context, script: Script,
	            settings: Settings, mut load_history: History) -> Self {
		let history = History::default();
		let (state, render) = (ScriptState::default(), Render::default());
		let mut state = GameState { script, settings, history, state, render, reload: false };

		load_history.divergences.reverse();
		state.state.next_target = Some(Target::default());
		while state.history.execution_count < load_history.execution_count {
			match state.script[&state.state.target] {
				Command::Diverge(_) => state.diverge(ctx,
					&load_history.divergences.pop().unwrap()),
				_ => state.advance(ctx),
			}
		}

		assert!(load_history.divergences.is_empty());
		state
	}

	pub fn advance(&mut self, ctx: &mut ggez::Context) {
		self.render.stage.finish_animation();
		match &mut self.render.text {
			Some(text) if !text.is_finished() => text.finish(),
			_ => loop {
				self.history.execution_count += 1;
				self.state.target = self.state.next_target.take()
					.unwrap_or(self.state.target.next());

				let command = &self.script[&self.state.target];
				command.execute(ctx, &mut self.state,
					&mut self.render, &self.script, &self.settings);

				match command {
					Command::Pause => break,
					Command::Diverge(_) => break,
					Command::Dialogue(_, _) => break,
					_ => (),
				}
			},
		}
	}

	/// Jumps to a selected label in a divergence.
	pub fn diverge(&mut self, ctx: &mut ggez::Context, label: &Label) {
		let target = self.script.labels[label].clone();
		self.history.divergences.push(label.clone());
		self.state.next_target = Some(target);
		self.render.branches.clear();
		self.advance(ctx);
	}
}

impl event::EventHandler for GameState {
	fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
		rate(ctx, self.settings.text_speed, |_|
			Ok(self.render.text.as_mut().map(|text| text.step())))?;
		self.state.sounds.retain(ggez::audio::SoundSource::playing);
		self.render.stage.update(ctx);
		Ok(())
	}

	fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
		graphics::clear(ctx, graphics::BLACK);
		self.render.background.as_ref().map(|image| graphics::draw(ctx,
			image, graphics::DrawParam::new())).transpose()?;
		self.render.stage.draw(ctx)?;
		self.render.character.as_ref().map(|text| text.draw(ctx)).transpose()?;
		self.render.text.as_ref().map(|text| text.draw(ctx)).transpose()?;
		self.render.branches.iter().try_for_each(|(button, _)| button.draw(ctx))?;
		self.render.shadow_bars.iter().try_for_each(|bar| {
			let bar = graphics::Mesh::new_rectangle(ctx,
				graphics::DrawMode::fill(), *bar, graphics::BLACK)?;
			graphics::draw(ctx, &bar, graphics::DrawParam::new())
		})?;
		graphics::present(ctx)
	}

	fn mouse_button_down_event(&mut self, ctx: &mut ggez::Context,
	                           _: input::mouse::MouseButton, x: f32, y: f32) {
		let (x, y) = transform(ctx, (x, y));
		match self.script[&self.state.target] {
			Command::Diverge(_) => {
				let label = self.render.branches.iter()
					.find(|(button, _)| button.rectangle().contains([x, y]));
				label.map(|(_, label)| label).cloned()
					.map(|label| self.diverge(ctx, &label));
			}
			_ => self.advance(ctx),
		}
	}

	fn mouse_motion_event(&mut self, ctx: &mut ggez::Context, x: f32, y: f32, _: f32, _: f32) {
		self.render.branches.iter_mut().for_each(|(button, _)|
			button.update(transform(ctx, (x, y))));
	}

	fn key_down_event(&mut self, ctx: &mut Context, key: event::KeyCode,
	                  modifiers: event::KeyMods, _: bool) {
		if self.settings.developer {
			if modifiers.contains(event::KeyMods::CTRL) {
				if key == event::KeyCode::R {
					save_history(ctx, &self.settings, &self.history);
					self.reload = true;
					event::quit(ctx);
				}
			}
		}
	}

	fn quit_event(&mut self, ctx: &mut Context) -> bool {
		save_history(ctx, &self.settings, &self.history);
		false
	}

	fn resize_event(&mut self, ctx: &mut ggez::Context, width: f32, height: f32) {
		let window_ratio = width / height;
		let view_ratio = self.settings.width / self.settings.height;
		graphics::set_screen_coordinates(ctx, match view_ratio < window_ratio {
			true => {
				let (screen_width, view_height) = (height * view_ratio, self.settings.height);
				let offset = (width - screen_width) * (self.settings.width / screen_width) / 2.0;
				self.render.shadow_bars[1] = [self.settings.width, 0.0, offset, view_height].into();
				self.render.shadow_bars[0] = [-offset, 0.0, offset, view_height].into();
				[-offset, 0.0, self.settings.width + offset * 2.0, view_height]
			}
			false => {
				let (screen_height, view_width) = (width * view_ratio.recip(), self.settings.width);
				let offset = (height - screen_height) * (self.settings.height / screen_height) / 2.0;
				self.render.shadow_bars[1] = [0.0, self.settings.height, view_width, offset].into();
				self.render.shadow_bars[0] = [0.0, -offset, view_width, offset].into();
				[0.0, -offset, view_width, self.settings.height + offset * 2.0]
			}
		}.into()).unwrap();
	}
}

pub fn rate<F, R>(ctx: &mut ggez::Context, rate: u32, mut function: F) -> ggez::GameResult
	where F: FnMut(&mut ggez::Context) -> ggez::GameResult<R> {
	Ok(while ggez::timer::check_update_time(ctx, rate) { function(ctx)?; })
}

/// Transforms absolute coordinates into screen coordinates.
pub fn transform(ctx: &ggez::Context, (x, y): (f32, f32)) -> (f32, f32) {
	let screen = graphics::screen_coordinates(ctx);
	let (width, height) = graphics::drawable_size(ctx);
	(screen.x + (screen.w / width) * x, screen.y + (screen.h / height) * y)
}

pub fn run<F>(settings: Settings, mut script: F) -> ggez::GameResult
	where F: FnMut(&mut ggez::Context, &Settings) -> ggez::GameResult<(Script, History)> {
	let ctx = ggez::ContextBuilder::new("kanna", "kanna")
		.window_mode(ggez::conf::WindowMode {
			resizable: true,
			width: settings.width,
			height: settings.height,
			..ggez::conf::WindowMode::default()
		});

	let (ctx, event_loop) = &mut ctx.build()?;
	settings.resource_paths.iter().map(std::path::PathBuf::from)
		.for_each(|path| ggez::filesystem::mount(ctx, path.as_path(), true));

	loop {
		let (script, history) = script(ctx, &settings)?;
		let mut state = GameState::load(ctx, script, settings.clone(), history);
		event::run(ctx, event_loop, &mut state)?;
		if !state.reload { break Ok(()); }
		ctx.continuing = true;
	}
}

/// Loads a script from a given path. No resources are loaded.
/// Loading referenced resources is performed using [`load_resources`](fn.load_resources.html).
pub fn load_script<P: Into<PathBuf>>(ctx: &mut ggez::Context, path: P) -> ggez::GameResult<Script> {
	let path = &path.into();
	crate::parser::parse(&read_string(ctx, path)?).map_err(|error|
		panic!("Failed to parse script at: {}, because: {:?}", path.display(), error))
}

/// Loads a set of characters from a given path. Characters are formatted in the TOML format.
pub fn load_characters<P: Into<PathBuf>>(ctx: &mut ggez::Context, path: P) -> ggez::GameResult<Characters> {
	let path = &path.into();
	toml::from_str(&read_string(ctx, path)?).map_err(|error|
		panic!("Failed to parse character set at: {}, because: {}", path.display(), error))
}

/// Reads a file from a given path as a string.
pub fn read_string<P: AsRef<Path>>(ctx: &mut ggez::Context, path: P) -> ggez::GameResult<String> {
	let mut string = String::new();
	let file = &mut ggez::filesystem::open(ctx, path)?;
	file.read_to_string(&mut string)?;
	Ok(string)
}

/// Loads the game history from the save path in the settings.
pub fn load_history(ctx: &mut ggez::Context, settings: &Settings) -> ggez::GameResult<History> {
	let buffer: Result<Vec<_>, _> = ggez::filesystem::open(ctx,
		&settings.save_path)?.bytes().collect();
	toml::from_slice(&buffer?).map_err(|error| {
		let error = format!("Failed to load saved history because: {}", error);
		ggez::GameError::ResourceLoadError(error)
	})
}

/// Saves the game history to the path specified in the settings.
pub fn save_history(ctx: &mut ggez::Context, settings: &Settings, history: &History) {
	let mut file = ggez::filesystem::create(ctx, &settings.save_path).unwrap_or_else(|error|
		panic!("Failed to open file: {}, for saving because: {}", settings.save_path, error));
	file.write_all(&toml::to_vec(history).unwrap_or_else(|error|
		panic!("Failed to serialize history for saving because: {}", error)))
		.unwrap_or_else(|error| panic!("Failed to write save history to file because: {}", error))
}

/// Loads all resources that are referenced in a script.
/// Ignores any resources that have already been loaded.
pub fn load_resources(ctx: &mut ggez::Context, script: &mut Script) -> ggez::GameResult {
	load_images(ctx, script)?;
	load_audio(ctx, script)
}

/// Loads all the images that are referenced in a script.
pub fn load_images(ctx: &mut ggez::Context, script: &mut Script) -> ggez::GameResult {
	let Characters(characters) = &script.characters;
	let paths = characters.values().flat_map(|states|
		states.values()).map(|state| &state.image);
	let paths = Iterator::chain(paths, script.commands.iter()
		.filter_map(|command| match command {
			Command::Stage(path) => Some(path),
			_ => None,
		}));

	Ok(for path in paths {
		if !script.images.contains_key(path) {
			let image = graphics::Image::new(ctx, path)?;
			script.images.insert(path.clone(), image);
		}
	})
}

/// Loads all the audio that is referenced in a script.
pub fn load_audio(ctx: &mut ggez::Context, script: &mut Script) -> ggez::GameResult {
	let script_audio = &mut script.audio;
	script.commands.iter().try_for_each(|command| match command {
		Command::Music(path) | Command::Sound(path) => Ok({
			if !script_audio.contains_key(path) {
				let audio = ggez::audio::SoundData::new(ctx, path)?;
				script_audio.insert(path.clone(), audio);
			}
		}),
		_ => Ok(()),
	})
}
