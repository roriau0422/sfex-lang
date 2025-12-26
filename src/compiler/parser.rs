use super::ast::*;
use super::token::{Token, TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: TokenType,
        line: usize,
        column: usize,
    },
    UnexpectedEof {
        line: usize,
        column: usize,
    },
    InvalidSyntax {
        message: String,
        line: usize,
        column: usize,
    },
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    current: Option<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Self {
            tokens: tokens.into_iter().peekable(),
            current: None,
        };
        parser.advance();
        parser
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut concepts = Vec::new();
        let mut situations = Vec::new();
        let mut story_body = Vec::new();

        while !self.is_at_end() {
            self.skip_ignorable();

            match self.peek_type() {
                Some(TokenType::Use) => {
                    /* Use statements are treated as part of the Story execution flow
                    Just parse it as a statement and add it to story_body
                    This means "Use" happens at runtime, which is fine for an interpreter */
                    let stmt = self.parse_statement()?;
                    story_body.push(stmt);
                }
                Some(TokenType::Story) => {
                    let segment = self.parse_story()?;
                    story_body.extend(segment.body);
                }
                Some(TokenType::Concept) => {
                    concepts.push(self.parse_concept()?);
                }
                Some(TokenType::Situation) => {
                    situations.push(self.parse_situation()?);
                }
                Some(TokenType::Dedent) => {
                    self.advance();
                }
                Some(TokenType::Eof) => break,
                None => break,
                _ => {
                    return Err(self.make_invalid_syntax(format!(
                        "Expected Story, Concept, or Situation. Found: {:?}",
                        self.peek_type()
                    )));
                }
            }
        }

        let story = Story { body: story_body };

        Ok(Program {
            story,
            concepts,
            situations,
        })
    }

    fn skip_ignorable(&mut self) {
        loop {
            match self.peek_type() {
                Some(TokenType::Newline) | Some(TokenType::Comment(_)) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    // Skip only comments and spaces, NOT newlines (for comma-separated parsing)
    fn skip_ignorable_no_newline(&mut self) {
        loop {
            match self.peek_type() {
                Some(TokenType::Comment(_)) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    // Skip ignorable tokens including INDENT/DEDENT (for use inside lists/maps)
    fn skip_ignorable_with_indent(&mut self) {
        loop {
            match self.peek_type() {
                Some(TokenType::Newline)
                | Some(TokenType::Comment(_))
                | Some(TokenType::Indent)
                | Some(TokenType::Dedent) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn parse_story(&mut self) -> Result<Story, ParseError> {
        self.expect(TokenType::Story)?;
        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let body = self.parse_block()?;

        Ok(Story { body })
    }

    fn parse_concept(&mut self) -> Result<Concept, ParseError> {
        self.expect(TokenType::Concept)?;
        self.expect(TokenType::Colon)?;
        let name = self.expect_identifier()?;

        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut when_observers = std::collections::HashMap::new();

        loop {
            self.skip_ignorable();

            match self.peek_type() {
                Some(TokenType::Dedent) | Some(TokenType::Eof) => break,
                Some(TokenType::To) => {
                    methods.push(self.parse_method()?);
                }
                Some(TokenType::When) => {
                    // Parse: When [property] changes:
                    self.advance(); // eat "When"
                    let property = self.expect_identifier()?;

                    // Expect "changes" identifier
                    let changes_word = self.expect_identifier()?;
                    if changes_word != "changes" {
                        return Err(self.make_unexpected_token(
                            "changes".to_string(),
                            TokenType::Identifier(changes_word),
                        ));
                    }

                    self.expect(TokenType::Colon)?;
                    self.skip_ignorable();
                    self.expect(TokenType::Indent)?;

                    // Parse the block of statements
                    let mut when_body = Vec::new();
                    loop {
                        self.skip_ignorable();
                        match self.peek_type() {
                            Some(TokenType::Dedent) | Some(TokenType::Eof) => break,
                            _ => when_body.push(self.parse_statement()?),
                        }
                    }

                    self.expect(TokenType::Dedent)?;

                    // Add to when_observers
                    when_observers.insert(property, when_body);
                }
                Some(TokenType::Identifier(_)) => {
                    // Parse first field
                    fields.push(self.expect_identifier()?);

                    // Parse comma-separated fields on same line
                    while self.check(&TokenType::Comma) {
                        self.advance(); // eat comma
                        self.skip_ignorable_no_newline(); // Skip spaces but not newlines

                        // If we hit a newline after comma, stop (next field on new line)
                        if self.check(&TokenType::Newline) {
                            break;
                        }

                        fields.push(self.expect_identifier()?);
                    }

                    self.skip_ignorable();
                }
                _ => break,
            }
        }

        self.expect(TokenType::Dedent)?;

        Ok(Concept {
            name,
            fields,
            methods,
            when_observers,
        })
    }

    fn parse_situation(&mut self) -> Result<Situation, ParseError> {
        self.expect(TokenType::Situation)?;
        self.expect(TokenType::Colon)?;
        let name = self.expect_identifier()?;

        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let mut adjustments = Vec::new();

        loop {
            self.skip_ignorable();

            match self.peek_type() {
                Some(TokenType::Dedent) | Some(TokenType::Eof) => break,
                Some(TokenType::Adjust) => {
                    adjustments.push(self.parse_adjustment()?);
                }
                _ => break,
            }
        }

        self.expect(TokenType::Dedent)?;

        Ok(Situation { name, adjustments })
    }

    fn parse_adjustment(&mut self) -> Result<Adjustment, ParseError> {
        self.expect(TokenType::Adjust)?;
        let concept_name = self.expect_identifier()?;
        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let mut methods = Vec::new();

        loop {
            self.skip_ignorable();

            match self.peek_type() {
                Some(TokenType::Dedent) | Some(TokenType::Eof) => break,
                Some(TokenType::To) => {
                    methods.push(self.parse_method()?);
                }
                _ => break,
            }
        }

        self.expect(TokenType::Dedent)?;

        Ok(Adjustment {
            concept_name,
            methods,
        })
    }

    fn parse_method(&mut self) -> Result<Method, ParseError> {
        self.expect(TokenType::To)?;
        let name = self.expect_identifier()?;

        let mut parameters = Vec::new();

        if self.check(&TokenType::With) {
            self.advance();
            parameters.push(self.expect_identifier()?);

            while self.check(&TokenType::And) {
                self.advance();
                parameters.push(self.expect_identifier()?);
            }
        }

        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let body = self.parse_block()?;

        Ok(Method {
            name,
            parameters,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        loop {
            self.skip_ignorable();

            if self.check(&TokenType::Dedent) || self.is_at_end() {
                break;
            }

            match self.peek_type() {
                Some(TokenType::To) | Some(TokenType::Adjust) | Some(TokenType::Else) => break,
                _ => {}
            }

            // Parse first statement
            statements.push(self.parse_statement()?);

            // Check for comma-separated statements on same line
            while self.check(&TokenType::Comma) {
                self.advance(); // eat comma
                self.skip_ignorable_no_newline(); // Skip spaces but not newlines

                // If we hit a newline after comma, stop (next statement on new line)
                if self.check(&TokenType::Newline) || self.check(&TokenType::Dedent) {
                    break;
                }

                // Parse next statement on same line
                statements.push(self.parse_statement()?);
            }
        }

        if self.check(&TokenType::Dedent) {
            self.advance();
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek_type() {
            Some(TokenType::Use) => {
                self.advance(); // Eat "Use"

                // Parse the module path (e.g. models.User)
                let mut path_parts = Vec::new();
                path_parts.push(self.expect_identifier()?);

                while self.check(&TokenType::Dot) {
                    self.advance(); // Eat dot
                    path_parts.push(self.expect_identifier()?);
                }

                // Construct the file path string: "models/User.sfex"
                let full_path = format!("{}.sfex", path_parts.join("/"));

                self.skip_ignorable();
                let line = self.current_line();
                Ok(Statement::Use {
                    module_path: full_path,
                    line,
                })
            }

            Some(TokenType::Create) => {
                let line = self.current_line();
                self.advance();
                let concept_name = self.expect_identifier()?;
                self.expect(TokenType::Called)?;
                let instance_name = self.expect_identifier()?;
                self.skip_ignorable();

                // Parse optional "with Field Value and Field Value..." syntax
                let mut initial_fields = Vec::new();
                if self.check(&TokenType::With) {
                    self.advance(); // eat "with"
                    self.skip_ignorable();

                    loop {
                        // Parse field name
                        let field_name = self.expect_identifier()?;
                        self.skip_ignorable();

                        // Parse field value (expression without logical operators to avoid consuming "and")
                        let field_value = self.parse_comparison()?;
                        initial_fields.push((field_name, field_value));

                        self.skip_ignorable();

                        // Check if there's an "and" to continue parsing more fields
                        if self.check(&TokenType::And) {
                            self.advance(); // eat "and"
                            self.skip_ignorable();
                        } else {
                            break;
                        }
                    }
                }

                Ok(Statement::Create {
                    concept_name,
                    instance_name,
                    initial_fields,
                    line,
                })
            }

            Some(TokenType::Identifier(name)) => {
                if name == "Print" {
                    let line = self.current_line();
                    self.advance();
                    let value = self.parse_expression()?;
                    self.skip_ignorable();
                    return Ok(Statement::Print { value, line });
                }

                if name == "Set" {
                    let line = self.current_line();
                    self.advance();
                    let target = self.parse_expression()?;

                    if self.check(&TokenType::To_) {
                        self.advance();
                    } else if self.check(&TokenType::To) {
                        self.advance();
                    } else {
                        return Err(self.make_unexpected_token(
                            "to".to_string(),
                            self.peek_type().cloned().unwrap_or(TokenType::Eof),
                        ));
                    }

                    let value = self.parse_expression()?;
                    self.skip_ignorable();
                    return Ok(Statement::Set {
                        target,
                        value,
                        line,
                    });
                }

                if name == "Switch" {
                    let line = self.current_line();
                    self.advance();
                    let next_id = self.expect_identifier()?;
                    if next_id == "on" {
                        let situation = self.expect_identifier()?;
                        self.skip_ignorable();
                        return Ok(Statement::SwitchOn { situation, line });
                    } else if next_id == "off" {
                        let situation = self.expect_identifier()?;
                        self.skip_ignorable();
                        return Ok(Statement::SwitchOff { situation, line });
                    }
                    return Err(
                        self.make_invalid_syntax("Expected 'on' or 'off' after Switch".to_string())
                    );
                }

                let next_is_assign = self
                    .tokens
                    .peek()
                    .map_or(false, |t| t.token_type == TokenType::Is);

                if next_is_assign {
                    let line = self.current_line();
                    let target = self.expect_identifier()?;
                    self.advance();
                    let value = self.parse_expression()?;
                    self.skip_ignorable();
                    return Ok(Statement::Assignment {
                        target,
                        value,
                        line,
                    });
                }

                let line = self.current_line();
                let expr = self.parse_expression()?;
                self.skip_ignorable();
                Ok(Statement::Expression { expr, line })
            }

            Some(TokenType::If) => self.parse_if(),
            Some(TokenType::When) => self.parse_when(),
            Some(TokenType::Try) => self.parse_try_catch(),
            Some(TokenType::Repeat) => self.parse_repeat(),
            Some(TokenType::For) => self.parse_for(),
            Some(TokenType::Return) => self.parse_return(),
            Some(TokenType::Break) => {
                let line = self.current_line();
                self.advance();
                self.skip_ignorable();
                Ok(Statement::Break { line })
            }
            Some(TokenType::Continue) => {
                let line = self.current_line();
                self.advance();
                self.skip_ignorable();
                Ok(Statement::Continue { line })
            }

            _ => {
                let line = self.current_line();
                let expr = self.parse_expression()?;
                self.skip_ignorable();
                Ok(Statement::Expression { expr, line })
            }
        }
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        let line = self.current_line();
        self.expect(TokenType::If)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let then_body = self.parse_block()?;

        let else_body = if self.check(&TokenType::Else) {
            self.advance();
            self.skip_ignorable();

            if self.check(&TokenType::If) {
                let elif_stmt = self.parse_if()?;
                Some(vec![elif_stmt])
            } else {
                self.expect(TokenType::Colon)?;
                self.skip_ignorable();
                self.expect(TokenType::Indent)?;
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_body,
            else_body,
            line,
        })
    }

    fn parse_when(&mut self) -> Result<Statement, ParseError> {
        let line = self.current_line();
        self.expect(TokenType::When)?;
        let value = self.parse_expression()?;
        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;

        let mut cases = Vec::new();
        let mut otherwise = None;

        // Parse Is cases and optional Otherwise
        loop {
            self.skip_ignorable(); // Skip newlines/whitespace between cases

            if self.check(&TokenType::Is) {
                self.advance();
                let match_value = self.parse_expression()?;
                self.expect(TokenType::Colon)?;
                self.skip_ignorable();
                self.expect(TokenType::Indent)?;
                let body = self.parse_block()?;
                cases.push((match_value, body));
            } else if self.check(&TokenType::Otherwise) {
                self.advance();
                self.expect(TokenType::Colon)?;
                self.skip_ignorable();
                self.expect(TokenType::Indent)?;
                otherwise = Some(self.parse_block()?);
                break;
            } else if self.check(&TokenType::Dedent) {
                break;
            } else {
                return Err(self.make_invalid_syntax(
                    "Expected 'Is' or 'Otherwise' in When block".to_string(),
                ));
            }
        }

        if cases.is_empty() {
            return Err(self.make_invalid_syntax(
                "When statement must have at least one 'Is' case".to_string(),
            ));
        }

        // Consume the final DEDENT that ends the When block
        if self.check(&TokenType::Dedent) {
            self.advance();
        }

        Ok(Statement::When {
            value,
            cases,
            otherwise,
            line,
        })
    }

    fn parse_try_catch(&mut self) -> Result<Statement, ParseError> {
        let line = self.current_line();
        self.expect(TokenType::Try)?;
        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;
        let try_body = self.parse_block()?;

        let mut catch_var = None;
        let mut catch_body = None;
        let mut always_body = None;

        self.skip_ignorable();

        // Parse optional Catch block
        if self.check(&TokenType::Catch) {
            self.advance();
            if let Some(TokenType::Identifier(var_name)) = self.peek_type() {
                catch_var = Some(var_name.clone());
                self.advance();
            }

            self.expect(TokenType::Colon)?;
            self.skip_ignorable();
            self.expect(TokenType::Indent)?;
            catch_body = Some(self.parse_block()?);
            self.skip_ignorable();
        }

        // Parse optional Always block
        if self.check(&TokenType::Always) {
            self.advance(); // Eat "Always"
            self.expect(TokenType::Colon)?;
            self.skip_ignorable();
            self.expect(TokenType::Indent)?;
            always_body = Some(self.parse_block()?);
        }

        // Consume the final DEDENT that ends the TryCatch block
        if self.check(&TokenType::Dedent) {
            self.advance();
        }

        Ok(Statement::TryCatch {
            try_body,
            catch_var,
            catch_body,
            always_body,
            line,
        })
    }

    fn parse_repeat(&mut self) -> Result<Statement, ParseError> {
        let line = self.current_line();
        self.expect(TokenType::Repeat)?;

        if self.check(&TokenType::While) {
            self.advance();
            let condition = self.parse_expression()?;
            self.expect(TokenType::Colon)?;
            self.skip_ignorable();
            self.expect(TokenType::Indent)?;
            let body = self.parse_block()?;
            return Ok(Statement::RepeatWhile {
                condition,
                body,
                line,
            });
        }

        let count = self.parse_expression()?;
        self.expect(TokenType::Times)?;

        // Optional: With variable
        let variable = if self.check(&TokenType::With) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;
        let body = self.parse_block()?;

        Ok(Statement::RepeatTimes {
            count,
            variable,
            body,
            line,
        })
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        let line = self.current_line();
        self.expect(TokenType::For)?;
        self.expect(TokenType::Each)?;
        let variable = self.expect_identifier()?;
        self.expect(TokenType::In)?;
        let iterable = self.parse_expression()?;
        self.expect(TokenType::Colon)?;
        self.skip_ignorable();
        self.expect(TokenType::Indent)?;
        let body = self.parse_block()?;

        Ok(Statement::ForEach {
            variable,
            iterable,
            body,
            line,
        })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        let line = self.current_line();
        self.expect(TokenType::Return)?;

        let value = if self.check(&TokenType::Newline) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.skip_ignorable();
        Ok(Statement::Return { value, line })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;

        while self.check(&TokenType::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        while self.check(&TokenType::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_additive()?;

        let op = match self.peek_type() {
            Some(TokenType::Equals) => {
                self.advance();
                BinaryOperator::Equal
            }
            Some(TokenType::NotEquals) => {
                self.advance();
                BinaryOperator::NotEqual
            }
            Some(TokenType::Greater) => {
                self.advance();
                BinaryOperator::Greater
            }
            Some(TokenType::Less) => {
                self.advance();
                BinaryOperator::Less
            }
            Some(TokenType::GreaterEq) => {
                self.advance();
                BinaryOperator::GreaterEq
            }
            Some(TokenType::LessEq) => {
                self.advance();
                BinaryOperator::LessEq
            }
            _ => return Ok(left),
        };

        let right = self.parse_additive()?;
        Ok(Expression::BinaryOp {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        })
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek_type() {
                Some(TokenType::Plus) => {
                    self.advance();
                    BinaryOperator::Add
                }
                Some(TokenType::Minus) => {
                    self.advance();
                    BinaryOperator::Subtract
                }
                _ => break,
            };

            let right = self.parse_multiplicative()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek_type() {
                Some(TokenType::Star) => {
                    self.advance();
                    BinaryOperator::Multiply
                }
                Some(TokenType::Slash) => {
                    self.advance();
                    BinaryOperator::Divide
                }
                Some(TokenType::Percent) => {
                    self.advance();
                    BinaryOperator::Modulo
                }
                _ => break,
            };

            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        match self.peek_type() {
            Some(TokenType::Not) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            }
            Some(TokenType::Minus) => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_type() {
                Some(TokenType::LeftBracket) => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenType::RightBracket)?;
                    expr = Expression::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(TokenType::Dot) => {
                    self.advance();
                    let member = self.expect_member_name()?;
                    if self.check(&TokenType::With) {
                        self.advance();

                        let mut arguments = Vec::new();
                        let arg_val = self.parse_comparison()?;
                        arguments.push((format!("arg0"), arg_val));

                        while self.check(&TokenType::And) {
                            self.advance(); // eat "and"
                            let arg_val = self.parse_comparison()?;
                            arguments.push((format!("arg{}", arguments.len()), arg_val));
                        }

                        expr = Expression::MethodCall {
                            object: Box::new(expr),
                            method: member,
                            arguments,
                        };
                    } else {
                        expr = Expression::MemberAccess {
                            object: Box::new(expr),
                            member,
                        };
                    }
                }
                Some(TokenType::LeftParen) => {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.check(&TokenType::RightParen) {
                                self.expect(TokenType::Comma)?;
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenType::RightParen)?;

                    expr = Expression::Call {
                        callee: Box::new(expr),
                        arguments: args,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.peek_type() {
            Some(TokenType::Number(n)) => {
                let num = n.clone();
                self.advance();
                Ok(Expression::Number(num))
            }
            Some(TokenType::String_(s)) => {
                let raw_string = s.clone();
                self.advance();
                self.parse_interpolated_string(&raw_string)
            }
            Some(TokenType::True_) => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            Some(TokenType::False_) => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            Some(TokenType::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Some(TokenType::LeftBracket) => self.parse_list(),
            Some(TokenType::LeftBrace) => self.parse_map(),
            Some(TokenType::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(expr)
            }
            Some(TokenType::Do) => {
                self.advance(); // eat "Do"
                self.expect(TokenType::In)?;
                self.expect(TokenType::Background)?;
                self.expect(TokenType::Colon)?;
                self.skip_ignorable();
                self.expect(TokenType::Indent)?;
                let body = self.parse_block()?;
                Ok(Expression::DoInBackground { body })
            }
            Some(TokenType::Proceed) => {
                self.advance();
                let mut arguments = Vec::new();

                if self.check(&TokenType::With) {
                    self.advance();
                    let arg_val = self.parse_comparison()?;
                    arguments.push(arg_val);

                    while self.check(&TokenType::And) {
                        self.advance();
                        let arg_val = self.parse_comparison()?;
                        arguments.push(arg_val);
                    }
                } else if self.check(&TokenType::LeftParen) {
                    self.advance();
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.check(&TokenType::RightParen) {
                                self.expect(TokenType::Comma)?;
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenType::RightParen)?;
                }

                Ok(Expression::Proceed { arguments })
            }
            _ => Err(self.make_unexpected_token(
                "expression".to_string(),
                self.current
                    .as_ref()
                    .map(|t| t.token_type.clone())
                    .unwrap_or(TokenType::Eof),
            )),
        }
    }

    fn parse_interpolated_string(&self, content: &str) -> Result<Expression, ParseError> {
        if !content.contains('{') {
            return Ok(Expression::String(content.to_string()));
        }

        let mut expressions = Vec::new();
        let mut last_pos = 0;
        let mut chars = content.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if c == '{' {
                let should_interpolate = match chars.peek() {
                    Some((_, next_c)) => next_c.is_alphabetic() || *next_c == '_',
                    None => false,
                };

                if !should_interpolate {
                    continue;
                }

                if i > last_pos {
                    let text_segment = &content[last_pos..i];
                    expressions.push(Expression::String(text_segment.to_string()));
                }

                let start_ident = i + 1;
                let mut end_ident = start_ident;
                let mut found_close = false;

                while let Some((j, c_inner)) = chars.peek() {
                    if *c_inner == '}' {
                        end_ident = *j;
                        found_close = true;
                        chars.next();
                        break;
                    } else {
                        chars.next();
                    }
                }

                if !found_close {
                    return Err(self.make_invalid_syntax(
                        "Unclosed string interpolation brace '}'".to_string(),
                    ));
                }

                let var_name = &content[start_ident..end_ident];
                let trimmed = var_name.trim();

                if trimmed.is_empty() {
                    return Err(
                        self.make_invalid_syntax("Empty string interpolation '{}'".to_string())
                    );
                }

                if trimmed.contains('.') {
                    let parts: Vec<&str> = trimmed.split('.').collect();
                    if parts.len() == 2 {
                        expressions.push(Expression::MemberAccess {
                            object: Box::new(Expression::Identifier(parts[0].to_string())),
                            member: parts[1].to_string(),
                        });
                    } else {
                        expressions.push(Expression::Identifier(trimmed.to_string()));
                    }
                } else {
                    expressions.push(Expression::Identifier(trimmed.to_string()));
                }

                last_pos = end_ident + 1;
            }
        }

        if last_pos < content.len() {
            expressions.push(Expression::String(content[last_pos..].to_string()));
        }

        if expressions.is_empty() {
            return Ok(Expression::String("".to_string()));
        }

        let mut iterator = expressions.into_iter();
        let first = iterator.next().unwrap();

        let final_expr = iterator.fold(first, |acc, expr| Expression::BinaryOp {
            left: Box::new(acc),
            operator: BinaryOperator::Add,
            right: Box::new(expr),
        });

        Ok(final_expr)
    }

    fn parse_list(&mut self) -> Result<Expression, ParseError> {
        self.expect(TokenType::LeftBracket)?;
        self.skip_ignorable_with_indent();

        let mut items = Vec::new();

        while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
            items.push(self.parse_expression()?);
            self.skip_ignorable_with_indent();

            if !self.check(&TokenType::RightBracket) {
                self.expect(TokenType::Comma)?;
                self.skip_ignorable_with_indent();
            }
        }

        self.expect(TokenType::RightBracket)?;
        Ok(Expression::List(items))
    }

    fn parse_map(&mut self) -> Result<Expression, ParseError> {
        self.expect(TokenType::LeftBrace)?;
        self.skip_ignorable_with_indent();

        let mut entries = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let key = self.expect_identifier()?;
            self.expect(TokenType::Colon)?;
            self.skip_ignorable_with_indent();
            let value = self.parse_expression()?;
            entries.push((key, value));
            self.skip_ignorable_with_indent();

            if !self.check(&TokenType::RightBrace) {
                self.expect(TokenType::Comma)?;
                self.skip_ignorable_with_indent();
            }
        }

        self.expect(TokenType::RightBrace)?;
        Ok(Expression::Map(entries))
    }

    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    fn peek_type(&self) -> Option<&TokenType> {
        self.current.as_ref().map(|t| &t.token_type)
    }

    fn current_line(&self) -> usize {
        self.current.as_ref().map(|t| t.line).unwrap_or(0)
    }

    fn current_column(&self) -> usize {
        self.current.as_ref().map(|t| t.column).unwrap_or(0)
    }

    fn make_unexpected_token(&self, expected: String, found: TokenType) -> ParseError {
        ParseError::UnexpectedToken {
            expected,
            found,
            line: self.current_line(),
            column: self.current_column(),
        }
    }

    fn make_invalid_syntax(&self, message: String) -> ParseError {
        ParseError::InvalidSyntax {
            message,
            line: self.current_line(),
            column: self.current_column(),
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if let Some(current_type) = self.peek_type() {
            std::mem::discriminant(current_type) == std::mem::discriminant(token_type)
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_type(), Some(TokenType::Eof) | None)
    }

    fn expect(&mut self, token_type: TokenType) -> Result<(), ParseError> {
        if self.check(&token_type) {
            self.advance();
            Ok(())
        } else {
            Err(self.make_unexpected_token(
                format!("{:?}", token_type),
                self.current
                    .as_ref()
                    .map(|t| t.token_type.clone())
                    .unwrap_or(TokenType::Eof),
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let Some(TokenType::Identifier(name)) = self.peek_type() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.make_unexpected_token(
                "identifier".to_string(),
                self.current
                    .as_ref()
                    .map(|t| t.token_type.clone())
                    .unwrap_or(TokenType::Eof),
            ))
        }
    }

    // Accept keywords as identifiers in member access contexts (e.g., Channel.Create)
    fn expect_member_name(&mut self) -> Result<String, ParseError> {
        match self.peek_type() {
            Some(TokenType::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Some(TokenType::Create) => {
                self.advance();
                Ok("Create".to_string())
            }
            Some(TokenType::Return) => {
                self.advance();
                Ok("Return".to_string())
            }
            _ => Err(self.make_unexpected_token(
                "member name".to_string(),
                self.current
                    .as_ref()
                    .map(|t| t.token_type.clone())
                    .unwrap_or(TokenType::Eof),
            )),
        }
    }
}

impl ParseError {
    pub fn location(&self) -> (usize, usize) {
        match self {
            ParseError::UnexpectedToken { line, column, .. }
            | ParseError::UnexpectedEof { line, column }
            | ParseError::InvalidSyntax { line, column, .. } => (*line, *column),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: expected {}, found {:?}",
                    line, column, expected, found
                )
            }
            ParseError::UnexpectedEof { line, column } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: unexpected end of file",
                    line, column
                )
            }
            ParseError::InvalidSyntax {
                message,
                line,
                column,
            } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: {}",
                    line, column, message
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
