//! The module for lexer related functions and types.
use std::{fmt::{self, Display, Formatter}, iter::Peekable, ops::Index, str::{CharIndices, Chars}};

use symbol::Symbol;
use token::Token;

pub mod token;
pub mod symbol;

/// Characters allowed in keywords
const KEYWORD_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789-";
/// Characters allowed in symbols
const SYMBOL_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-+*/|<>=!?@#$%";
/// Characters scapable in strings
const ESCAPABLE_CHARS: &'static str = "\"ntr\\";
/// Characters that indicate the end of a token
const TK_END_CHARS: &'static str = " \n\t\r(){}[]\";,";

pub struct Lexer<'source> {
    source: &'source str,
    index: CharIndices<'source>,
    current: char,
    current_index: usize,
    current_line: usize,
    current_column: usize,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl<'source> Lexer<'source> {
    /// Builds a new lexer from a source string.
    pub fn new(source: &'source str) -> Self {
        let mut index = source.char_indices(); 
        let (i, c) = index.next().unwrap_or((0, '\0'));

        Self {
            source, // TODO: Remove \r
            current: c,
            current_index: i,
            index,
            current_line: 1,
            current_column: 1,
        }
    }

    /// Advances the lexer to the next character.
    fn advance(&mut self) -> Option<char> {
        self.index.next().map(|(i, c)| {
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            } else {
                self.current_column += 1;
            }

            self.current = c;
            self.current_index = i;

            c
        })
    }

    /// Advances the lexer n characters
    fn advancen(&mut self, n: usize) -> bool {
        if n == 0 {
            return true;
        }

        self.index.nth(n-1).map(|(i, c)| {
            self.current = c;
            self.current_index = i;
            self.current_column += n;
        }).is_some()
    }

    #[inline]
    fn current(&self) -> char {
        self.current
    }

    /// Returns the substring from the current character to the n-th character.
    fn currentn(&self, n: usize) -> &'source str {
        let end = self.index.clone()
            .nth(n-1)
            .map(|(i, _)| i)
            .unwrap_or(self.source.len());

        &self.source[self.current_index..end]
    }

    /// Returns the next character without advancing.
    #[inline]
    fn peek(&self) -> Option<char> {
        self.index.clone().next().map(|(_, c)| c)
    }

    /// Returns the position of the lexer in the source code.
    #[inline]
    fn position(&self) -> Position {
        Position {
            line: self.current_line,
            column: self.current_column,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            match self.current {
                // Skip whitespace
                c if c.is_whitespace() => { self.advance(); },
                // Parse a scope start
                '(' | '{' | '[' => tokens.push(Token::Open(self.current)),
                // Parse a scope end
                ')' | '}' | ']' => tokens.push(Token::Close(self.current)),
                // Parse a string
                '"' => {
                    let string = self.lex_string()?;
                    tokens.push(string);
                },
                // Parse a keyword
                ':' => {
                    let kw = self.lex_keyword()?;
                    tokens.push(kw);
                },
                // Parse a character
                '\\' => {
                    let (ch, rest) = lex_char(chars)?;
                    tokens.push(ch);
                    chars = rest;
                },
                // Parse a comment
                ';' => {
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            self.advance();
                        }
                    }
                },
                // Parse a number
                c if ((c == '-' || c == '.') && chars.peek().is_some_and(|c| c.is_numeric())) || c.is_numeric() => {
                    let (number, rest) = lex_number(chars, c)?;
                    tokens.push(number);
                    chars = rest;
                },
                // Parse a symbol
                c if SYMBOL_CHARS.contains(c) => {
                    let (symbol, rest) = lex_symbol(chars, c)?;
                    tokens.push(Token::Symbol(symbol));
                    chars = rest;
                },
                // Error on unexpected character
                _ => return Err(format!("Unexpected character: {}", c)),
            };
        }    

        Ok(tokens)
    }

    /// This expects `current` to be `"`. It will consume the string and return a token.
    /// The lexer will be at the next character after the closing `"`.
    fn lex_string(&mut self) -> Result<Token, String> {
        let mut string = String::new();
        //let start = self.position();

        loop {
            match self.advance() {
                None => return Err(format!("Unexpected end of input, expected `\"` at {}", self.position())),
                Some('\\') => match self.advance() {
                    None => return Err(format!("Unexpected end of input, expected `n`, `t`, `r`, `\\` or `\"` at {}", self.position())),
                    Some(c) if ESCAPABLE_CHARS.contains(c) => string.push(c),
                    Some(c) => return Err(format!("Unexpected escape character: {} at {}", c, self.position())),
                },
                Some('"') => { self.advance(); break },
                Some(c) => string.push(c),
            }
        }

        //let end = self.position();

        Ok(Token::String(string))
    }

    /// This expects `current` to be `:`. It will consume the keyword and return it.
    /// The lexer will be at the next character after the keyword.
    fn lex_keyword(&mut self) -> Result<Token, String> {
        let mut keyword = String::new();
    
        loop {
            match self.advance() {
                Some(c) if KEYWORD_CHARS.contains(c) => {
                    keyword.push(c);
                },
                Some(c) if TK_END_CHARS.contains(c) => {
                    if keyword.is_empty() {
                        return Err(format!("Empty keyword at {}", self.position()));
                    }
                    break;
                },
                Some(c) => return Err(format!("Unexpected character: {} at {} while parsing the keyword `:{}`", c, self.position(), keyword)),
                None => {
                    if keyword.is_empty() {
                        return Err(format!("Empty keyword at {}", self.position()));
                    }
                    break;
                },
            }
        }

        Ok(Token::Keyword(keyword))
    }
}

