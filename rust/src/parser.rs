use super::ssnf::ssnf;
use std::{str::Chars, usize, vec};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AltArg {
    Concat {
        // Нам всё равно в какой форме конкат: в скобках или нет
        args: Vec<ConcatArg>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
    },
    Star(Box<StarArg>),
    Regex {
        arg: String,
        parenthesized: bool,
    },
}

impl From<ParsingResult> for AltArg {
    fn from(value: ParsingResult) -> Self {
        match value {
            ParsingResult::Alt { args, accepts_empty } => {
                unreachable!();
            }
            ParsingResult::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized,
            } => {
                AltArg::Concat { args, body_accepts_empty, tail_accepts_empty }
            }
            ParsingResult::Star(arg) => AltArg::Star(arg),
            ParsingResult::Regex { arg, parenthesized } => {
                AltArg::Regex { arg, parenthesized }
            }
        }
    }
}

impl From<ParsingResult> for ConcatArg {
    fn from(value: ParsingResult) -> Self {
        match value {
            ParsingResult::Alt { args, accepts_empty } => {
                ConcatArg::Alt { args, accepts_empty }
            }
            ParsingResult::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized,
            } => ConcatArg::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
            },
            ParsingResult::Star(arg) => ConcatArg::Star(arg),
            ParsingResult::Regex { arg, parenthesized } => {
                ConcatArg::Regex { arg, parenthesized }
            }
        }
    }
}

impl From<ParsingResult> for StarArg {
    fn from(value: ParsingResult) -> Self {
        match value {
            ParsingResult::Alt { args, accepts_empty } => {
                StarArg::Alt { args, accepts_empty }
            }
            ParsingResult::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized,
            } => {
                StarArg::Concat { args, body_accepts_empty, tail_accepts_empty }
            }
            ParsingResult::Star(arg) => unreachable!(),
            ParsingResult::Regex { arg, parenthesized } => StarArg::Regex(arg),
        }
    }
}

impl From<AltArg> for ParsingResult {
    fn from(value: AltArg) -> Self {
        match value {
            AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
                ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: true,
                }
            }
            AltArg::Star(arg) => ParsingResult::Star(arg),
            AltArg::Regex { arg, parenthesized } => {
                ParsingResult::Regex { arg, parenthesized }
            }
        }
    }
}

