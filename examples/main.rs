use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");

	let script = Script {
		commands: vec![
			Command::Dialogue(Character("John Wick".into()), "John Wick needs your credit card number and the three digits on the back so he can win this epic victory and take home the bread.".into()),
			Command::Dialogue(Character("Bruh Moment".into()), "Hi, this is a bruh moment.".into()),
			Command::Stage("/background.jpg".into()),
		],
		images: std::collections::HashMap::new(),
	};

	kanna::game::run(script, settings)
}
