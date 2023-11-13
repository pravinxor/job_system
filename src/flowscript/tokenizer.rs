use std::iter::Peekable;

use super::util;

#[derive(Debug, PartialEq)]
pub enum BrState {
    Open,
    Closed,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Key {
    Digraph,
    Shape,
    Data,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Arrow,
    Bracket(BrState),
    Brace(BrState),
    Comma,
    Equals,
    Text(String),
    ReservedText(Key),
    Semicolon,
}

pub struct Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    chars: Peekable<I>,
}

impl<I> Tokenizer<I>
where
    I: Iterator<Item = char>,
{
    fn new(chars: I) -> Self {
        Self {
            chars: chars.peekable(),
        }
    }
}

// Extension trait for convenience
pub trait TokenizerAdapter: Iterator<Item = char> + Sized {
    fn tokens(self) -> Tokenizer<Self> {
        Tokenizer::new(self)
    }
}
impl<I: Iterator<Item = char>> TokenizerAdapter for I {}

fn extract_str_until<I, P>(iter: &mut Peekable<I>, predicate: P) -> String
where
    I: Iterator<Item = char>,
    P: Fn(&char) -> bool,
{
    let mut s = String::new();
    let mut is_escaped = false;
    while iter
        .peek()
        .filter(|e| is_escaped || !predicate(e))
        .is_some()
    {
        let c = iter.next().unwrap();
        is_escaped = c == '\\';

        if !is_escaped {
            s.push(c)
        }
    }
    s
}

impl<I> Iterator for Tokenizer<I>
where
    I: Iterator<Item = char>,
{
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
                ',' => Some(Token::Comma),
                '-' if self.chars.next_if_eq(&'>').is_some() => Some(Token::Arrow),
                '"' => {
                    let t = Some(Token::Text(extract_str_until(&mut self.chars, |c| {
                        *c == '"'
                    })));
                    self.chars.next(); // extract the last '"'
                    t
                }
                ch if next_tok.is_alphanumeric() => {
                    let mut s = String::from(ch);
                    s += &extract_str_until(&mut self.chars, |c| {
                        !c.is_ascii_alphanumeric() && *c != '_'
                    });

                    match &s {
                        s if s.eq_ignore_ascii_case("digraph") => {
                            Some(Token::ReservedText(Key::Digraph))
                        }
                        s if s.eq_ignore_ascii_case("shape") => {
                            Some(Token::ReservedText(Key::Shape))
                        }
                        s if s.eq_ignore_ascii_case("data") => Some(Token::ReservedText(Key::Data)),
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
