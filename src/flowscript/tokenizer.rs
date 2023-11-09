use std::{iter::Peekable, str::Chars};

use super::util;

#[derive(Debug, PartialEq)]
pub enum BrState {
    Open,
    Closed,
}

#[derive(Debug, PartialEq)]
pub enum Key {
    Digraph,
    Shape,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Arrow,
    Bracket(BrState),
    Brace(BrState),
    Equals,
    Text(String),
    ReservedText(Key),
    Semicolon,
}

pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        util::extract_until(&mut self.chars, |c| !c.is_whitespace());

        if let Some(next_tok) = self.chars.next() {
            match next_tok {
                '[' => Some(Token::Bracket(BrState::Open)),
                ']' => Some(Token::Bracket(BrState::Closed)),
                '{' => Some(Token::Brace(BrState::Open)),
                '}' => Some(Token::Brace(BrState::Closed)),
                ';' => Some(Token::Semicolon),
                '=' => Some(Token::Equals),
                '-' if self.chars.next_if_eq(&'>').is_some() => Some(Token::Arrow),
                '"' => {
                    let t = Some(Token::Text(
                        util::extract_until(&mut self.chars, |c| *c == '"')
                            .iter()
                            .collect(),
                    ));
                    self.chars.next(); // extract the last '"'
                    t
                }
                ch if next_tok.is_alphanumeric() => {
                    let mut s = String::from(ch);
                    s.push_str(
                        &util::extract_until(&mut self.chars, |c| !c.is_ascii_alphanumeric())
                            .iter()
                            .collect::<String>(),
                    );

                    match &s {
                        s if s.eq_ignore_ascii_case("digraph") => {
                            Some(Token::ReservedText(Key::Digraph))
                        }
                        s if s.eq_ignore_ascii_case("shape") => {
                            Some(Token::ReservedText(Key::Shape))
                        }

                        _ => Some(Token::Text(s)),
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
