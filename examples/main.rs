use kanna::*;

pub fn main() -> ggez::GameResult {
	let script = Script {
		commands: vec![
			Command::Dialogue(Character("John Wick".into()), "John Wicks needs your credit card number and the three digits on the back so he can win this epic victory and take home the bread.".into()),
			Command::Dialogue(Character("Bruh Moment".into()), "Hi, this is a bruh moment.".into()),
		],
	};

	let settings = Settings::default();
	kanna::game::run(script, settings)
}
