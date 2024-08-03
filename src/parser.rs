use std::{iter::Peekable, vec::IntoIter};

use crate::lexer::{symbol::Symbol, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Form {
    Call(Vec<Form>),
    Symbol(Symbol),
    Float(f64),
    Integer(i64),
    String(String),
    Char(char),
    Keyword(String),
    List(Vec<Form>),
    Map(Vec<(Form, Form)>),
}

pub fn parse(mut tokens: Peekable<IntoIter<Token>>) -> Result<(Form, Peekable<IntoIter<Token>>), String> {
    loop {
        match tokens.next() {
            None => return Err(format!("Unexpected end of input")),
            Some(token) => match token {
                Token::Open('(') => {
                    let form = parse_call(tokens)?;
                    return Ok(form);
                },
                Token::Open('[') => {
                    let form = parse_list(tokens)?;
                    return Ok(form);
                },
                Token::Open('{') => {
                    let form = parse_map(tokens)?;
                    return Ok(form);
                },
                Token::Integer(i) => return Ok((Form::Integer(i), tokens)),
                Token::Float(f) => return Ok((Form::Float(f), tokens)),
                Token::String(s) => return Ok((Form::String(s), tokens)),
                Token::Char(c) => return Ok((Form::Char(c), tokens)),
                Token::Symbol(s) => return Ok((Form::Symbol(s), tokens)),
                Token::Keyword(k) => return Ok((Form::Keyword(k), tokens)),
                _ => return Err(format!("Unexpected token: {:?}", token)),
            },
        }
    }
}

fn parse_call(mut tokens: Peekable<IntoIter<Token>>) -> Result<(Form, Peekable<IntoIter<Token>>), String> {
    let mut forms = Vec::new();

    loop {
        match tokens.peek() {
            None => return Err(format!("Unexpected end of input")),
            Some(token) => match token {
                Token::Close(')') => {
                    tokens.next();
                    return Ok((Form::Call(forms), tokens))
                }, // TODO: Ban empty calls
                Token::Close(c) => return Err(format!("Unexpected token: `{}`, expected `)`", c)),
                _ => {
                    let (form, tks) = parse(tokens)?;
                    forms.push(form);
                    tokens = tks;
                },
            },
        }
    }
}

fn parse_list(mut tokens: Peekable<IntoIter<Token>>) -> Result<(Form, Peekable<IntoIter<Token>>), String> {
    let mut forms = Vec::new();

    loop {
        match tokens.peek() {
            None => return Err(format!("Unexpected end of input")),
            Some(token) => match token {
                Token::Close(']') => {
                    tokens.next();
                    return Ok((Form::List(forms), tokens));
                },
                Token::Close(c) => return Err(format!("Unexpected token: `{}`, expected `]`", c)),
                _ => {
                    let (form, tks) = parse(tokens)?;
                    forms.push(form);
                    tokens = tks;
                },
            },
        }
    }
}

fn parse_map(mut tokens: Peekable<IntoIter<Token>>) -> Result<(Form, Peekable<IntoIter<Token>>), String> {
    let mut forms = Vec::new();

    loop {
        match tokens.peek() {
            None => return Err(format!("Unexpected end of input")),
            Some(token) => match token {
                Token::Close('}') => {
                    tokens.next();
                    return Ok((Form::Map(forms), tokens));
                },
                Token::Close(c) => return Err(format!("Unexpected token: `{}`, expected `}}`", c)),
                _ => {
                    let (key, tks) = parse(tokens)?;
                    let (value, tks) = parse(tks)?;
                    forms.push((key, value));
                    tokens = tks;
                },
            },
        }
    }
}