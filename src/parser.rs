use crate::{Label, lexer::Lexer, Target};

use super::{Character, Command, Script};

#[derive(Debug, PartialEq)]
pub enum Token {
	Identifier(String),
	String(String),
	ScopeOpen,
	ScopeClose,
	Terminator,
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
	UnmatchedQuote,
	ExpectedIdentifier,
	ExpectedString,
	Expected(Token),
	UnexpectedToken,
	InvalidCommand,
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
		Token::Identifier(identifier) => match identifier.as_str() {
			"stage" => {
				let path = lexer.string().map_err(|error| (error, Token::Terminator))?;
				script.commands.push(Command::Stage(path.into()));
			}
			"diverge" => {
				lexer.expect(Token::Terminator).map_err(|error| (error, Token::Terminator))?;
				lexer.expect(Token::ScopeOpen).map_err(|error| (error, Token::Terminator))?;
				parse_diverge(lexer, script).map_err(|error| (error, Token::ScopeClose))?;
			}
			"label" => {
				let label = lexer.identifier().map_err(|error| (error, Token::Terminator))?;
				script.labels.insert(Label(label), Target(script.commands.len()));
			}
			"jump" => {
				let label = lexer.identifier().map_err(|error| (error, Token::Terminator))?;
				script.commands.push(Command::Jump(Label(label)));
			}
			_ => return Err((ParserError::InvalidCommand, Token::Terminator)),
		}
		Token::String(string) => match lexer.token().map_err(|error| (error, Token::Terminator))? {
			Some(Token::Terminator) =>
				script.commands.push(Command::Dialogue(None, string)),
			Some(Token::String(dialogue)) => {
				let character = Some(Character(string));
				script.commands.push(Command::Dialogue(character, dialogue));
				lexer.expect(Token::Terminator).map_err(|error| (error, Token::Terminator))?;
			}
			_ => return Err((ParserError::Expected(Token::Terminator), Token::Terminator)),
		},
		Token::ScopeOpen => return Err((ParserError::UnexpectedToken, Token::ScopeClose)),
		Token::ScopeClose => return Err((ParserError::UnexpectedToken, Token::Terminator)),
		Token::Terminator => (),
	};
	Ok(false)
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
			_ => return Err(ParserError::ExpectedString),
		}
	}
}
