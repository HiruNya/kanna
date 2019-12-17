use crate::{Command, Label, lexer::Lexer, Script, Target};
use crate::character::{CharacterName, InstanceName, StateName};

#[derive(Debug, PartialEq)]
pub enum Token {
	Identifier(String),
	String(String),
	Numeric(f32),
	ScopeOpen,
	ScopeClose,
	BracketOpen,
	BracketClose,
	ListSeparator,
	Terminator,
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
	UnmatchedQuote,
	ExpectedIdentifier,
	ExpectedString,
	ExpectedNumeric,
	Expected(Token),
	UnexpectedToken,
	InvalidCommand,
	InvalidNumeric,
}

pub fn parse(string: &str) -> Result<Script, Vec<ParserError>> {
	let mut errors = Vec::new();
	let mut script = Script::default();
	let lexer = &mut Lexer::new(string);

	loop {
		match parse_command(lexer, &mut script) {
			Ok(false) => (),
			Ok(true) => break,
			Err((error, target)) => {
				lexer.skip_take(target);
				errors.push(error);
			}
		}
	}

	match errors.is_empty() {
		false => Err(errors),
		true => Ok(script)
	}
}

pub fn parse_command(lexer: &mut Lexer, script: &mut Script) -> Result<bool, (ParserError, Token)> {
	let initial = lexer.token().map_err(|error|
		(error, Token::Terminator))?;
	let initial = match initial {
		Some(token) => token,
		None => return Ok(true),
	};

	match initial {
		Token::Terminator => (),
		Token::Identifier(identifier) => match identifier.as_str() {
			"change" => {
				let instance = InstanceName(inline(lexer.string())?);
				let state = StateName(inline(lexer.string())?);
				script.commands.push(Command::Change(instance, state));
			}
			"diverge" => {
				inline(lexer.expect(Token::Terminator))?;
				inline(lexer.expect(Token::ScopeOpen))?;
				parse_diverge(lexer, script).map_err(|error| (error, Token::ScopeClose))?;
			}
			"label" => {
				let label = Label(inline(lexer.identifier())?);
				script.labels.insert(label, Target(script.commands.len()));
			}
			"position" => {
				let instance = InstanceName(inline(lexer.string())?);
				script.commands.push(Command::Position(instance, position(lexer)?));
			}
			"spawn" => {
				let character = CharacterName(inline(lexer.string())?);
				let state = StateName(inline(lexer.string())?);
				let position = position(lexer)?;

				script.commands.push(Command::Spawn(character, state, position,
					match inline(lexer.token())? {
						None | Some(Token::Terminator) => None,
						Some(Token::String(string)) => Some(InstanceName(string)),
						Some(_) => return Err((ParserError::UnexpectedToken, Token::Terminator)),
					}));
			}
			"kill" => script.commands.push(Command::Kill(InstanceName(inline(lexer.string())?))),
			"show" => script.commands.push(Command::Show(InstanceName(inline(lexer.string())?))),
			"hide" => script.commands.push(Command::Hide(InstanceName(inline(lexer.string())?))),
			"stage" => script.commands.push(Command::Stage(inline(lexer.string())?.into())),
			"jump" => script.commands.push(Command::Jump(Label(inline(lexer.identifier())?))),
			"music" => script.commands.push(Command::Music(inline(lexer.string())?.into())),
			"sound" => script.commands.push(Command::Sound(inline(lexer.string())?.into())),
			_ => return Err((ParserError::InvalidCommand, Token::Terminator)),
		}
		Token::String(string) => match lexer.token().map_err(|error| (error, Token::Terminator))? {
			Some(Token::Terminator) =>
				script.commands.push(Command::Dialogue(None, string)),
			Some(Token::String(dialogue)) => {
				let character = Some(CharacterName(string));
				script.commands.push(Command::Dialogue(character, dialogue));
				inline(lexer.expect(Token::Terminator))?;
			}
			_ => return Err((ParserError::Expected(Token::Terminator), Token::Terminator)),
		},
		Token::ScopeOpen => return Err((ParserError::UnexpectedToken, Token::ScopeClose)),
		_ => return Err((ParserError::UnexpectedToken, Token::Terminator)),
	};
	Ok(false)
}

pub fn inline<T>(result: Result<T, ParserError>) -> Result<T, (ParserError, Token)> {
	result.map_err(|error| (error, Token::Terminator))
}

pub fn position(lexer: &mut Lexer) -> Result<(f32, f32), (ParserError, Token)> {
	inline(lexer.expect(Token::BracketOpen))?;
	let position_x = inline(lexer.numeric())?;
	inline(lexer.expect(Token::ListSeparator))?;
	let position_y = inline(lexer.numeric())?;
	inline(lexer.expect(Token::BracketClose))?;
	Ok((position_x, position_y))
}

pub fn parse_diverge(lexer: &mut Lexer, script: &mut Script) -> Result<(), ParserError> {
	let mut branches = Vec::new();
	loop {
		match lexer.token() {
			Ok(Some(Token::ScopeClose)) => {
				script.commands.push(Command::Diverge(branches));
				return Ok(());
			}
			Ok(Some(Token::String(string))) => {
				let identifier = lexer.identifier()?;
				branches.push((string, Label(identifier)));
				lexer.expect(Token::Terminator)?;
			}
			Ok(Some(Token::Terminator)) => (),
			_ => return Err(ParserError::ExpectedString),
		}
	}
}
