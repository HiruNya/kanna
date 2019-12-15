use kanna::*;

pub fn main() -> ggez::GameResult {
	let mut settings = Settings::default();
	settings.resource_paths.push(env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/resources");

	let script = r#"
"John Wick" "John Wick needs your credit card number and the three digits on the back"
"so he can win this epic victory and take home the bread."
"Bruh Moment" "Hi, this is a bruh moment."
stage "/background.jpg"
diverge
	"Sign" bruh-moment-sigh
	"Rest" bruh-moment-rest
"Don't sigh me!"
"Bruh moments are indeed for resting."
	"#;

	let mut script = parser::parse(script).unwrap();
	script.labels.insert(Label("bruh-moment-sigh".into()), Target(5));
	script.labels.insert(Label("bruh-moment-rest".into()), Target(6));
	kanna::game::run(script, settings)
}
