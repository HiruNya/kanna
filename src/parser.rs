use crate::{Command, FlagName, Label, lexer::Lexer, Script, Target};
use crate::animation::AnimationDeclaration;
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
	SquareOpen,
	SquareClose,
	ListSeparator,
	Underscore,
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
                let animation = animation(lexer, true)?;
				script.commands.push(Command::Change(instance, state, animation));
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
				let position =  position(lexer)?;
				let animation = animation(lexer, true)?;
				script.commands.push(Command::Position(instance, position, animation));
			}
			"spawn" => {
				let character = CharacterName(inline(lexer.string())?);
				let state = StateName(inline(lexer.string())?);
				let position = position(lexer)?;
                let mut check_with = true;
                let mut is_end = false;
                let instance_name = match inline(lexer.token())? {
					None | Some(Token::Terminator) => {
						is_end = true;
						None
					},
					Some(Token::Identifier(ident)) if ident == "with" => {
						check_with = false;
						None
					}
					Some(Token::String(string)) => Some(InstanceName(string)),
					Some(_) => return Err((ParserError::UnexpectedToken, Token::Terminator)),
				};
				let animation = if !is_end { animation(lexer, check_with)? } else { None };
				script.commands.push(Command::Spawn(character, state, position, instance_name, animation));
			}
			"if" => {
				let flag = FlagName(inline(lexer.identifier())?);
				script.commands.push(Command::If(flag, Label(inline(lexer.identifier())?)));
			}
			"pause" => script.commands.push(Command::Pause),
			"flag" => script.commands.push(Command::Flag(FlagName(inline(lexer.identifier())?))),
			"unflag" => script.commands.push(Command::Unflag(FlagName(inline(lexer.identifier())?))),
			"kill" => script.commands.push(Command::Kill(InstanceName(inline(lexer.string())?), animation(lexer, true)?)),
			"show" => script.commands.push(Command::Show(InstanceName(inline(lexer.string())?), animation(lexer, true)?)),
			"hide" => script.commands.push(Command::Hide(InstanceName(inline(lexer.string())?), animation(lexer, true)?)),
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

pub fn animation(lexer: &mut Lexer, check_with: bool) -> Result<Option<AnimationDeclaration>, (ParserError, Token)> {
	if check_with {
		match inline(lexer.token())? {
			None | Some(Token::Terminator) => return Ok(None),
			Some(Token::Identifier(string)) if string == "with" => {},
			Some(token) => return Err((ParserError::UnexpectedToken, token)),
		}
	}
	let name = inline(lexer.identifier())?;
	inline(lexer.expect(Token::SquareOpen))?;
	let mut arguments = Vec::new();
	while let Some(token) = inline(lexer.token())? {
		if token == Token::SquareClose { break }
		if !arguments.is_empty() {
			inline(lexer.expect(Token::ListSeparator))?
		}
		let arg = match token {
			Token::Underscore => None,
			Token::Numeric(n) => Some(n),
			token => return Err((ParserError::UnexpectedToken, token)),
		};
		arguments.push(arg)
	}
	Ok(Some(AnimationDeclaration { name, arguments }))
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