impl ToString for AltArg {
    fn to_string(&self) -> String {
        match self {
            AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
                args.iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            }
            AltArg::Star(arg) => format!("({})*", arg.to_string()),
            AltArg::Regex { arg, parenthesized } => {
                if *parenthesized {
                    format!("({arg})")
                } else {
                    arg.to_string()
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConcatArg {
    Concat {
        // конкат внутри конката может быть ТОЛЬКО в скобках
        args: Vec<ConcatArg>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
    },
    Alt {
        args: Vec<AltArg>,
        accepts_empty: bool,
    },
    Star(Box<StarArg>),
    Regex {
        arg: String,
        parenthesized: bool,
    },
}

impl From<ConcatArg> for ParsingResult {
    fn from(value: ConcatArg) -> Self {
        match value {
            ConcatArg::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
            } => ParsingResult::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized: true,
            },
            ConcatArg::Alt { args, accepts_empty } => {
                ParsingResult::Alt { args, accepts_empty }
            }
            ConcatArg::Star(arg) => ParsingResult::Star(arg),
            ConcatArg::Regex { arg, parenthesized } => {
                ParsingResult::Regex { arg, parenthesized }
            }
        }
    }
}

impl ToString for ConcatArg {
    fn to_string(&self) -> String {
        match self {
            ConcatArg::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
            } => format!(
                "({})",
                args.iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            ConcatArg::Alt { args, accepts_empty } => {
                format!(
                    "({})",
                    args.iter()
                        .map(|item| { item.to_string() })
                        .collect::<Vec<String>>()
                        .join("|")
                )
            }
            ConcatArg::Star(arg) => format!("({})*", arg.to_string()),
            ConcatArg::Regex { arg, parenthesized } => {
                if *parenthesized {
                    format!("({arg})")
                } else {
                    arg.to_string()
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StarArg {
    Alt {
        args: Vec<AltArg>,
        accepts_empty: bool,
    },
    Concat {
        // конкат внутри звезды может быть только в скобках
        args: Vec<ConcatArg>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
    },
    Regex(String),
}

impl From<StarArg> for ParsingResult {
    fn from(value: StarArg) -> Self {
        match value {
            StarArg::Alt { args, accepts_empty } => {
                ParsingResult::Alt { args, accepts_empty }
            }
            StarArg::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
            } => ParsingResult::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized: true,
            },
            StarArg::Regex(arg) => {
                ParsingResult::Regex { arg, parenthesized: true }
            }
        }
    }
}

impl ToString for StarArg {
    fn to_string(&self) -> String {
        match self {
            StarArg::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
            } => args
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(""),
            StarArg::Alt { args, accepts_empty } => args
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join("|"),
            StarArg::Regex(arg) => arg.to_string(),
        }
    }
}

#[derive(Default)]
pub struct Parser<'a> {
    index: usize,
    expr_iter: Option<Chars<'a>>,
    next_char: Option<char>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParsingResult {
    Alt {
        args: Vec<AltArg>,
        accepts_empty: bool,
    },
    Concat {
        args: Vec<ConcatArg>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
        parenthesized: bool,
    },
    Star(Box<StarArg>),
    Regex {
        arg: String,
        parenthesized: bool,
    },
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self, regex: &'a str) -> ParsingResult {
        self.index = 0;
        self.expr_iter = Some(regex.chars());
        self.next_char = None;
        let mut res = self.expect_alternative();
        ssnf(res)
    }

    fn expect_alternative(&mut self) -> ParsingResult {
        let mut alt_args: Vec<AltArg> = vec![];
        let mut alt_accepts_empty = true;

        let mut last_concat_parens = false;
        loop {
            let subexpr = self.expect_unary();
            match subexpr {
                ParsingResult::Alt { args, accepts_empty: accept_empty } => {
                    alt_args.extend(args);
                    alt_accepts_empty &= accept_empty;
                }
                ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized,
                } => {
                    last_concat_parens = parenthesized;
                    alt_args.push(AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    });
                    alt_accepts_empty &=
                        body_accepts_empty & tail_accepts_empty;
                }
                ParsingResult::Star(arg) => {
                    alt_args.push(AltArg::Star(arg));
                }
                ParsingResult::Regex { arg, parenthesized } => {
                    alt_args.push(AltArg::Regex { arg, parenthesized });
                    alt_accepts_empty = false;
                }
            }
            if !self.check('|') {
                break;
            }
            self.advance();
        }

        if alt_args.len() >= 2 {
            ParsingResult::Alt {
                args: alt_args,
                accepts_empty: alt_accepts_empty,
            }
        } else if alt_args.len() == 1 {
            match alt_args.pop().unwrap() {
                AltArg::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: last_concat_parens,
                },
                AltArg::Star(arg) => ParsingResult::Star(arg),
                AltArg::Regex { arg, parenthesized } => {
                    ParsingResult::Regex { arg, parenthesized }
                }
            }
        } else {
            unreachable!();
        }
    }

    // UNARY ::= (CONCAT "*"+)* CONCAT "*"*
    fn expect_unary(&mut self) -> ParsingResult {
        let mut unary_args: Vec<ConcatArg> = vec![];
        let mut unary_body_accepts_empty: bool = true;
        let mut unary_tail_accepts_empty: bool = true;

        let mut last_concat_parens = false;
        loop {
            if self.check('|') || self.check(')') || self.at_end() {
                break;
            }

            let subexpr = self.expect_concat();
            if !self.check('*') {
                match subexpr {
                    ParsingResult::Alt { args, accepts_empty } => {
                        unary_body_accepts_empty &= unary_tail_accepts_empty;
                        unary_tail_accepts_empty = accepts_empty;
                        unary_args.push(ConcatArg::Alt { args, accepts_empty });
                    }
                    ParsingResult::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                        parenthesized,
                    } => {
                        last_concat_parens = parenthesized;
                        if parenthesized {
                            unary_body_accepts_empty &=
                                unary_tail_accepts_empty;
                            unary_tail_accepts_empty =
                                body_accepts_empty & tail_accepts_empty;
                            unary_args.push(ConcatArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            })
                        } else {
                            unary_args.extend(args);
                            unary_body_accepts_empty &=
                                unary_tail_accepts_empty & body_accepts_empty;
                            unary_tail_accepts_empty = tail_accepts_empty;
                        }
                    }
                    ParsingResult::Star(arg) => {
                        unary_body_accepts_empty &= unary_tail_accepts_empty;
                        unary_tail_accepts_empty = true;
                        unary_args.push(ConcatArg::Star(arg))
                    }
                    ParsingResult::Regex { arg, parenthesized } => {
                        unary_body_accepts_empty &= unary_tail_accepts_empty;
                        unary_tail_accepts_empty = false;
                        unary_args
                            .push(ConcatArg::Regex { arg, parenthesized });
                    }
                }
                break;
            }
            loop {
                self.advance();
                if !self.check('*') {
                    break;
                }
            }

