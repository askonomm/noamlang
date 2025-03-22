use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Identifiers and literals
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),

    // Types
    TypeString,
    TypeInteger,
    TypeUnknown,
    TypeTrue,
    TypeFalse,

    // Symbols
    LeftBracket,      // [
    RightBracket,     // ]
    LeftBrace,        // {
    RightBrace,       // }
    LeftParen,        // (
    RightParen,       // )
    Equals,           // is
    NotEquals,        // is not
    Colon,            // :
    Comma,            // ,

    // Keywords
    If,
    Func,

    // Comments
    Comment(String),

    // End of file
    EOF,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current_char = chars.next();

        Lexer {
            input: chars,
            current_char,
        }
    }

    fn advance(&mut self) {
        self.current_char = self.input.next();
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }
    
    fn read_comment(&mut self) -> String {
        let mut comment = String::new();
        
        // Skip the initial '//'
        self.advance();
        self.advance();
        
        // Read until the end of line or end of file
        while let Some(c) = self.current_char {
            if c == '\n' || c == '\r' {
                break;
            }

            comment.push(c);
            self.advance();
        }
        
        comment.trim().to_string()
    }

    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();

        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }

        identifier
    }

    fn read_number(&mut self) -> i64 {
        let mut number = String::new();

        while let Some(c) = self.current_char {
            if c.is_digit(10) {
                number.push(c);
                self.advance();
            } else {
                break;
            }
        }

        number.parse::<i64>().unwrap_or(0)
    }

    fn read_string_literal(&mut self) -> String {
        let mut string = String::new();

        // Skip the opening parenthesis
        self.advance();

        while let Some(c) = self.current_char {
            if c == ')' {
                // End of string literal
                self.advance();
                break;
            } else {
                string.push(c);
                self.advance();
            }
        }

        string
    }

    fn read_type_value(&mut self) -> Token {
        let identifier = self.read_identifier();

        // Check if the next character is an opening parenthesis
        if let Some('(') = self.current_char {
            match identifier.as_str() {
                "String" => {
                    let string_value = self.read_string_literal();
                    return Token::StringLiteral(string_value);
                },
                "Integer" => {
                    // For Integer, we need to parse the content as a number
                    let string_value = self.read_string_literal();
                    if let Ok(int_value) = string_value.parse::<i64>() {
                        return Token::IntegerLiteral(int_value);
                    } else {
                        // If parsing fails, return 0 or handle error
                        return Token::IntegerLiteral(0);
                    }
                },
                _ => {}
            }
        }

        // If not followed by a parenthesis or not a known type
        match identifier.as_str() {
            "String" => Token::TypeString,
            "Integer" => Token::TypeInteger,
            "True" => Token::TypeTrue,
            "False" => Token::TypeFalse,
            "Unknown" => Token::TypeUnknown,
            "if" => Token::If,
            "func" => Token::Func,
            _ => Token::Identifier(identifier),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        // Check for comment first, before the match statement to avoid borrow issues
        if self.current_char == Some('/') && self.peek() == Some(&'/') {
            let comment = self.read_comment();
            return Token::Comment(comment);
        }

        match self.current_char {
            None => Token::EOF,

            Some('[') => {
                self.advance();
                Token::LeftBracket
            },

            Some(']') => {
                self.advance();
                Token::RightBracket
            },

            Some('{') => {
                self.advance();
                Token::LeftBrace
            },

            Some('}') => {
                self.advance();
                Token::RightBrace
            },

            Some('(') => {
                self.advance();
                Token::LeftParen
            },

            Some(')') => {
                self.advance();
                Token::RightParen
            },

            Some('i') => {
                self.advance(); // consume 'i'
                
                // Check if it's "is"
                if self.current_char == Some('s') {
                    self.advance(); // consume 's'
                    
                    // Check if it's "is not"
                    if self.current_char == Some(' ') {
                        self.advance(); // consume space
                        
                        // Try to match "not"
                        if self.current_char == Some('n') {
                            self.advance(); // consume 'n'
                            
                            if self.current_char == Some('o') {
                                self.advance(); // consume 'o'
                                
                                if self.current_char == Some('t') {
                                    self.advance(); // consume 't'
                                    return Token::NotEquals;
                                }
                            }
                        }
                    }
                    
                    return Token::Equals;
                }
                
                // If it's not "is" or "is not", treat 'i' as an identifier
                let mut identifier = String::from("i");
                while let Some(c) = self.current_char {
                    if c.is_alphanumeric() || c == '_' {
                        identifier.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                match identifier.as_str() {
                    "if" => Token::If,
                    _ => Token::Identifier(identifier),
                }
            },
            
            Some(':') => {
                self.advance();
                Token::Colon
            },
            
            Some(',') => {
                self.advance();
                Token::Comma
            },

            Some(c) if c.is_alphabetic() => {
                self.read_type_value()
            },

            Some(c) if c.is_digit(10) => {
                let number = self.read_number();
                Token::IntegerLiteral(number)
            },

            Some(_) => {
                self.advance();
                self.next_token()
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();

            if token == Token::EOF {
                tokens.push(token);
                break;
            }

            // Include comment tokens as they will be handled by the parser
            tokens.push(token);
        }

        tokens
    }
}