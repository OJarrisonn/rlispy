use std::{iter::Peekable, str::Chars};

const KEYWORD_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789-";
const SYMBOL_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-+*/|<>=!?@#$%";
const ESCAPABLE_CHARS: &'static str = "\"ntr\\";

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Symbol(Symbol),
    Keyword(String),
    Open(char),
    Close(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub head: String,
    pub tail: Vec<String>,
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    loop {
        match chars.next() {
            None => break,
            Some(c) => match c {
                // Skip whitespace
                c if c.is_whitespace() => continue,
                // Parse a scope start
                '(' | '{' | '[' => tokens.push(Token::Open(c)),
                // Parse a scope end
                ')' | '}' | ']' => tokens.push(Token::Close(c)),
                // Parse a string
                '"' => {
                    let (string, rest) = lex_string(chars)?;
                    tokens.push(Token::String(string));
                    chars = rest;
                },
                // Parse a keyword
                ':' => {
                    let (kw, rest) = lex_keyword(chars);
                    tokens.push(Token::Keyword(kw));
                    chars = rest;
                },
                // Parse a character
                '\\' => {
                    let (ch, rest) = lex_char(chars)?;
                    tokens.push(ch);
                    chars = rest;
                },
                // Parse a comment
                ';' => {
                    while let Some(c) = chars.next() {
                        if c == '\n' {
                            break;
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
            }
        }
    }    

    Ok(tokens)
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

fn lex_keyword(mut source: Peekable<Chars>) -> (String, Peekable<Chars>) {
    let mut keyword = String::new();

    loop {
        match source.peek() {
            Some(&c) if KEYWORD_CHARS.contains(c) => {
                keyword.push(c);
                source.next();
            },
            _ => break,
        }
    }

    (keyword, source)
}

fn lex_string(mut source: Peekable<Chars>) -> Result<(String, Peekable<Chars>), String> {
    let mut string = String::new();

    loop {
        match source.next() {
            None => return Err("Unexpected end of input".to_string()),
            Some('\\') => match source.next() {
                None => return Err("Unexpected end of input".to_string()),
                Some('"') => string.push('"'),
                Some(c) if ESCAPABLE_CHARS.contains(c) => string.push(c),
                Some(c) => return Err(format!("Unexpected escape character: {}", c)),
            },
            Some('"') => break,
            Some(c) => string.push(c),
        }
    }

    Ok((string, source))
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
}