            unary_body_accepts_empty &= unary_tail_accepts_empty;
            unary_tail_accepts_empty = true;
            match subexpr {
                ParsingResult::Alt { args, accepts_empty } => {
                    unary_args.push(ConcatArg::Star(Box::new(StarArg::Alt {
                        args,
                        accepts_empty,
                    })));
                }
                ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: true,
                } => {
                    last_concat_parens = true;
                    unary_args.push(ConcatArg::Star(Box::new(
                        StarArg::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        },
                    )))
                }
                ParsingResult::Concat {
                    mut args,
                    mut body_accepts_empty,
                    mut tail_accepts_empty,
                    parenthesized: false,
                } => {
                    last_concat_parens = false;
                    let last = args
                        .pop()
                        .expect("Concat always have at least 2 items");
                    match last {
                        ConcatArg::Alt {
                            args: last_args,
                            accepts_empty: last_accepts_empty,
                        } => {
                            tail_accepts_empty = true;
                            args.push(ConcatArg::Star(Box::new(
                                StarArg::Alt {
                                    args: last_args,
                                    accepts_empty: last_accepts_empty,
                                },
                            )));
                        }
                        ConcatArg::Star(_) => (),
                        ConcatArg::Regex { mut arg, parenthesized } => {
                            tail_accepts_empty = true;

                            if parenthesized || arg.len() == 1 {
                                args.push(ConcatArg::Star(Box::new(
                                    StarArg::Regex(arg),
                                )))
                            } else {
                                body_accepts_empty = false;

                                let last_char = arg.pop().unwrap();
                                args.push(ConcatArg::Regex {
                                    arg,
                                    parenthesized: false,
                                });
                                args.push(ConcatArg::Star(Box::new(
                                    StarArg::Regex(last_char.to_string()),
                                )));
                            }
                        }
                        ConcatArg::Concat {
                            args: c_args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        } => args.push(ConcatArg::Star(Box::new(
                            StarArg::Concat {
                                args: c_args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            },
                        ))),
                    };

                    unary_args.push(ConcatArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    })
                }
                ParsingResult::Star(arg) => {
                    unary_args.push(ConcatArg::Star(arg))
                }
                ParsingResult::Regex { mut arg, parenthesized } => {
                    if parenthesized || arg.len() == 1 {
                        unary_args.push(ConcatArg::Star(Box::new(
                            StarArg::Regex(arg),
                        )))
                    } else {
                        let last_char = arg.pop().unwrap();
                        unary_args.push(ConcatArg::Regex {
                            arg,
                            parenthesized: false,
                        });
                        unary_args.push(ConcatArg::Star(Box::new(
                            StarArg::Regex(last_char.to_string()),
                        )));
                    }
                }
            };
        }
        if unary_args.len() >= 2 {
            ParsingResult::Concat {
                args: unary_args,
                body_accepts_empty: unary_body_accepts_empty,
                tail_accepts_empty: unary_tail_accepts_empty,
                parenthesized: false,
            }
        } else if unary_args.len() == 1 {
            match unary_args.pop().unwrap() {
                ConcatArg::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: last_concat_parens,
                },
                ConcatArg::Alt { args, accepts_empty } => {
                    ParsingResult::Alt { args, accepts_empty }
                }
                ConcatArg::Star(arg) => ParsingResult::Star(arg),
                ConcatArg::Regex { arg, parenthesized } => {
                    ParsingResult::Regex { arg, parenthesized }
                }
            }
        } else {
            unreachable!();
        }
    }

    fn expect_concat(&mut self) -> ParsingResult {
        let mut main_args: Vec<ConcatArg> = vec![];
        let mut main_body_accepts_empty = true;
        let mut main_tail_accepts_empty = true;

        let mut const_regex = String::new();
        loop {
            let next_opt = self.peek();
            if next_opt.is_none() {
                break;
            }
            let next = next_opt.unwrap();
            match next {
                '(' => {
                    self.advance();
                    if !const_regex.is_empty() {
                        main_args.push(ConcatArg::Regex {
                            arg: const_regex,
                            parenthesized: false,
                        });
                        main_body_accepts_empty &= main_tail_accepts_empty;
                        main_tail_accepts_empty = false;

                        const_regex = String::new();
                    }

                    let subexpr = self.expect_alternative();
                    self.consume(')');

                    match subexpr {
                        ParsingResult::Alt { args, accepts_empty } => {
                            main_args
                                .push(ConcatArg::Alt { args, accepts_empty });
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty = accepts_empty;
                        }
                        ParsingResult::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                            parenthesized: _,
                        } => {
                            main_args.push(ConcatArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            });
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty =
                                body_accepts_empty & tail_accepts_empty;
                        }
                        ParsingResult::Star(arg) => {
                            main_args.push(ConcatArg::Star(arg));
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty = true;
                        }
                        ParsingResult::Regex { arg, parenthesized: _ } => {
                            main_args.push(ConcatArg::Regex {
                                arg,
                                parenthesized: true,
                            });
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty = true;
                        }
                    };
                }
                '|' | ')' | '*' => break,
                c => {
                    self.advance();
                    const_regex.push(c);
                }
            }
        }

        if !const_regex.is_empty() {
            main_args.push(ConcatArg::Regex {
                arg: const_regex,
                parenthesized: false,
            });
            main_body_accepts_empty &= main_tail_accepts_empty;
            main_tail_accepts_empty = false;
        }

        if main_args.len() >= 2 {
            ParsingResult::Concat {
                args: main_args,
                body_accepts_empty: main_body_accepts_empty,
                tail_accepts_empty: main_tail_accepts_empty,
                parenthesized: false,
            }
        } else if main_args.len() == 1 {
            match main_args.pop().unwrap() {
                ConcatArg::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: true,
                },
                ConcatArg::Alt { args, accepts_empty } => {
                    ParsingResult::Alt { args, accepts_empty }
                }
                ConcatArg::Star(arg) => ParsingResult::Star(arg),
                ConcatArg::Regex { arg, parenthesized } => {
                    ParsingResult::Regex { arg, parenthesized }
                }
            }
        } else {
            unreachable!();
        }
    }

    fn peek(&mut self) -> Option<char> {
        if self.next_char.is_none() {
            self.next_char = self
                .expr_iter
                .as_mut()
                .expect("expected initialized iterator")
                .next();
            if self.next_char.is_some() {
                self.index += 1;
            }
        }
        self.next_char
    }

    fn advance(&mut self) {
        if self.next_char.is_some() {
            self.next_char = None;
        } else {
            self.index += 1;
            self.expr_iter
                .as_mut()
                .expect("expected initialized iterator")
                .next();
        }
    }

    fn report(&mut self, message: &str) -> ! {
        panic!("[col {}] {}", self.index, message);
    }

    fn consume(&mut self, expected: char) {
        if let Some(next) = self.peek() {
            if next == expected {
                self.advance();
            } else {
                self.report(
                    format!("expected '{}', but '{}' found", expected, next)
                        .as_str(),
                )
            }
        } else {
            self.report(
                format!("expected '{}', but 'EOF' found", expected).as_str(),
            )
        }
    }

    fn check(&mut self, expected: char) -> bool {
        if let Some(next) = self.peek() {
            if next == expected {
                return true;
            }
        }
        false
    }

    fn at_end(&mut self) -> bool {
        self.peek().is_none()
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::Parser;
    use super::{AltArg, ConcatArg, ParsingResult, StarArg};

    #[test]
    fn basic_const_test() {
        let expr = "test_regex";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Regex {
            arg: "test_regex".to_string(),
            parenthesized: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_alternative_test() {
        let expr = "abc|cde";
        let mut parser = Parser::default();

        let res = parser.parse(expr);

        let expected = ParsingResult::Alt {
            args: vec![
                AltArg::Regex { arg: "abc".to_string(), parenthesized: false },
                AltArg::Regex { arg: "cde".to_string(), parenthesized: false },
            ],
            accepts_empty: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_concat_test() {
        let expr = "abc(cde)efg";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Regex {
                    arg: "abc".to_string(),
                    parenthesized: false,
                },
                ConcatArg::Regex {
                    arg: "cde".to_string(),
                    parenthesized: true,
                },
                ConcatArg::Regex {
                    arg: "efg".to_string(),
                    parenthesized: false,
                },
            ],
            body_accepts_empty: false,
            tail_accepts_empty: false,
            parenthesized: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_star_test() {
        let expr = "(abc)*";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected =
            ParsingResult::Star(Box::new(StarArg::Regex("abc".to_string())));

        assert_eq!(expected, res);
    }

    #[test]
    fn concat_with_star() {
        let expr = "(abc)*(cde)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Regex("abc".to_string()))),
                ConcatArg::Regex {
                    arg: "cde".to_string(),
                    parenthesized: true,
                },
            ],
            body_accepts_empty: true,
            tail_accepts_empty: false,
            parenthesized: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn star_concat() {
        let expr = "(ab)*(ed)*";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Regex("ab".to_string()))),
                ConcatArg::Star(Box::new(StarArg::Regex("ed".to_string()))),
            ],
            body_accepts_empty: true,
            tail_accepts_empty: true,
            parenthesized: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn double_star() {
        let expr = "(abc)**";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected =
            ParsingResult::Star(Box::new(StarArg::Regex("abc".to_string())));

        assert_eq!(expected, res)
    }

    #[test]
    fn non_greedy_star() {
        let expr = "(cd)qa*";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Regex { arg: "cd".to_string(), parenthesized: true },
                ConcatArg::Regex { arg: "q".to_string(), parenthesized: false },
                ConcatArg::Star(Box::new(StarArg::Regex("a".to_string()))),
            ],
            body_accepts_empty: false,
            tail_accepts_empty: true,
            parenthesized: false,
        };

        assert_eq!(expected, res)
    }

    #[test]
    fn concat_double_star() {
        let expr = "(abc)*(cde)**";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Regex("abc".to_string()))),
                ConcatArg::Star(Box::new(StarArg::Regex("cde".to_string()))),
            ],
            body_accepts_empty: true,
            tail_accepts_empty: true,
            parenthesized: false,
        };

        assert_eq!(expected, res);
    }

    #[test]
    fn trailing_concat_in_unary() {
        let expr = "a*abc(q)r";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Regex("a".to_string()))),
                ConcatArg::Regex {
                    arg: "abc".to_string(),
                    parenthesized: false,
                },
                ConcatArg::Regex { arg: "q".to_string(), parenthesized: true },
                ConcatArg::Regex { arg: "r".to_string(), parenthesized: false },
            ],
            body_accepts_empty: false,
            tail_accepts_empty: false,
            parenthesized: false,
        };

        assert_eq!(expected, res);
    }

    #[test]
    fn the_test() {
        let expr = "(abc)*((cde)|(edf))**|(qrp)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Alt {
            args: vec![
                AltArg::Concat {
                    args: vec![
                        ConcatArg::Star(Box::new(StarArg::Regex(
                            "abc".to_string(),
                        ))),
                        ConcatArg::Star(Box::new(StarArg::Alt {
                            args: vec![
                                AltArg::Regex {
                                    arg: "cde".to_string(),
                                    parenthesized: true,
                                },
                                AltArg::Regex {
                                    arg: "edf".to_string(),
                                    parenthesized: true,
                                },
                            ],
                            accepts_empty: false,
                        })),
                    ],
                    body_accepts_empty: true,
                    tail_accepts_empty: true,
                },
                AltArg::Regex { arg: "qrp".to_string(), parenthesized: true },
            ],
            accepts_empty: false,
        };

        assert_eq!(expected, res);
    }

    #[test]
    fn ssnf_on_concat() {
        let expr = "((abc)*(cde)*)*";
        let mut parser = Parser::default();

        let res = parser.parse(expr);

        let expected = ParsingResult::Star(Box::new(StarArg::Alt {
            args: vec![
                AltArg::Regex { arg: "abc".to_string(), parenthesized: true },
                AltArg::Regex { arg: "cde".to_string(), parenthesized: true },
            ],
            accepts_empty: false,
        }));
        assert_eq!(expected, res);
    }

    #[test]
    fn nested_alternative() {
        let expr = "(a|(b|(c|d)))";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Alt {
            args: vec![
                AltArg::Regex { arg: "a".to_string(), parenthesized: false },
                AltArg::Regex { arg: "b".to_string(), parenthesized: false },
                AltArg::Regex { arg: "c".to_string(), parenthesized: false },
                AltArg::Regex { arg: "d".to_string(), parenthesized: false },
            ],
            accepts_empty: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn star_simplification() {
        let expr = "((bcd)*(abc)*)**a***(((abc)*)**)***";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Alt {
                    args: vec![
                        AltArg::Regex {
                            arg: "abc".to_string(),
                            parenthesized: true,
                        },
                        AltArg::Regex {
                            arg: "bcd".to_string(),
                            parenthesized: true,
                        },
                    ],
                    accepts_empty: false,
                })),
                ConcatArg::Star(Box::new(StarArg::Regex("a".to_string()))),
                ConcatArg::Star(Box::new(StarArg::Regex("abc".to_string()))),
            ],
            body_accepts_empty: true,
            tail_accepts_empty: true,
            parenthesized: false,
        };

        // assert_eq!(expected, res);
    }
}
