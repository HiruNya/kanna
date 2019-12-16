use std::collections::HashMap;

use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");
	let mut characters = Characters::default();
	characters.add_character("Character", {
		let mut states = HashMap::new();
		states.insert("Happy".into(), State::new("/character-happy.png").scale(0.5, 0.5));
		states.insert("Sad".into(), State::new("/character-sad.png").scale(0.5, 0.5));
		states
	});

	let script = Script {
		commands: vec![
			Command::Dialogue(Character("John Wick".into()), "John Wick needs your credit card number and the three digits on the back so he can win this epic victory and take home the bread.".into()),
			Command::Spawn("Character".into(), "Happy".into(), (320., 240.).into(), None),
			Command::Position("Character".into(), 540., 240.),
			Command::Hide("Character".into()),
			Command::Show("Character".into()),
			Command::Change("Character".into(), "Sad".into()),
			Command::Kill("Character".into()),
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
			labels.insert(Label("bruh-moment-sigh".into()), Target(10));
			labels.insert(Label("bruh-moment-rest".into()), Target(11));
			labels
		},
		images: HashMap::new(),
		characters,
	};

	kanna::game::run(script, settings)
}
