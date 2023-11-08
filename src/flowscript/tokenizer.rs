use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum BrState {
    Open,
    Closed,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Arrow,
    Bracket(BrState),
    Brace(BrState),
    Digraph,
    Text(String),
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

    fn extract_string_until<P>(&mut self, predicate: P) -> String
    where
        P: Fn(&char) -> bool,
    {
        let mut s = String::new();
        while let Some(ch) = self.chars.peek() {
            if predicate(ch) {
                break;
            }
            s.push(self.chars.next().unwrap());
        }
        s
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.extract_string_until(|c| !c.is_whitespace());

        if let Some(next_tok) = self.chars.next() {
            match next_tok {
                '[' => Some(Token::Bracket(BrState::Open)),
                ']' => Some(Token::Bracket(BrState::Closed)),
                '{' => Some(Token::Brace(BrState::Open)),
                '}' => Some(Token::Brace(BrState::Closed)),
                '-' if self.chars.next_if_eq(&'>').is_some() => Some(Token::Arrow),
                '"' => {
                    let t = Some(Token::Text(self.extract_string_until(|c| *c == '"')));
                    self.chars.next(); // extract the last '"'
                    t
                }
                ch if next_tok.is_alphanumeric() => {
                    let mut s = String::from(ch);
                    s += &self.extract_string_until(|c| !c.is_alphanumeric());

                    Some(Token::Text(s))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
