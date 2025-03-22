use crate::parser::{Expression, Program, Statement, Parameter};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Boolean(bool),
    Null,
    Function {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Function { name, .. } => write!(f, "<function {}>", name),
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            values: HashMap::new(),
            parent: None,
        }; 
        
        // Add print function
        env.define("print".to_string(), Value::Function {
            name: "print".to_string(),
            parameters: vec![],
            body: vec![],
        });

        env
    }
    
    pub fn extend(parent: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(parent) = &self.parent {
                    parent.get(name)
                } else {
                    None
                }
            }
        }
    }
    
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'", name))
        }
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }
    
    pub fn interpret(&mut self, program: Program) -> Result<(), String> {
        for statement in program.statements {
            self.execute_statement(&statement)?;
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, statement: &Statement) -> Result<Value, String> {
        match statement {
            Statement::Expression(expr) => self.evaluate_expression(expr),
            
            Statement::FunctionDeclaration { name, parameters, body } => {
                let function = Value::Function {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                };
                self.environment.define(name.clone(), function);
                Ok(Value::Null)
            },
            
            Statement::IfStatement { condition, body } => {
                let condition_value = self.evaluate_expression(condition)?;
                
                if self.is_truthy(&condition_value) {
                    let mut result = Value::Null;
                    for stmt in body {
                        result = self.execute_statement(stmt)?;
                    }
                    Ok(result)
                } else {
                    Ok(Value::Null)
                }
            },
            
            Statement::Comment(_) => Ok(Value::Null),
        }
    }
    
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(i) => *i != 0,
            Value::String(s) => !s.is_empty(),
            Value::Function { .. } => true,
        }
    }
    
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
            
            Expression::IntegerLiteral(i) => Ok(Value::Integer(*i)),
            
            Expression::Identifier(name) => {
                match self.environment.get(name) {
                    Some(value) => Ok(value),
                    None => Err(format!("Undefined variable '{}'", name)),
                }
            },
            
            Expression::FunctionCall { name, arguments } => {
                let function = self.environment.get(name)
                    .ok_or_else(|| format!("Undefined function '{}'", name))?;
                
                match function {
                    Value::Function { name, parameters, body } => {
                        // Special case for built-in print function
                        if name == "print" {
                            let mut arg_values = Vec::new();
                            for arg in arguments {
                                let value = self.evaluate_expression(arg)?;
                                arg_values.push(value);
                            }
                            
                            for value in arg_values {
                                println!("{}", value);
                            }
                            
                            return Ok(Value::Null);
                        }

                        // User-defined function
                        if arguments.len() != parameters.len() {
                            return Err(format!(
                                "Expected {} arguments but got {}",
                                parameters.len(),
                                arguments.len()
                            ));
                        }
                        
                        let mut arg_values = Vec::new();
                        for arg in arguments {
                            let value = self.evaluate_expression(arg)?;
                            arg_values.push(value);
                        }
                        
                        let mut env = Environment::extend(self.environment.clone());
                        
                        for (param, value) in parameters.iter().zip(arg_values) {
                            env.define(param.name.clone(), value);
                        }
                        
                        let previous_env = std::mem::replace(&mut self.environment, env);
                        
                        let mut result = Value::Null;
                        for stmt in &body {
                            result = self.execute_statement(stmt)?;
                        }
                        
                        self.environment = previous_env;
                        Ok(result)
                    },
                    _ => Err(format!("'{}' is not a function", name)),
                }
            },
            
            Expression::TypedValue { type_name, value } => {
                // Special case for String[Hello] and similar constructs
                if let Expression::Identifier(ident) = &**value {
                    match type_name.as_str() {
                        "String" => return Ok(Value::String(ident.clone())),
                        "Integer" => {
                            if let Ok(i) = ident.parse::<i64>() {
                                return Ok(Value::Integer(i));
                            } else {
                                return Err(format!("Cannot convert '{}' to Integer", ident));
                            }
                        },
                        _ => {}
                    }
                }
                
                let inner_value = self.evaluate_expression(value)?;
                
                // Type checking
                match (type_name.as_str(), &inner_value) {
                    ("String", Value::String(_)) => Ok(inner_value),
                    ("Integer", Value::Integer(_)) => Ok(inner_value),
                    _ => Err(format!(
                        "Type mismatch: expected {}, got {:?}", 
                        type_name, 
                        inner_value
                    )),
                }
            },
            
            Expression::BinaryOperation { left, operator, right } => {
                let left_value = self.evaluate_expression(left)?;
                let right_value = self.evaluate_expression(right)?;
                
                match operator.as_str() {
                    "is" => Ok(Value::Boolean(self.values_equal(&left_value, &right_value))),
                    "is not" => Ok(Value::Boolean(!self.values_equal(&left_value, &right_value))),
                    _ => Err(format!("Unknown operator: {}", operator)),
                }
            },
        }
    }
    
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}