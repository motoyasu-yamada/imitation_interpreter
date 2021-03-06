use std::collections::BTreeMap;
use super::token::{Token, TokenKind};
use super::lexer;
use super::errors::{Errors};
use super::ast::{Program, Statement, Statement::LetStatement,
                 Expression, Precedence};

#[derive(Debug, Clone)]
pub struct Parser<'a>  {
    lexer: lexer::Lexer<'a>,
    current_token: Token,
    next_token: Token,
}

impl<'a>  Parser<'a>  {
    pub fn new(l: lexer::Lexer<'a>) -> Self {
        let mut p = Parser{
            lexer: l,
            current_token: Token{token_type: TokenKind::DEFAULT, literal: "default".to_string() },
            next_token: Token{token_type: TokenKind::DEFAULT, literal: "default".to_string() },
        };
        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.current_token = self.next_token.clone();
        self.next_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Result<Program, Errors> {
        let mut statements: Vec<Statement> = vec![];

        // read token until it reaches at the end of sentence.
        while !self.is_current_token(TokenKind::EOF){
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.next_token();
        };
        Ok(Program {statements: statements})
    }

    fn parse_statement(&mut self) -> Result<Statement, Errors> {
        match self.current_token.token_type {
            TokenKind::LET => {
                Ok((self.parse_let_statement()?))
            },
            TokenKind::RETURN => {
                Ok(self.parse_return_statement()?)
            },
            _ => {
                Ok(self.parse_expression_statement()?)
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, Errors> {
        // Since current token is let, next_token() is implemented in order to read identifier next to 'let'.
        self.next_token();

        // If current token is identifier, next token must be '==' or '!='
        if !self.is_current_token(TokenKind::IDENT) || self.expect_next_token(TokenKind::IDENT){
            return Err(Errors::TokenInvalid(self.next_token.clone()))
        }
        let identifier = Expression::Identifier(self.current_token.literal.clone());
        // If there isn't assign next to identifier, return error.
        if !self.expect_next_token(TokenKind::ASSIGN) {
            return Err(Errors::TokenInvalid(self.next_token.clone()))
        }
        // skip a assign token
        self.next_token();

        // get right side value.
        let stmt_value = self.parse_expression(Precedence::LOWEST)?;
        if self.is_next_token(TokenKind::SEMICOLON) {
            self.next_token();
        }
        let stmt = LetStatement {
                identifier: identifier,
                value: stmt_value
        };
        return Ok(stmt)
    }

    fn parse_return_statement(&mut self) -> Result<Statement, Errors> {
        // skip return value and then read value next to return.
        self.next_token();
        let return_value = self.parse_expression(Precedence::LOWEST)?;

        // read token until it reaches at the end of sentence.
        while !self.is_current_token(TokenKind::SEMICOLON) {
            self.next_token()
        }
        return Ok(Statement::Return(return_value))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, Errors> {
        let expression = self.parse_expression(Precedence::LOWEST)?;
        if self.is_next_token(TokenKind::SEMICOLON) {
            self.next_token()
        }
        return Ok(Statement::ExpressionStatement(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, Errors> {
        let mut exp = match self.current_token.token_type {
            TokenKind::IDENT => {Expression::Identifier(self.parse_identifier()?)},
            TokenKind::STRING => {
                Expression::String(self.parse_string()?)},
            TokenKind::INT => Expression::Integer(self.parse_integer()?),
            TokenKind::TRUE => Expression::Bool(true),
            TokenKind::FALSE => Expression::Bool(false),
            TokenKind::IF =>   self.parse_if_expression()?,
            TokenKind::LPAREN => self.parse_grouped_expression()?,
            TokenKind::LBRACE => self.parse_hash_literal()?,
            TokenKind::LBRACKET => self.parse_array_literal()?,
            TokenKind::FUNCTION => self.parse_function_expression()?,
            TokenKind::BANG => self.parse_prefix_expression()?,
            TokenKind::MINUS => self.parse_prefix_expression()?,
            _ => return Err(Errors::TokenInvalid(self.current_token.clone()))
        };
        while !self.is_next_token(TokenKind::SEMICOLON) && precedence < self.next_precedence() {
            //operator must be set in current position,
            //so token must be read once forward.
            match self.next_token.token_type {
                TokenKind::PLUS => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::MINUS => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::SLASH => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::ASTERISK => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::EQ => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::NotEq => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::LT => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::GT => {
                    self.next_token();
                    exp =  self.parse_infix_expression(exp)?;
                },
                TokenKind::LPAREN => {
                    self.next_token();
                    exp =  self.parse_call_arguments(exp)?;
                },
                TokenKind::LBRACKET => {
                    self.next_token();
                    exp =  self.parse_index_expression(exp)?;
                },
                _ => {
                    return Ok(exp);                
                }
            }
        }
        return Ok(exp)
    }

    fn parse_identifier(&mut self) -> Result<String, Errors> {
        return Ok(self.current_token.literal.to_string())
    }

    fn parse_string(&mut self) -> Result<String, Errors> {
        return Ok(self.current_token.literal.to_string())
    }

    fn parse_integer(&mut self) -> Result<i32, Errors> {
        return Ok(self.current_token.literal.parse::<i32>().unwrap())
    }
    fn parse_hash_literal(&mut self) -> Result<Expression, Errors> {
        let mut pairs = BTreeMap::new();

        while !self.is_next_token(TokenKind::RBRACE) {
            self.next_token();
            let key = self.parse_expression(Precedence::LOWEST)?;
            if !self.expect_next_token(TokenKind::COLON) {
                return Ok(Expression::Null)
            }
            self.next_token();
            let value = self.parse_expression(Precedence::LOWEST)?;
            // the values inside btree_map is alphabetically ordered.
            pairs.insert(Box::new(key), Box::new(value));
            if !self.is_next_token(TokenKind::RBRACE) && !self.expect_next_token(TokenKind::COMMA) {
                return Ok(Expression::Null)
            }
        }
        if !self.expect_next_token(TokenKind::RBRACE) {
            return Ok(Expression::Null)
        }

        return Ok(Expression::Hashmap(pairs))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, Errors> {
        match self.parse_expression_list(TokenKind::RBRACKET)? {
            list => Ok(Expression::Array(list)),
            _ => Ok(Expression::Null)
        }
    }

    fn parse_expression_list(&mut self, end: TokenKind)-> Result<Vec<Expression>, Errors> {
        let mut list: Vec<Expression> = vec![];
        
        if self.is_next_token(end) {
            self.next_token();
            return Ok(list)
        } else {
            // skip left bracket;
            self.next_token();
            list.push(self.parse_expression(Precedence::LOWEST)?);

            // fetch values inside list.
            while self.is_next_token(TokenKind::COMMA) {
                self.next_token();
                self.next_token();
                list.push(self.parse_expression(Precedence::LOWEST)?)
            }
            if self.expect_next_token(end) {
                Ok(list)
            } else {
                unimplemented!()
            }
        }
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, Errors> {
        self.next_token();
        let index = self.parse_expression(Precedence::LOWEST)?;
        if !self.expect_next_token(TokenKind::RBRACKET) {
            return Ok(Expression::Null)
        }
        Ok(Expression::IndexExpression{array: Box::new(left), subscript: Box::new(index)})
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression, Errors> {
        self.next_token();
        let lparen = self.parse_expression(Precedence::LOWEST)?;
        if self.expect_next_token(TokenKind::RPAREN) {
             return Ok(lparen)
        }  else {
            Err(Errors::TokenInvalid(self.current_token.clone()))
        }
}

    fn parse_if_expression(&mut self) ->  Result<Expression, Errors> {
        if !self.is_next_token(TokenKind::LPAREN) {
            return Ok(Expression::Null)
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST);

        if !self.expect_next_token(TokenKind::LBRACE) {
            return Ok(Expression::Null)
            }
        let expression = Expression::IfExpression{
                            condition: Box::new(condition?),
                            consequence: Box::new(self.parse_block_statements(TokenKind::LBRACE)?),
                            alternative: self.alternative()?,
                                                  };
        Ok(expression)
    }

    fn parse_block_statements(&mut self, token_kind: TokenKind) -> Result<Statement, Errors> {
        self.next_token();
        let mut statements: Vec<Statement> = vec![];
        while !self.is_current_token(TokenKind::RBRACE) && !self.is_current_token(TokenKind::EOF) {
            let statement = self.parse_statement()?;
            statements.push(statement);
            self.next_token();
        }
        Ok(Statement::Block(statements))
    }

    fn alternative(&mut self) -> Result<Option<Box<Statement>>, Errors> {
        if self.is_next_token(TokenKind::ELSE) {
        self.next_token();
        if self.expect_next_token(TokenKind::LBRACE) {
            let alternative = self.parse_block_statements(TokenKind::LBRACE)?;
            Ok(Some(Box::new(alternative)))
        }else {
            return Err(Errors::TokenInvalid(self.current_token.clone()))
        }
    } else {
           Ok(None)
        }
    }

    fn parse_function_expression(&mut self) -> Result<Expression, Errors> {
        if self.expect_next_token(TokenKind::LPAREN) {
            println!("TokenKind should be LPAREN but actually is {:?}",self.next_token.token_type)            
        }
        let parameters = self.parse_function_parameters()?;
        if self.expect_next_token(TokenKind::LBRACE) {
            println!("TokenKind should be LBRACE but actually is {:?}",self.next_token.token_type)            
        }        

        let body = self.parse_block_statements(TokenKind::LBRACE)?;
        let expression = Expression::FunctionLiteral{
            parameters: parameters,
            body: Box::new(body)
        };
        Ok(expression)
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Expression>, Errors> {
        let mut identifiers = vec![];
        // if next_token is ")", there are no parameters 
        if self.is_next_token(TokenKind::RPAREN) {
            self.next_token();
            return Ok(identifiers)
        }
        // if function has one or more parameters
        // skip "(" and push these into list.
        self.next_token();
        identifiers.push(Expression::Identifier(self.current_token.literal.clone()));
        while self.is_next_token(TokenKind::COMMA) {
            self.next_token();
            self.next_token();
        identifiers.push(Expression::Identifier(self.current_token.literal.clone()));
        }
        if !self.expect_next_token(TokenKind::RPAREN) {
            panic!()
        }
        Ok(identifiers)
    }

    fn parse_call_arguments(&mut self, func: Expression) -> Result<Expression, Errors> {
        let mut arguments = vec![];

        if self.is_next_token(TokenKind::RPAREN) {
            self.next_token();
            return Ok(Expression::CallExpression{function: Box::new(func), body: arguments})
        } else {
        self.next_token();
        arguments.push(self.parse_expression(Precedence::LOWEST)?);
        while self.is_next_token(TokenKind::COMMA) {
            self.next_token();
            self.next_token();
            arguments.push(self.parse_expression(Precedence::LOWEST)?);
        }
        if !self.expect_next_token(TokenKind::RPAREN) {
            return Ok(Expression::Null)
                }
            }
        Ok(Expression::CallExpression{function: Box::new(func), body: arguments})
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, Errors> {
        let current_token = self.current_token.literal.to_string();
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX)?;
        let expression = Expression::PrefixExpression{
                                           operator: current_token,
                                           right_expression: Box::new(right)
                                        };
        return Ok(expression)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, Errors> {
        let operator = match self.current_token.token_type {
            TokenKind::PLUS => "+".to_string(),
            TokenKind::MINUS => "-".to_string(),
            TokenKind::ASTERISK => "*".to_string(),
            TokenKind::SLASH => "/".to_string(),
            TokenKind::EQ => "==".to_string(),
            TokenKind::NotEq => "!=".to_string(),
            TokenKind::LT => "<".to_string(),
            TokenKind::GT => ">".to_string(),
            _ => {panic!()}
        };
        // current token will be read in parse_expression().
        // next token must be implemented in order that next operator is set to current_token
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        let infix_expression = Expression::InfixExpression{
                                    left_expression: Box::new(left),
                                    operator: operator,
                                    right_expression: Box::new(right)
        };
        return Ok(infix_expression)
    }

    fn current_precedence(&mut self) -> Precedence {
        return self.current_token.get_precedence()
    }

    fn next_precedence(&mut self) -> Precedence {
        return self.next_token.get_precedence()
    }

    fn is_current_token(&self, token_kind: TokenKind) -> bool {
        self.current_token.token_type == token_kind
    }

    fn is_next_token(&self, token_kind: TokenKind) -> bool {
        self.next_token.token_type == token_kind
    }

    fn expect_next_token(&mut self, token_kind: TokenKind) -> bool {
        if self.is_next_token(token_kind){
            self.next_token();
            return true
        } else {
            return false
        }
    }
}

// if cfg(test) is written, test code is compiled only when test runs
#[cfg(test)]// test runs only when execute cargo run
mod testing {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::str::FromStr;


    #[test]
    fn test_let_statement() {
        let input = r#"let x = 5;
                       let y = 10;
                       let foobar = 838383;"#;
        
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        let stmt = format!("{}", &program.statements[0]);
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_return_statement() {
        let input = r#"return 5;
                       return 10;
                       return 993322;"#;
        
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 3);
        let tests = vec![
            "return 5",
            "return 10",
            "return 993322",
        ];

        for (i, test) in tests.iter().enumerate() {
            let stmt = format!("{}", &program.statements[i]);
            assert_eq!(*stmt, **test);
        }
    }
    #[test]
    fn test_identifier_expression() {
        let input = "foobar;".to_string();
        
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.statements.len(), 1); // 識別子が一つであること
        let stmt = format!("{}", program.statements[0]);
        assert_eq!(stmt, "foobar");
        }

        #[test]
        fn test_interger_expression() {
            let input = "5".to_string();
            
            let lexer = Lexer::new(&input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
            let stmt = format!("{}", program.statements[0]);
            assert_eq!(stmt, "5".to_string());
            }

        #[test]
        fn test_prefix_expression() {
            let prefix_tests = vec!["!5","-15"];
            // compare the result of parseing the first element of tuple
            // with second, third elements.
            for test in prefix_tests.iter() {
                let lexer = Lexer::new(test);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
                let statements = format!("{}", program.statements[0]);
                for (i, statement) in prefix_tests.iter().enumerate() {
                    assert_eq!(*statement, prefix_tests[i]);
                }
            }
        }

            #[test]
            fn test_infix_expression() {
                let infix_tests = vec!["5 + 5;", "5 - 5;", "5 * 5;", "5 / 5;",
                                       "5 > 5;", "5 < 5;", "5 == 5;", "5 != 5;"];
                // compare the result of parseing the first element of tuple
                // with second, third elements.
                for test in infix_tests.iter() {
                    let lexer = Lexer::new(test);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program().unwrap();
                    assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
                    let statements = format!("{}", program.statements[0]);
                    for (i, statement) in infix_tests.iter().enumerate() {
                        assert_eq!(*statement, infix_tests[i]);
                    }
                }
            }

            #[test]
            fn test_operator_precedence_parsing() {
                  let infix_tests = vec![
                                        ("((-a) * b)", "-a * b"),
                                        ("(!(-a))", "!-a"), 
                                        ("((a + b) + c)", "a + b + c"),
                                        ("((a + b) - c)", "a + b - c"),
                                        ("((a * b) * c)", "a * b * c"),
                                        ("((a * b) / c)", "a * b / c"), 
                                        ("(a + (b / c))", "a + b / c"),
                                        ("(1 + (2 + 3)) + 4","1 + 2 + 3 + 4"),
                                        ("((5 + 5) * 2)", "5 + 5 * 2"),
                                        ("(2 / (5 + 5))", "2 / 5 + 5"),
                                        ("(-(5 + 5))", "-5 + 5"),
                                        ("(!(true == true))", "!true == true"),
                                        ("((a * ([1, 2, 3, 4][(b * c)])) * d)", "a * [1, 2, 3, 4][b * c] * d"),
                                        ];
                // compare the result of parseing the first element of tuple
                // with second, third elements.
                for (i, test) in infix_tests.iter().enumerate() {
                    let lexer = Lexer::new(test.0);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program().unwrap();
                    let statements = format!("{}", program.statements[0]);
                    assert_eq!(statements, test.1);
                }
            }


                #[test]
                fn test_bool_expression() {
                    let bool_tests = vec![
                                        ("true", true),
                                        ("false", false),
                                        ];
                    // compare the result of parseing the first element of tuple
                    // with second, third elements.
                    for (test, right) in bool_tests.iter() {
                        let lexer = Lexer::new(test);
                        let mut parser = Parser::new(lexer);
                        let program = parser.parse_program().unwrap();
                        assert_eq!(program.statements.len(), 1); // confirm the number of statements is 1.
                        let statements = format!("{}", program.statements[0]);
                        assert_eq!(FromStr::from_str(&statements.to_string()[..]), Ok(*right));
                        }
                    }    
            #[test]
            fn test_bool_infix_expression() {
                let bool_tests = vec![
                                    ("3 > 5 == false", 3, ">", 5, "==", "false"),
                                    ("3 > 5 == false", 3, "<", 5, "==", "true"),
                                    ];
                // compare the result of parseing the first element of tuple
                // with second, third elements.
                for (test, left, operator, right, bool_ident, bool_literal) in bool_tests.iter() {
                    let lexer = Lexer::new(test);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program();
                    }
                }    

            #[test]
            fn test_if_expression() {
                let input = "if (1 > 2) {10} else {20}".to_string();
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                println!("{:?}", statements);
                assert_eq!(input, statements);
                }

            #[test]
            fn test_function_expression() {
                let input = "fn (x, y) {x + y}".to_string();
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!(input, statements);
                }

            #[test]
            fn test_call_expression() {
                let input = "add(1, 2 * 3, 4 + 5);".to_string();
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!(input, statements);
                }
            #[test]
            fn test_string_literal_expression() {
                let input = r#""Hello world;""#;
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!("Hello world;", statements);
                }

            #[test]
            fn test_parse_array_literals() {
                let input = "[1, 2 * 2, 3 + 3]";
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!("[1, 2 * 2, 3 + 3]", statements);
                }
            #[test]
            fn test_parse_index_expressions() {
                let input = "my_array[1 + 1]";
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!("my_array[1 + 1]", statements);
                }
            #[test]
            fn test_parse_hash_literal_string_keys() {
                let input = "{\"a\": 4, \"b\": 1, \"c\": 3, \"d\": 2}";
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!("{a: 4, b: 1, c: 3, d: 2}", statements);
                }
            #[test]
            fn test_parse_empty_hash_literal() {
                let input = "{}";
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!("{}", statements);
                }
            #[test]
            fn test_parse_hash_literal_with_expressions() {
                let input = "{\"one\": 0 + 1, \"two\": 10-8, \"three\": 15 / 5}";
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program().unwrap();
                let statements = format!("{}", program.statements[0]);
                assert_eq!("{one: 0 + 1, three: 15 / 5, two: 10 - 8}", statements);
                    }
            }