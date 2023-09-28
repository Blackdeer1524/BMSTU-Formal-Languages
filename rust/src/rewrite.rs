use std::{str::Chars, usize, vec};

#[derive(Debug, PartialEq, Eq)]
enum AltArgs {
    Concat {
        // Нам всё равно в какой форме конкат: в скобках или нет
        args: Vec<ConcatArgs>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
    },
    Star(Box<StarArg>),
    Regex {
        arg: String,
        parenthesized: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum ConcatArgs {
    Concat {
        // конкат внутри конката может быть ТОЛЬКО в скобках
        args: Vec<ConcatArgs>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
    },
    Alt {
        args: Vec<AltArgs>,
        accepts_empty: bool,
    },
    Star(Box<StarArg>),
    Regex {
        arg: String,
        parenthesized: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum StarArg {
    Alt {
        args: Vec<AltArgs>,
        accepts_empty: bool,
    },
    Concat {
        // конкат внутри звезды может быть только в скобках
        args: Vec<ConcatArgs>,
        body_accepts_empty: bool,
        tail_accepts_empty: bool,
    },
    Regex(String),
}

#[derive(Default)]
struct Parser<'a> {
    index: usize,
    expr_iter: Option<Chars<'a>>,
    next_char: Option<char>,
}

#[derive(Debug, PartialEq, Eq)]
enum ParsingResult {
    Alt {
        args: Vec<AltArgs>,
        accepts_empty: bool,
    },
    Concat {
        args: Vec<ConcatArgs>,
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
        self.expect_alternative()
    }

    fn expect_alternative(&mut self) -> ParsingResult {
        let mut alt_args: Vec<AltArgs> = vec![];
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
                    alt_args.push(AltArgs::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    });
                    alt_accepts_empty &=
                        body_accepts_empty & tail_accepts_empty;
                }
                ParsingResult::Star(arg) => {
                    alt_args.push(AltArgs::Star(arg));
                }
                ParsingResult::Regex { arg, parenthesized } => {
                    alt_args.push(AltArgs::Regex { arg, parenthesized });
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
                AltArgs::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: last_concat_parens,
                },
                AltArgs::Star(arg) => ParsingResult::Star(arg),
                AltArgs::Regex { arg, parenthesized } => {
                    ParsingResult::Regex { arg, parenthesized }
                }
            }
        } else {
            unreachable!();
        }
    }

    // UNARY ::= (CONCAT "*"+)* CONCAT
    fn expect_unary(&mut self) -> ParsingResult {
        let mut unary_args: Vec<ConcatArgs> = vec![];
        let mut unary_body_accepts_empty: bool = true;
        let mut unary_tail_accepts_empty: bool = true;
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
                        unary_args
                            .push(ConcatArgs::Alt { args, accepts_empty });
                    }
                    ParsingResult::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                        parenthesized,
                    } => {
                        if parenthesized {
                            unary_body_accepts_empty &=
                                unary_tail_accepts_empty;
                            unary_tail_accepts_empty =
                                body_accepts_empty & tail_accepts_empty;
                            unary_args.push(ConcatArgs::Concat {
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
                        unary_args.push(ConcatArgs::Star(arg))
                    }
                    ParsingResult::Regex { arg, parenthesized } => {
                        unary_body_accepts_empty &= unary_tail_accepts_empty;
                        unary_tail_accepts_empty = false;
                        unary_args
                            .push(ConcatArgs::Regex { arg, parenthesized });
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
                    unary_args.push(ConcatArgs::Star(Box::new(StarArg::Alt {
                        args,
                        accepts_empty,
                    })));
                }
                ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: true,
                } => unary_args.push(ConcatArgs::Star(Box::new(
                    StarArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    },
                ))),
                ParsingResult::Concat {
                    mut args,
                    mut body_accepts_empty,
                    mut tail_accepts_empty,
                    parenthesized: false,
                } => {
                    let last = args
                        .pop()
                        .expect("Concat always have at least 2 items");
                    match last {
                        ConcatArgs::Alt {
                            args: last_args,
                            accepts_empty: last_accepts_empty,
                        } => {
                            tail_accepts_empty = true;
                            args.push(ConcatArgs::Star(Box::new(
                                StarArg::Alt {
                                    args: last_args,
                                    accepts_empty: last_accepts_empty,
                                },
                            )));
                        }
                        ConcatArgs::Star(_) => (),
                        ConcatArgs::Regex { mut arg, parenthesized } => {
                            tail_accepts_empty = true;

                            if parenthesized || arg.len() == 1 {
                                args.push(ConcatArgs::Star(Box::new(
                                    StarArg::Regex(arg),
                                )))
                            } else {
                                body_accepts_empty = false;

                                let last_char = arg.pop().unwrap();
                                args.push(ConcatArgs::Regex {
                                    arg,
                                    parenthesized: false,
                                });
                                args.push(ConcatArgs::Star(Box::new(
                                    StarArg::Regex(last_char.to_string()),
                                )));
                            }
                        }
                        ConcatArgs::Concat {
                            args: c_args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        } => args.push(ConcatArgs::Star(Box::new(
                            StarArg::Concat {
                                args: c_args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            },
                        ))),
                    };

                    unary_args.push(ConcatArgs::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    })
                }
                ParsingResult::Star(arg) => {
                    unary_args.push(ConcatArgs::Star(arg))
                }
                ParsingResult::Regex { mut arg, parenthesized } => {
                    if parenthesized || arg.len() == 1 {
                        unary_args.push(ConcatArgs::Star(Box::new(
                            StarArg::Regex(arg),
                        )))
                    } else {
                        let last_char = arg.pop().unwrap();
                        unary_args.push(ConcatArgs::Regex {
                            arg,
                            parenthesized: false,
                        });
                        unary_args.push(ConcatArgs::Star(Box::new(
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
                ConcatArgs::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: false,
                },
                ConcatArgs::Alt { args, accepts_empty } => {
                    ParsingResult::Alt { args, accepts_empty }
                }
                ConcatArgs::Star(arg) => ParsingResult::Star(arg),
                ConcatArgs::Regex { arg, parenthesized } => {
                    ParsingResult::Regex { arg, parenthesized }
                }
            }
        } else {
            unreachable!();
        }
    }

    fn expect_concat(&mut self) -> ParsingResult {
        let mut main_args: Vec<ConcatArgs> = vec![];
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
                        main_args.push(ConcatArgs::Regex {
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
                                .push(ConcatArgs::Alt { args, accepts_empty });
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty = accepts_empty;
                        }
                        ParsingResult::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                            parenthesized: _,
                        } => {
                            main_args.push(ConcatArgs::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            });
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty =
                                body_accepts_empty & tail_accepts_empty;
                        }
                        ParsingResult::Star(arg) => {
                            main_args.push(ConcatArgs::Star(arg));
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty = true;
                        }
                        ParsingResult::Regex { arg, parenthesized: _ } => {
                            main_args.push(ConcatArgs::Regex {
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
            main_args.push(ConcatArgs::Regex {
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
                ConcatArgs::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: false,
                },
                ConcatArgs::Alt { args, accepts_empty } => {
                    ParsingResult::Alt { args, accepts_empty }
                }
                ConcatArgs::Star(arg) => ParsingResult::Star(arg),
                ConcatArgs::Regex { arg, parenthesized } => {
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

    use crate::rewrite::{AltArgs, ConcatArgs, ParsingResult, StarArg};

    use super::Parser;

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
                AltArgs::Regex { arg: "abc".to_string(), parenthesized: false },
                AltArgs::Regex { arg: "cde".to_string(), parenthesized: false },
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
                ConcatArgs::Regex {
                    arg: "abc".to_string(),
                    parenthesized: false,
                },
                ConcatArgs::Regex {
                    arg: "cde".to_string(),
                    parenthesized: true,
                },
                ConcatArgs::Regex {
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
                ConcatArgs::Star(Box::new(StarArg::Regex("abc".to_string()))),
                ConcatArgs::Regex {
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
                ConcatArgs::Star(Box::new(StarArg::Regex("ab".to_string()))),
                ConcatArgs::Star(Box::new(StarArg::Regex("ed".to_string()))),
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
                ConcatArgs::Regex {
                    arg: "cd".to_string(),
                    parenthesized: true,
                },
                ConcatArgs::Regex {
                    arg: "q".to_string(),
                    parenthesized: false,
                },
                ConcatArgs::Star(Box::new(StarArg::Regex("a".to_string()))),
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
                ConcatArgs::Star(Box::new(StarArg::Regex("abc".to_string()))),
                ConcatArgs::Star(Box::new(StarArg::Regex("cde".to_string()))),
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
                ConcatArgs::Star(Box::new(StarArg::Regex("a".to_string()))),
                ConcatArgs::Regex {
                    arg: "abc".to_string(),
                    parenthesized: false,
                },
                ConcatArgs::Regex { arg: "q".to_string(), parenthesized: true },
                ConcatArgs::Regex {
                    arg: "r".to_string(),
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
    fn the_test() {
        let expr = "(abc)*((cde)|(edf))**|(qrp)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Alt {
            args: vec![
                AltArgs::Concat {
                    args: vec![
                        ConcatArgs::Star(Box::new(StarArg::Regex(
                            "abc".to_string(),
                        ))),
                        ConcatArgs::Star(Box::new(StarArg::Alt {
                            args: vec![
                                AltArgs::Regex {
                                    arg: "cde".to_string(),
                                    parenthesized: true,
                                },
                                AltArgs::Regex {
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
                AltArgs::Regex { arg: "qrp".to_string(), parenthesized: true },
            ],
            accepts_empty: false,
        };

        assert_eq!(expected, res);
    }

    // #[test]
    // fn star_simplification() {
    //     let expr = "((abc)*|(bcd)*)**a***(((abc)*)**)***";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = OperationArg::Operation(Operation::Concat {
    //         args: vec![
    //             OperationArg::Operation(Operation::Star(Box::new(
    //                 OperationArg::Operation(Operation::Alternative {
    //                     args: vec![
    //                         OperationArg::Const {
    //                             expr: "abc".to_string(),
    //                             parenthesized: true,
    //                         },
    //                         OperationArg::Const {
    //                             expr: "bcd".to_string(),
    //                             parenthesized: true,
    //                         },
    //                     ],
    //                     accepts_empty: true,
    //                 }),
    //             ))),
    //             OperationArg::Operation(Operation::Star(Box::new(
    //                 OperationArg::Const {
    //                     expr: "a".to_string(),
    //                     parenthesized: false,
    //                 },
    //             ))),
    //             OperationArg::Operation(Operation::Star(Box::new(
    //                 OperationArg::Const {
    //                     expr: "abc".to_string(),
    //                     parenthesized: true,
    //                 },
    //             ))),
    //         ],
    //         accepts_empty: true,
    //     });
    //     assert_eq!(expected, res);
    // }
}
