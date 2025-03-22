use crate::parser::{Expression, Program, Statement};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    String,
    Integer,
    Boolean,
    Void,
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    Unknown,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "String"),
            Type::Integer => write!(f, "Integer"),
            Type::Boolean => write!(f, "Boolean"),
            Type::Void => write!(f, "Void"),
            Type::Function { parameters, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            },
            Type::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone)]
pub struct TypeEnvironment {
    types: HashMap<String, Type>,
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        let mut env = TypeEnvironment {
            types: HashMap::new(),
            parent: None,
        };
        
        // Add built-in functions
        env.define("print".to_string(), Type::Function {
            parameters: vec![Type::Unknown],  // print can take any type
            return_type: Box::new(Type::Void),
        });
        
        // Add the dummy 'function' function
        env.define("function".to_string(), Type::Function {
            parameters: vec![Type::Unknown],
            return_type: Box::new(Type::Unknown),
        });
        
        env
    }
    
    pub fn extend(parent: TypeEnvironment) -> Self {
        TypeEnvironment {
            types: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn define(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }
    
    pub fn get(&self, name: &str) -> Option<Type> {
        match self.types.get(name) {
            Some(ty) => Some(ty.clone()),
            None => {
                if let Some(parent) = &self.parent {
                    parent.get(name)
                } else {
                    None
                }
            }
        }
    }
}

pub struct TypeChecker {
    environment: TypeEnvironment,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            environment: TypeEnvironment::new(),
        }
    }
    
    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        for statement in &program.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }
    
    fn check_statement(&mut self, statement: &Statement) -> Result<Type, String> {
        match statement {
            Statement::Expression(expr) => self.check_expression(expr),
            
            Statement::FunctionDeclaration { name, parameters, body } => {
                // Collect parameter types
                let mut param_types = Vec::new();
                
                for param in parameters {
                    let param_type = self.parse_type_name(&param.type_name);
                    param_types.push(param_type);
                }
                
                // Create function type
                let func_type = Type::Function {
                    parameters: param_types.clone(),
                    return_type: Box::new(Type::Void),  // Default return type
                };
                
                // Define function in environment before checking body
                self.environment.define(name.clone(), func_type);
                
                // Create a new environment for function body
                let current_env = self.environment.clone();
                let prev_env = std::mem::replace(&mut self.environment, 
                                                TypeEnvironment::extend(current_env));
                
                // Add parameters to the new environment
                for (param, param_type) in parameters.iter().zip(param_types) {
                    self.environment.define(param.name.clone(), param_type);
                }
                
                // Check function body
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                
                // Restore previous environment
                self.environment = prev_env;
                
                Ok(Type::Void)
            },
            
            Statement::IfStatement { condition, body } => {
                // Check condition
                let cond_type = self.check_expression(condition)?;
                
                // In a more strict language, we'd require condition to be boolean
                if cond_type != Type::Boolean && cond_type != Type::Unknown {
                    return Err(format!(
                        "If condition must be a boolean, got {}", cond_type
                    ));
                }
                
                // Check body
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                
                Ok(Type::Void)
            },
            
            Statement::Comment(_) => Ok(Type::Void),
        }
    }
    
    fn check_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::StringLiteral(_) => Ok(Type::String),
            
            Expression::IntegerLiteral(_) => Ok(Type::Integer),
            
            Expression::Identifier(name) => {
                match self.environment.get(name) {
                    Some(ty) => Ok(ty),
                    None => Err(format!("Undefined variable '{}'", name)),
                }
            },
            
            Expression::FunctionCall { name, arguments } => {
                // Check if function exists
                let func_type = match self.environment.get(name) {
                    Some(ty) => ty,
                    None => return Err(format!("Undefined function '{}'", name)),
                };
                
                // Special case for built-in 'print' function
                if name == "print" {
                    // Check all arguments
                    for arg in arguments {
                        self.check_expression(arg)?;
                    }
                    return Ok(Type::Void);
                }
                
                // Special case for 'function' function
                if name == "function" {
                    if let Some(arg) = arguments.first() {
                        return self.check_expression(arg);
                    }
                    return Ok(Type::Unknown);
                }
                
                // For normal functions, check parameter types
                match func_type {
                    Type::Function { parameters, return_type } => {
                        // Check argument count
                        if arguments.len() != parameters.len() {
                            return Err(format!(
                                "Function '{}' expects {} arguments, got {}",
                                name, parameters.len(), arguments.len()
                            ));
                        }
                        
                        // Check each argument type
                        for (arg, param_type) in arguments.iter().zip(parameters.iter()) {
                            let arg_type = self.check_expression(arg)?;
                            if !self.types_compatible(&arg_type, param_type) {
                                return Err(format!(
                                    "Type mismatch: expected {}, got {}",
                                    param_type, arg_type
                                ));
                            }
                        }
                        
                        Ok(*return_type)
                    },
                    _ => Err(format!("'{}' is not a function", name)),
                }
            },
            
            Expression::TypedValue { type_name, value } => {
                let expected_type = self.parse_type_name(type_name);
                
                // Special case for String[Hello] and similar constructs
                if let Expression::Identifier(_) = &**value {
                    return Ok(expected_type);
                }
                
                // For other expressions, check their type
                let value_type = self.check_expression(value)?;
                
                if !self.types_compatible(&value_type, &expected_type) {
                    return Err(format!(
                        "Type mismatch: expected {}, got {}",
                        expected_type, value_type
                    ));
                }
                
                Ok(expected_type)
            },
            
            Expression::BinaryOperation { left, operator, right } => {
                let _left_type = self.check_expression(left)?;
                let _right_type = self.check_expression(right)?;
                
                match operator.as_str() {
                    "is" | "is not" => {
                        // Any type can be compared for equality/inequality
                        Ok(Type::Boolean)
                    },
                    _ => Err(format!("Unknown operator: {}", operator)),
                }
            },
        }
    }
    
    fn parse_type_name(&self, name: &str) -> Type {
        match name {
            "String" => Type::String,
            "Integer" => Type::Integer,
            "Boolean" => Type::Boolean,
            "Unknown" => Type::Unknown,
            _ => Type::Unknown,
        }
    }
    
    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        // If either type is Unknown, we allow it (gradual typing)
        if *expected == Type::Unknown || *actual == Type::Unknown {
            return true;
        }
        
        // Otherwise, types must match exactly
        actual == expected
    }
}