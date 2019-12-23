use ggez::{self, audio, Context, event, graphics, input};

use super::*;

#[derive(Debug)]
pub struct GameState {
	script: Script,
	settings: Settings,
	state: ScriptState,
	render: Render,
}

impl GameState {
	pub fn new(ctx: &mut ggez::Context, script: Script, settings: Settings) -> Self {
		let (mut state, mut render) = (ScriptState::default(), Render::default());
		script[&state.target].execute(ctx, &mut state, &mut render, &script, &settings);
		GameState { script, settings, state, render }
	}

	pub fn advance(&mut self, ctx: &mut ggez::Context) {
		self.render.stage.finish_animation();
		match &mut self.render.text {
			Some(text) if !text.is_finished() => text.finish(),
			_ => match self.state.next_target.take() {
				Some(target) => self.jump(ctx, target),
				None => {
					self.state.target.advance();
					self.jump(ctx, self.state.target.clone())
				}
			},
		}
	}

	pub fn jump(&mut self, ctx: &mut ggez::Context, target: Target) {
		self.state.target = target;
		self.script[&self.state.target].execute(ctx, &mut self.state,
			&mut self.render, &self.script, &self.settings)
	}
}

impl event::EventHandler for GameState {
	fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
		rate(ctx, self.settings.text_speed, |_|
			Ok(self.render.text.as_mut().map(|text| text.step())))?;
		self.state.sounds.retain(Source::playing);
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
				for (button, label) in &self.render.branches {
					if button.rectangle().contains([x, y]) {
						let target = self.script.labels[&label].clone();
						self.render.branches.clear();
						self.jump(ctx, target);
						return;
					}
				}
			}
			_ => self.advance(ctx),
		}
	}

	fn mouse_motion_event(&mut self, ctx: &mut ggez::Context, x: f32, y: f32, _: f32, _: f32) {
		self.render.branches.iter_mut().for_each(|(button, _)|
			button.update(transform(ctx, (x, y))));
	}

	fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
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

pub fn run(mut script: Script, settings: Settings) -> ggez::GameResult {
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

	load_images(ctx, &mut script)?;
	load_audio(ctx, &mut script)?;

	let state = &mut GameState::new(ctx, script, settings);
	event::run(ctx, event_loop, state)
}

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
		let image = graphics::Image::new(ctx, path)?;
		script.images.insert(path.clone(), image);
	})
}

pub fn load_audio(ctx: &mut ggez::Context, script: &mut Script) -> ggez::GameResult {
	Ok(for command in &script.commands {
		match command {
			Command::Music(path) | Command::Sound(path) => {
				let audio = audio::SoundData::new(ctx, path)?;
				script.audio.insert(path.clone(), audio);
			}
			_ => (),
		}
	})
}
