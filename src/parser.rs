use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expression {
    StringLiteral(String),
    IntegerLiteral(i64),
    Identifier(String),
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    TypedValue {
        type_name: String,
        value: Box<Expression>,
    },
    BinaryOperation {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
    },
    IfStatement {
        condition: Expression,
        body: Vec<Statement>,
    },
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current_position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(statement) => program.statements.push(statement),
                Err(e) => return Err(e),
            }
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek_token();
        
        match token {
            Token::Func => self.parse_function_declaration(),
            Token::If => self.parse_if_statement(),
            Token::Comment(comment) => {
                self.advance();
                Ok(Statement::Comment(comment))
            },
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, String> {
        // Consume 'func' token
        self.advance();
        
        // Get function name
        let name = match self.consume_token() {
            Token::Identifier(name) => name,
            _ => return Err("Expected function name after 'func' keyword".to_string()),
        };
        
        // Consume opening parenthesis
        if !self.match_token(&Token::LeftParen) {
            return Err("Expected '(' after function name".to_string());
        }
        
        // Parse parameters
        let parameters = self.parse_parameters()?;
        
        // Consume closing parenthesis
        if !self.match_token(&Token::RightParen) {
            return Err("Expected ')' after parameters".to_string());
        }
        
        // Consume opening brace
        if !self.match_token(&Token::LeftBrace) {
            return Err("Expected '{' after function declaration".to_string());
        }
        
        // Parse function body
        let mut body = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        
        // Consume closing brace
        if !self.match_token(&Token::RightBrace) {
            return Err("Expected '}' after function body".to_string());
        }
        
        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            body,
        })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
        let mut parameters = Vec::new();
        
        // If next token is ')', we have no parameters
        if self.check(&Token::RightParen) {
            return Ok(parameters);
        }
        
        loop {
            // Get parameter name
            let name = match self.consume_token() {
                Token::Identifier(name) => name,
                _ => return Err("Expected parameter name".to_string()),
            };
            
            // Consume colon
            if !self.match_token(&Token::Colon) {
                return Err("Expected ':' after parameter name".to_string());
            }
            
            // Get parameter type
            let type_name = match self.consume_token() {
                Token::TypeString => "String".to_string(),
                Token::TypeInteger => "Integer".to_string(),
                Token::TypeUnknown => "Unknown".to_string(),
                Token::Identifier(type_name) => type_name,
                _ => return Err("Expected type name after ':'".to_string()),
            };
            
            parameters.push(Parameter { name, type_name });
            
            // If next token is ')', we're done
            if self.check(&Token::RightParen) {
                break;
            }
            
            // Otherwise, expect a comma
            if !self.check(&Token::Comma) {
                return Err("Expected ',' between parameters".to_string());
            }
            
            // Consume comma
            self.advance();
        }
        
        Ok(parameters)
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        // Consume 'if' token
        self.advance();
        
        // Parse condition
        let condition = self.parse_expression()?;
        
        // Consume opening brace
        if !self.match_token(&Token::LeftBrace) {
            return Err("Expected '{' after if condition".to_string());
        }
        
        // Parse if body
        let mut body = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        
        // Consume closing brace
        if !self.match_token(&Token::RightBrace) {
            return Err("Expected '}' after if body".to_string());
        }
        
        Ok(Statement::IfStatement { condition, body })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        let expr = self.parse_primary_expression()?;
        
        // Check for binary operations like 'is' and 'is not'
        if self.check(&Token::Equals) {
            self.advance(); // Consume the 'is' token
            let right = self.parse_primary_expression()?;
            return Ok(Expression::BinaryOperation {
                left: Box::new(expr),
                operator: "is".to_string(),
                right: Box::new(right),
            });
        } else if self.check(&Token::NotEquals) {
            self.advance(); // Consume the 'is not' token
            let right = self.parse_primary_expression()?;
            return Ok(Expression::BinaryOperation {
                left: Box::new(expr),
                operator: "is not".to_string(),
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        match self.peek_token() {
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expression::StringLiteral(s))
            },
            Token::IntegerLiteral(i) => {
                self.advance();
                Ok(Expression::IntegerLiteral(i))
            },
            Token::Identifier(name) => {
                self.advance();
                // Check if it's a function call
                if self.check(&Token::LeftParen) {
                    self.advance();  // Consume '('
                    let arguments = self.parse_arguments()?;
                    if !self.match_token(&Token::RightParen) {
                        return Err("Expected ')' after function arguments".to_string());
                    }
                    Ok(Expression::FunctionCall { name, arguments })
                } else {
                    Ok(Expression::Identifier(name))
                }
            },
            Token::TypeString | Token::TypeInteger => {
                let type_name = match self.consume_token() {
                    Token::TypeString => "String".to_string(),
                    Token::TypeInteger => "Integer".to_string(),
                    _ => unreachable!(),
                };
                
                // We expect a left bracket after the type name
                if !self.match_token(&Token::LeftBracket) {
                    return Err("Expected '[' after type name".to_string());
                }
                
                // Parse the value inside the brackets
                let value = self.parse_expression()?;
                
                // We expect a right bracket to close
                if !self.match_token(&Token::RightBracket) {
                    return Err("Expected ']' after type value".to_string());
                }
                
                Ok(Expression::TypedValue {
                    type_name,
                    value: Box::new(value),
                })
            },
            Token::TypeTrue => {
                self.advance();
                Ok(Expression::Identifier("True".to_string()))
            },
            Token::TypeFalse => {
                self.advance();
                Ok(Expression::Identifier("False".to_string()))
            },
            _ => Err(format!("Unexpected token: {:?}", self.peek_token())),
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut arguments = Vec::new();
        
        // If next token is ')', we have no arguments
        if self.check(&Token::RightParen) {
            return Ok(arguments);
        }
        
        loop {
            let argument = self.parse_expression()?;
            arguments.push(argument);
            
            // If next token is ')', we're done
            if self.check(&Token::RightParen) {
                break;
            }
            
            // Otherwise, expect a comma
            if !self.check(&Token::Comma) {
                return Err("Expected ',' between arguments".to_string());
            }
            
            // Consume comma
            self.advance();
        }
        
        Ok(arguments)
    }

    fn peek_token(&self) -> Token {
        if self.current_position >= self.tokens.len() {
            Token::EOF
        } else {
            self.tokens[self.current_position].clone()
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.peek_token();
        self.current_position += 1;
        token
    }

    fn consume_token(&mut self) -> Token {
        self.advance()
    }

    fn is_at_end(&self) -> bool {
        self.peek_token() == Token::EOF
    }

    fn check(&self, token_type: &Token) -> bool {
        &self.peek_token() == token_type
    }

    fn match_token(&mut self, token_type: &Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
}