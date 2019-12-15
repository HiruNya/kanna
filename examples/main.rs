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
label bruh-moment-sigh
"Don't sigh me!"
jump bruh-moment-end

label bruh-moment-rest
"Bruh moments are indeed for resting."
jump bruh-moment-end

label bruh-moment-end
"The weather sure is nice today."
	"#;

	let script = parser::parse(script).unwrap();
	kanna::game::run(script, settings)
}
