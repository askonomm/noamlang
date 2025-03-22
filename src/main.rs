mod lexer;
mod parser;
mod interpreter;
mod typechecker;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::typechecker::TypeChecker;

fn main() {
    let input = r#"
        // Define a custom function to greet someone
        func greet(name: String) {
            print(String[Hello], name)
        }

        // Call our custom function
        greet(String[World])

        // Test conditional execution with "is" operator
        if String[Hello] is String[Hello] {
            print(String[True])
        }

        // Test conditional execution with "is not" operator
        if String[Hello] is not String[Goodbye] {
            print(String[Different])
        }

        // Use the built-in function
        print(String[Answer], Integer[42])
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
        Ok(ast) => {
            println!("{:#?}", ast);
            
            println!("\nType checking program:");
            let mut type_checker = TypeChecker::new();
            match type_checker.check_program(&ast) {
                Ok(_) => {
                    println!("Type checking successful");
                    println!("\nInterpreting program:");
                    let mut interpreter = Interpreter::new();
                    match interpreter.interpret(ast) {
                        Ok(_) => println!("Program executed successfully"),
                        Err(e) => println!("Runtime error: {}", e),
                    }
                },
                Err(e) => println!("Type error: {}", e),
            }
        },
        Err(e) => println!("Parsing error: {}", e),
    }
}
