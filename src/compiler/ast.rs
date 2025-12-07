#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub story: Story,
    pub concepts: Vec<Concept>,
    pub situations: Vec<Situation>,
}

// Story: Main entry point
#[derive(Debug, Clone, PartialEq)]
pub struct Story {
    pub body: Vec<Statement>,
}

// Concept: Class definition
#[derive(Debug, Clone, PartialEq)]
pub struct Concept {
    pub name: String,
    pub fields: Vec<String>,
    pub methods: Vec<Method>,
    pub when_observers: std::collections::HashMap<String, Vec<Statement>>,
}

// Situation: Context that modifies behavior
#[derive(Debug, Clone, PartialEq)]
pub struct Situation {
    pub name: String,
    pub adjustments: Vec<Adjustment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Adjustment {
    pub concept_name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Use {
        module_path: String, // "models.User"
        line: usize,
    },

    // Variable assignment: Name is "Johgn"
    Assignment {
        target: String,
        value: Expression,
        line: usize,
    },

    // Create Concept called Instance
    Create {
        concept_name: String,
        instance_name: String,
        initial_fields: Vec<(String, Expression)>, // Field name -> initial value
        line: usize,
    },

    // Set statement: Set Score to 100
    Set {
        target: Expression,
        value: Expression,
        line: usize,
    },

    // Print statement: Print "Hello"
    Print {
        value: Expression,
        line: usize,
    },

    // Switch on/off situations
    SwitchOn {
        situation: String,
        line: usize,
    },
    SwitchOff {
        situation: String,
        line: usize,
    },

    // If/Else: If Score > 100: ... Else: ...
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
        line: usize,
    },

    // When/Is/Otherwise: When Score: Is 100: ... Is 90: ... Otherwise: ...
    When {
        value: Expression,
        cases: Vec<(Expression, Vec<Statement>)>,
        otherwise: Option<Vec<Statement>>,
        line: usize,
    },

    // Try/Catch/Always: Try: ... Catch error: ... Always: ...
    TryCatch {
        try_body: Vec<Statement>,
        catch_var: Option<String>, // The error variable name (e.g., "error")
        catch_body: Option<Vec<Statement>>,
        always_body: Option<Vec<Statement>>,
        line: usize,
    },

    // Repeat N times: Repeat 5 times: ... or Repeat 5 times With I: ...
    RepeatTimes {
        count: Expression,
        variable: Option<String>, // Optional loop variable (1-indexed)
        body: Vec<Statement>,
        line: usize,
    },

    // Repeat while: Repeat while Score < 100: ...
    RepeatWhile {
        condition: Expression,
        body: Vec<Statement>,
        line: usize,
    },

    // For each: For each Item in List: ...
    ForEach {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
        line: usize,
    },

    // Return: Return Result
    Return {
        value: Option<Expression>,
        line: usize,
    },

    // Break/Continue
    Break {
        line: usize,
    },
    Continue {
        line: usize,
    },

    // Expression statement (method calls, etc.)
    Expression {
        expr: Expression,
        line: usize,
    },
}

// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Number(String),
    String(String),
    Boolean(bool),

    // List: [1, 2, 3]
    List(Vec<Expression>),

    // Map: { name: "John", age: 34 }
    Map(Vec<(String, Expression)>),

    // Identifier: Score, Name
    Identifier(String),

    // Binary operations: A + B, Score > 100
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    // Unary operations: not Active
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },

    // Index access: Numbers[1], User["name"]
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    // Member access: User.Name
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },

    // Method call: User.AddPoints with 10
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<(String, Expression)>,
    },

    // Function call: Print("Hello")
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },

    // Do in background: ... - Returns TaskHandle
    DoInBackground {
        body: Vec<Statement>,
    },

    // Proceed() - Call next adjustment layer in stack
    Proceed {
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %

    // Comparison
    Equal,     // =
    NotEqual,  // !=
    Greater,   // >
    Less,      // <
    GreaterEq, // >=
    LessEq,    // <=

    // Logical
    And, // and
    Or,  // or
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,   // not
    Minus, // -
}

// Helper constructors for common patterns
impl Expression {
    pub fn number(value: &str) -> Self {
        Expression::Number(value.to_string())
    }

    pub fn string(value: &str) -> Self {
        Expression::String(value.to_string())
    }

    pub fn boolean(value: bool) -> Self {
        Expression::Boolean(value)
    }

    pub fn identifier(name: &str) -> Self {
        Expression::Identifier(name.to_string())
    }

    pub fn binary_op(left: Expression, op: BinaryOperator, right: Expression) -> Self {
        Expression::BinaryOp {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        }
    }
}

impl Statement {
    pub fn assignment(target: &str, value: Expression) -> Self {
        Statement::Assignment {
            target: target.to_string(),
            value,
            line: 0,
        }
    }

    pub fn print(value: Expression) -> Self {
        Statement::Print { value, line: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_expression() {
        let expr = Expression::binary_op(
            Expression::number("0.1"),
            BinaryOperator::Add,
            Expression::number("0.2"),
        );

        match expr {
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                assert_eq!(*left, Expression::Number("0.1".to_string()));
                assert_eq!(operator, BinaryOperator::Add);
                assert_eq!(*right, Expression::Number("0.2".to_string()));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_create_assignment() {
        let stmt = Statement::assignment("Name", Expression::string("Temka"));

        match stmt {
            Statement::Assignment {
                target,
                value,
                line: 0,
            } => {
                assert_eq!(target, "Name");
                assert_eq!(value, Expression::String("Temka".to_string()));
            }
            _ => panic!("Expected Assignment"),
        }
    }

    #[test]
    fn test_create_if_statement() {
        let condition = Expression::binary_op(
            Expression::identifier("Score"),
            BinaryOperator::Greater,
            Expression::number("100"),
        );

        let then_body = vec![Statement::print(Expression::string("High score!"))];

        let stmt = Statement::If {
            condition,
            then_body,
            else_body: None,
            line: 0,
        };

        match stmt {
            Statement::If {
                condition,
                then_body,
                else_body,
                line: _,
            } => {
                assert!(matches!(condition, Expression::BinaryOp { .. }));
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_none());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_create_list() {
        let list = Expression::List(vec![
            Expression::number("10"),
            Expression::number("20"),
            Expression::number("30"),
        ]);

        match list {
            Expression::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Expression::Number("10".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_create_map() {
        let map = Expression::Map(vec![
            ("name".to_string(), Expression::string("Temka")),
            ("age".to_string(), Expression::number("34")),
        ]);

        match map {
            Expression::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "name");
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_create_story() {
        let story = Story {
            body: vec![
                Statement::assignment("Name", Expression::string("Temka")),
                Statement::print(Expression::identifier("Name")),
            ],
        };

        assert_eq!(story.body.len(), 2);
    }

    #[test]
    fn test_create_concept() {
        let concept = Concept {
            name: "User".to_string(),
            fields: vec!["Name".to_string(), "Score".to_string()],
            methods: vec![Method {
                name: "AddPoints".to_string(),
                parameters: vec!["Amount".to_string()],
                body: vec![Statement::Set {
                    target: Expression::identifier("Score"),
                    value: Expression::binary_op(
                        Expression::identifier("Score"),
                        BinaryOperator::Add,
                        Expression::identifier("Amount"),
                    ),
                    line: 0,
                }],
            }],
            when_observers: std::collections::HashMap::new(),
        };

        assert_eq!(concept.name, "User");
        assert_eq!(concept.fields.len(), 2);
        assert_eq!(concept.methods.len(), 1);
    }
}
