use std::collections::HashMap;

use kanna::*;
use kanna::character::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");

	let mut characters = Characters::default();
	characters.insert(CharacterName("Character".into()), {
		let mut states = HashMap::new();
		states.insert(StateName("Happy".into()), CharacterState::new("/character-happy.png").scale((0.5, 0.5)));
		states.insert(StateName("Sad".into()), CharacterState::new("/character-sad.png").scale((0.5, 0.5)));
		states
	});

	kanna::game::run(settings, |ctx| {
		let mut script = kanna::game::load_script(ctx, "/script.txt")?;
		script.characters = characters;
		kanna::game::load_resources(ctx, script)
	})
}