fn lex_symbol(mut source: Peekable<Chars>, first: char) -> Result<(Symbol, Peekable<Chars>), String> {
    let mut parts = vec![];
    let mut current = first.to_string();

    loop {
        match source.peek() {
            Some(&c) if SYMBOL_CHARS.contains(c) => {
                current.push(c);
                source.next();
            },
            Some('.') => {
                parts.push(current);
                current = String::new();
                source.next();
            },
            _ => {
                if current.is_empty() {
                    return Err(format!("A symbol can't end with a `.`"));
                }

                parts.push(current);
                break;
            },
        }
    }

    let head = parts.remove(0);

    Ok((Symbol { head, tail: parts }, source))
}

fn lex_number(mut source: Peekable<Chars>, first: char) -> Result<(Token, Peekable<Chars>), String> {
    let mut number = first.to_string();

    loop {
        match source.peek() {
            Some(&c) if c.is_numeric() => {
                number.push(c);
                source.next();
            },
            Some(&'.') => {
                number.push('.');
                source.next();
            },
            _ => break,
        }
    }

    if number.chars().filter(|&c| c == '.').count() > 1 {
        return Err(format!("Invalid number: {}", number));
    }

    let tk = if number.contains('.') {
        Token::Float(number.parse().unwrap())
    } else {
        Token::Integer(number.parse().unwrap())
    };

    Ok((tk, source))
}

fn lex_char(mut source: Peekable<Chars>) -> Result<(Token, Peekable<Chars>), String> {
    let mut ch = String::new();

    while let Some(c) = source.next() {
        if c == ' ' {
            break;
        } else {
            ch.push(c);
        }
    }

    let c = match ch.as_str() {
        "newline" => '\n',
        "return" => '\r',
        "tab" => '\t',
        "space" => ' ',
        c if c.len() == 1 => c.chars().next().unwrap(), 
        _ => return Err(format!("Invalid character: {}", ch)),
    };

    Ok((Token::Char(c), source))
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_int_number() {
        let sources = vec![
            ("", '0', 0, ""),
            ("23", '1', 123, ""),
            ("21 foo", '3', 321, " foo"),
            ("123", '-', -123, ""),
        ];

        for (source, first, expected, rest) in sources {
            let (token, rest_iter) = super::lex_number(source.chars().peekable(), first).unwrap();
            assert_eq!(token, super::Token::Integer(expected));
            assert_eq!(rest_iter.collect::<String>(), rest);
        }
    }

    #[test]
    fn parse_float_number() {
        let sources = vec![
            (".0", '0', 0.0, ""),
            ("23.", '1', 123.0, ""),
            ("21.foo", '3', 321.0, "foo"),
            ("123.456", '-', -123.456, ""),
            ("123bar", '.', 0.123, "bar"),
        ];

        for (source, first, expected, rest) in sources {
            let (token, rest_iter) = super::lex_number(source.chars().peekable(), first).unwrap();
            assert_eq!(token, super::Token::Float(expected));
            assert_eq!(rest_iter.collect::<String>(), rest);
        }
    }

    #[test]
    fn sandbox() {
        let source = "(fóo bar baz)";
        let mut lexer = super::Lexer::new(source);

        assert_eq!(lexer.current, '(');
        assert_eq!(lexer.currentn(3), "(fó");
        assert_eq!(lexer.peek(), Some('f'));
        assert_eq!(lexer.position(), (1, 1));
        lexer.advance();
        assert_eq!(lexer.current, 'f');
        assert_eq!(lexer.position(), (1, 2));
        lexer.advancen(3);
        assert_eq!(lexer.current, ' ');
        lexer.advance();
        assert_eq!(lexer.currentn(3), "bar");
        assert_eq!(lexer.current, 'b');
        assert_eq!(lexer.peek(), Some('a'));
    }
}