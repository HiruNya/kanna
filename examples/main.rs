use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");

	kanna::game::run(settings, |ctx, settings| {
		let mut script = kanna::game::load_script(ctx, "/script.txt")?;
		script.characters = kanna::game::load_characters(ctx, "/characters.toml")?;
		let history = kanna::game::load_history(ctx, settings)
			.unwrap_or_else(|_| History::default());
		kanna::game::load_resources(ctx, &mut script)?;
		Ok((script, history))
	})
}
