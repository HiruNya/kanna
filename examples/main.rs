use std::collections::HashMap;

use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");
	let mut characters = Characters::default();
	characters.insert(CharacterName("Character".into()), {
		let mut states = HashMap::new();
		states.insert(StateName("Happy".into()), State::new("/character-happy.png").scale((0.5, 0.5)));
		states.insert(StateName("Sad".into()), State::new("/character-sad.png").scale((0.5, 0.5)));
		states
	});

	let script = Script {
		characters,
		commands: vec![
			Command::Dialogue(Some(CharacterName("John Wick".into())),
				"John Wick needs your credit card number and the three digits on the back so he can win this epic victory and take home the bread.".into()),
			Command::Spawn(CharacterName("Character".into()), StateName("Happy".into()), (320.0, 240.0), None),
			Command::Position(InstanceName("Character".into()), (540.0, 240.0)),
			Command::Hide(InstanceName("Character".into())),
			Command::Show(InstanceName("Character".into())),
			Command::Change(InstanceName("Character".into()), StateName("Sad".into())),
			Command::Kill(InstanceName("Character".into())),
			Command::Dialogue(Some(CharacterName("Bruh Moment".into())), "Hi, this is a bruh moment.".into()),
			Command::Stage("/background.jpg".into()),
			Command::Diverge(vec![
				("Sigh".into(), Label("bruh-moment-sigh".into())),
				("Rest".into(), Label("bruh-moment-rest".into())),
			]),
			Command::Dialogue(Some(CharacterName("Bruh Moment".into())), "Don't sigh me!".into()),
			Command::Dialogue(Some(CharacterName("Bruh Moment".into())), "Bruh moments are indeed for resting.".into()),
		],
		labels: {
			let mut labels = HashMap::new();
			labels.insert(Label("bruh-moment-sigh".into()), Target(10));
			labels.insert(Label("bruh-moment-rest".into()), Target(11));
			labels
		},
		images: HashMap::new(),
		audio: HashMap::new(),
	};

	kanna::game::run(script, settings)
}
