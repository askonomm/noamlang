mod lexer;
mod parser;

use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() {
    let input = r#"
        func print(arg: Unknown, arg2: String) {
            // do something here
        }

        print(String[Hello])
        print(Integer[123123])

        if function(String[Hello]) = String[Hello] {
            print(String[Hello])
        }
    "#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("Tokens:");
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("\nParsing AST:");
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => println!("Parsing error: {}", e),
    }
}
