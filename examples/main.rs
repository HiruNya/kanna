use std::collections::HashMap;

use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");
	let mut characters = Characters::default();
	characters.add_character("Character", {
		let mut states = HashMap::new();
		states.insert("Happy".into(), State::new("/character-happy.png"));
		states
	});

	let script = Script {
		commands: vec![
			Command::Dialogue(Character("John Wick".into()), "John Wick needs your credit card number and the three digits on the back so he can win this epic victory and take home the bread.".into()),
			Command::Spawn("Character".into(), "Happy".into(), None, None),
			Command::Dialogue(Character("Bruh Moment".into()), "Hi, this is a bruh moment.".into()),
			Command::Stage("/background.jpg".into()),
			Command::Diverge(vec![
				("Sigh".into(), Label("bruh-moment-sigh".into())),
				("Rest".into(), Label("bruh-moment-rest".into())),
			]),
			Command::Dialogue(Character("Bruh Moment".into()), "Don't sigh me!".into()),
			Command::Dialogue(Character("Bruh Moment".into()), "Bruh moments are indeed for resting.".into()),
		],
		labels: {
			let mut labels = HashMap::new();
			labels.insert(Label("bruh-moment-sigh".into()), Target(4));
			labels.insert(Label("bruh-moment-rest".into()), Target(5));
			labels
		},
		images: HashMap::new(),
		characters,
	};

	kanna::game::run(script, settings)
}
