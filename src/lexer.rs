use std::cmp::Ordering;
use std::iter::Peekable;
use std::str::CharIndices;

use crate::parser::{ParserError, Token};

#[derive(Debug)]
pub struct Lexer<'a> {
	string: &'a str,
	characters: Peekable<CharIndices<'a>>,
	indentation: usize,
	target_indent: usize,
	new_line: bool,
}

impl<'a> Lexer<'a> {
	pub fn new(string: &'a str) -> Self {
		let characters = string.char_indices().peekable();
		Lexer { string, characters, indentation: 0, target_indent: 0, new_line: true }
	}

	pub fn token(&mut self) -> Result<Option<Token>, ParserError> {
		self.next().transpose()
	}

	pub fn identifier(&mut self) -> Result<String, ParserError> {
		match self.next().transpose()? {
			Some(Token::Identifier(identifier)) => Ok(identifier),
			_ => Err(ParserError::ExpectedIdentifier),
		}
	}

	pub fn string(&mut self) -> Result<String, ParserError> {
		match self.next().transpose()? {
			Some(Token::String(string)) => Ok(string),
			_ => Err(ParserError::ExpectedString),
		}
	}

	pub fn expect(&mut self, token: Token) -> Result<(), ParserError> {
		match self.next().transpose()?.as_ref() == Some(&token) {
			false => Err(ParserError::Expected(token)),
			true => Ok(())
		}
	}

	/// Skips all tokens until the target token is consumed.
	pub fn skip_take(&mut self, target: Token) {
		let target = Ok(target);
		while let Some(token) = self.next() {
			if token == target { break; }
		}
	}
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Result<Token, ParserError>;

	fn next(&mut self) -> Option<Self::Item> {
		match usize::cmp(&self.indentation, &self.target_indent) {
			Ordering::Less => {
				self.indentation += 1;
				return Some(Ok(Token::ScopeOpen));
			}
			Ordering::Greater => {
				self.indentation -= 1;
				return Some(Ok(Token::ScopeClose));
			}
			Ordering::Equal => (),
		}

		if self.new_line {
			self.new_line = false;
			let mut target_indent = 0;
			while let Some((_, '\t')) = self.characters.peek() {
				self.characters.next();
				target_indent += 1;
			}

			match self.characters.peek() {
				None | Some((_, '\n')) => (),
				_ => self.target_indent = target_indent,
			}
			return self.next();
		}

		let (start, character) = match self.characters.next() {
			Some((start, character)) => (start, character),
			None if self.target_indent == 0 => return None,
			None => {
				self.target_indent = 0;
				return self.next();
			}
		};

		match character {
			'\n' => {
				self.new_line = true;
				Some(Ok(Token::Terminator))
			}
			'"' => loop {
				let character = self.characters.peek();
				match character {
					Some((_, '"')) => {
						let (index, _) = self.characters.next().unwrap();
						let string = self.string[start + 1..index].to_owned();
						return Some(Ok(Token::String(escape(string))));
					}
					Some((_, '\\')) => {
						self.characters.next();
						self.characters.next()
					}
					None | Some((_, '\n')) => return Some(Err(ParserError::UnmatchedQuote)),
					Some(_) => self.characters.next(),
				};
			},
			_ => match character.is_whitespace() {
				true => self.next(),
				false => {
					while let Some((_, character)) = self.characters.peek() {
						match character.is_whitespace() {
							false => self.characters.next(),
							true => break,
						};
					}

					let end = self.characters.peek().map(|(index, _)| *index);
					let string = &self.string[start..end.unwrap_or(self.string.len())];
					Some(Ok(Token::Identifier(string.to_owned())))
				}
			}
		}
	}
}

pub fn escape(string: String) -> String {
	string.replace("\\n", "\n").replace("\\\"", "\"")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_lexer() {
		assert_eq!(Lexer::new("string").next(), Some(Ok(Token::Identifier("string".to_owned()))));
		assert_eq!(Lexer::new("\"string\"").next(), Some(Ok(Token::String("string".to_owned()))));
		assert_eq!(Lexer::new("\"string\\n\"").next(), Some(Ok(Token::String("string\n".to_owned()))));
		assert_eq!(Lexer::new("\"\\\"\"").next(), Some(Ok(Token::String("\"".to_owned()))));
		assert_eq!(Lexer::new("\"string").next(), Some(Err(ParserError::UnmatchedQuote)));

		assert_eq!(Lexer::new("\t").next(), None);
		assert_eq!(Lexer::new("\t\n").next(), Some(Ok(Token::Terminator)));
		assert_eq!(&Lexer::new("\tstring").collect::<Vec<_>>(), &[Ok(Token::ScopeOpen),
			Ok(Token::Identifier("string".to_owned())), Ok(Token::ScopeClose)]);

		assert_eq!(&Lexer::new("diverge\n\t\"string\"").collect::<Vec<_>>(),
			&[Ok(Token::Identifier("diverge".to_owned())), Ok(Token::Terminator),
				Ok(Token::ScopeOpen), Ok(Token::String("string".to_owned())), Ok(Token::ScopeClose)]);
	}
}
