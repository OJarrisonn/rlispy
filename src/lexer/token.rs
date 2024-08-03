use super::symbol::Symbol;

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