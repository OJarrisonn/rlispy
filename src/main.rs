use rlispy::{lexer::lex, parser::parse};

fn main() {
    let source = r#"
        (defn add [a b]
            (+ a b))
    "#;

    let tokens = lex(source).unwrap();

    for token in &tokens {
        println!("{:?}", token);
    }

    let (form, _) = parse(tokens.into_iter().peekable()).unwrap();

    println!("{:#?}", form);
}