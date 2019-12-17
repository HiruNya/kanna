use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");

	kanna::game::run(settings, |ctx| {
		let mut script = kanna::game::load_script(ctx, "/script.txt")?;
		script.characters = kanna::game::load_characters(ctx, "/characters.toml")?;
		kanna::game::load_resources(ctx, script)
	})
}
