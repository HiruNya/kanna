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

	let script = r#"
"John Wick" "John Wick needs your credit card number and the three digits on the back so he can win this epic victory and take home the bread."
spawn "Character" "Happy" (320, 240)
position "Character" (540, 240)
hide "Character"
show "Character"
change "Character" "Sad"
kill "Character"
"Bruh Moment" "Hi, this is a bruh moment."
stage "/background.jpg"

diverge
	"Sigh" bruh-moment-sigh
	"Rest" bruh-moment-rest

label bruh-moment-sigh
"Don't sigh me!"
jump bruh-moment-end

label bruh-moment-rest
"Bruh moments are indeed for resting."
jump bruh-moment-end

label bruh-moment-end
music "/music.ogg"
"The weather sure is nice today."

	"#;

	let mut script = kanna::parser::parse(script).unwrap();
	script.characters = characters;
	kanna::game::run(script, settings)
}
