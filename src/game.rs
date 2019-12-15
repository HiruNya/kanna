use ggez::{self, Context, event, graphics, input};

use super::*;

#[derive(Debug)]
pub struct GameState {
	script: Script,
	settings: Settings,
	state: ScriptState,
	render: Render,
}

impl GameState {
	pub fn new(script: Script, settings: Settings) -> Self {
		let mut render = Render::default();
		let mut state = ScriptState::default();
		script[&state.target].execute(&mut state,
			&mut render, &script, &settings);
		GameState { script, settings, state, render }
	}

	pub fn advance(&mut self) {
		match &mut self.render.text {
			Some(text) if !text.is_finished() => text.finish(),
			_ => {
				self.state.target.advance();
				self.jump(self.state.target.clone());
			}
		}
	}

	pub fn jump(&mut self, target: Target) {
		self.state.target = target;
		self.script[&self.state.target].execute(&mut self.state,
			&mut self.render, &self.script, &self.settings)
	}
}

impl event::EventHandler for GameState {
	fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
		rate(ctx, self.settings.text_speed, |_| Ok(self.render.text.as_mut().map(|text| text.step())))?;
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
		graphics::present(ctx)
	}

	fn mouse_button_down_event(&mut self, _: &mut ggez::Context,
	                           _: input::mouse::MouseButton, x: f32, y: f32) {
		match self.script[&self.state.target] {
			Command::Diverge(_) => {
				for (button, label) in &self.render.branches {
					if button.rectangle().contains([x, y]) {
						let target = self.script.labels[&label].clone();
						self.render.branches.clear();
						self.jump(target);
						return;
					}
				}
			}
			_ => self.advance(),
		}
	}

	fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32, _: f32, _: f32) {
		self.render.branches.iter_mut().for_each(|(button, _)| button.update((x, y)));
	}
}

pub fn rate<F, R>(ctx: &mut ggez::Context, rate: u32, mut function: F) -> ggez::GameResult
	where F: FnMut(&mut ggez::Context) -> ggez::GameResult<R> {
	Ok(while ggez::timer::check_update_time(ctx, rate) { function(ctx)?; })
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

	let state = &mut GameState::new(script, settings);
	event::run(ctx, event_loop, state)
}

pub fn load_images(ctx: &mut ggez::Context, script: &mut Script) -> ggez::GameResult {
	script.characters.load_images(&mut script.images, ctx)?;
	for command in &script.commands {
		match command {
			Command::Stage(path) => {
				let image = graphics::Image::new(ctx, path)?;
				script.images.insert(path.clone(), image);
			}
			_ => (),
		}
	};
	Ok(())
}
