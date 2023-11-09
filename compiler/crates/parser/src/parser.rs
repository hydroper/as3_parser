use std::rc::Rc;
use crate::*;

pub struct Parser<'input> {
    tokenizer: Tokenizer<'input>,
    previous_token: (Token, Location),
    token: (Token, Location),
}

impl<'input> Parser<'input> {
    /// Constructs a parser.
    pub fn new(source: &'input Rc<Source>) -> Self {
        Self {
            tokenizer: Tokenizer::new(source),
            previous_token: (Token::Eof, Location::with_line_and_offset(&source, 1, 0)),
            token: (Token::Eof, Location::with_line_and_offset(&source, 1, 0)),
        }
    }

    fn source(&self) -> &Rc<Source> {
        &self.tokenizer.source
    }

    fn add_syntax_error(&self, location: Location, kind: DiagnosticKind, arguments: Vec<Box<DiagnosticArgument>>) {
        self.source().add_diagnostic(Diagnostic::new_syntax_error(location, kind, arguments));
    }

    fn add_warning(&self, location: Location, kind: DiagnosticKind, arguments: Vec<Box<DiagnosticArgument>>) {
        self.source().add_diagnostic(Diagnostic::new_warning(location, kind, arguments));
    }

    fn next(&mut self, reserved_words: bool) -> Result<(), ParserFailure> {
        self.previous_token = self.token.clone();
        self.token = self.tokenizer.scan_ie_div(reserved_words)?;
        Ok(())
    }

    fn next_ie_xml_tag(&mut self) -> Result<(), ParserFailure> {
        self.previous_token = self.token.clone();
        self.token = self.tokenizer.scan_ie_xml_tag()?;
        Ok(())
    }

    fn next_ie_xml_content(&mut self) -> Result<(), ParserFailure> {
        self.previous_token = self.token.clone();
        self.token = self.tokenizer.scan_ie_xml_content()?;
        Ok(())
    }

    fn consume(&mut self, token: Token) -> Result<bool, ParserFailure> {
        if self.token.0 == token {
            self.next(true)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn consume_identifier(&mut self, reserved_words: bool) -> Result<Option<(String, Location)>, ParserFailure> {
        if let Token::Identifier(id) = self.token.0.clone() {
            let location = self.token.1.clone();
            self.next(true)?;
            Ok(Some((id, location)))
        } else {
            if reserved_words {
                if let Some(id) = self.token.0.keyword_name() {
                    let location = self.token.1.clone();
                    self.next(true)?;
                    return Ok(Some((id, location)));
                }
            }
            Ok(None)
        }
    }

    fn consume_context_keyword(&mut self, name: String) -> Result<bool, ParserFailure> {
        if let Token::Identifier(id) = self.token.0.clone() {
            if id == name {
                self.next(true)?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), ParserFailure> {
        if self.token.0 != token {
            self.add_syntax_error(self.token.1.clone(), DiagnosticKind::Expected, diagnostic_arguments![Token(token), Token(self.token.0.clone())]);
            Err(ParserFailure)
        } else {
            self.next(true)?;
            Ok(())
        }
    }

    fn expect_identifier(&mut self, reserved_words: bool) -> Result<(String, Location), ParserFailure> {
        if let Token::Identifier(id) = self.token.0.clone() {
            let location = self.token.1.clone();
            self.next(true)?;
            Ok((id, location))
        } else {
            if reserved_words {
                if let Some(id) = self.token.0.keyword_name() {
                    let location = self.token.1.clone();
                    self.next(true)?;
                    return Ok((id, location));
                }
            }
            self.add_syntax_error(self.token.1.clone(), DiagnosticKind::ExpectedIdentifier, diagnostic_arguments![Token(self.token.0.clone())]);
            Err(ParserFailure)
        }
    }

    fn expect_context_keyword(&mut self, name: String) -> Result<(), ParserFailure> {
        if let Token::Identifier(id) = self.token.0.clone() {
            if id == name {
                self.next(true)?;
                return Ok(());
            }
        }
        self.add_syntax_error(self.token.1.clone(), DiagnosticKind::Expected, diagnostic_arguments![String(name), Token(self.token.0.clone())]);
        Err(ParserFailure)
    }

    /// Expects a greater-than symbol. If the facing token is not greater-than,
    /// but starts with a greater-than symbol, the first character is shifted off
    /// from the facing token.
    fn expect_generics_gt(&mut self) -> Result<(), ParserFailure> {
        match self.token.0 {
            Token::Gt => {
                self.next(true)?;
                Ok(())
            },
            Token::Ge => {
                self.token.0 = Token::Assign;
                self.token.1.first_offset += 1;
                Ok(())
            },
            Token::RightShift => {
                self.token.0 = Token::Gt;
                self.token.1.first_offset += 1;
                Ok(())
            },
            Token::RightShiftAssign => {
                self.token.0 = Token::Ge;
                self.token.1.first_offset += 1;
                Ok(())
            },
            Token::UnsignedRightShift => {
                self.token.0 = Token::RightShift;
                self.token.1.first_offset += 1;
                Ok(())
            },
            Token::UnsignedRightShiftAssign => {
                self.token.0 = Token::RightShiftAssign;
                self.token.1.first_offset += 1;
                Ok(())
            },
            _ => {
                self.expect(Token::Gt)
            },
        }
    }

    pub fn expect_eof(&mut self) -> Result<(), ParserFailure> {
        self.expect(Token::Eof)
    }

    pub fn parse_opt_expression(&mut self) -> Result<Option<ast::Expression>, ParserFailure> {
        //
    }
}