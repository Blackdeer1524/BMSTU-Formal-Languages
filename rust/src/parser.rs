use std::{str::Chars, usize, vec};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AltArg {
    Concat { args: Vec<ConcatArg>, accepts_empty: bool },
    Star(Box<StarArg>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConcatArg {
    Alt { args: Vec<AltArg>, accepts_empty: bool },
    Star(Box<StarArg>),
    Char(char),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StarArg {
    Alt(Vec<AltArg>),
    Concat(Vec<ConcatArg>),
}

impl ToString for ConcatArg {
    fn to_string(&self) -> String {
        match self {
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
            ConcatArg::Char(c) => c.to_string(),
        }
    }
}

impl ToString for AltArg {
    fn to_string(&self) -> String {
        match self {
            AltArg::Concat { args, accepts_empty } => args
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(""),
            AltArg::Star(arg) => format!("({})*", arg.to_string()),
        }
    }
}

impl ToString for StarArg {
    fn to_string(&self) -> String {
        match self {
            StarArg::Concat(args) => args
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(""),
            StarArg::Alt(args) => args
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join("|"),
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
    Alt { args: Vec<AltArg>, accepts_empty: bool },
    Concat { args: Vec<ConcatArg>, accepts_empty: bool },
    Star(Box<StarArg>),
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self, regex: &'a str) -> ParsingResult {
        self.index = 0;
        self.expr_iter = Some(regex.chars());
        self.next_char = None;
        let mut res = self.expect_alternative();
        res
    }

    fn expect_alternative(&mut self) -> ParsingResult {
        let mut alt_args: Vec<AltArg> = vec![];
        let mut alt_accepts_empty = true;
        loop {
            let x = self.expect_unary();
            match x {
                ParsingResult::Alt { args, accepts_empty } => {
                    alt_accepts_empty &= accepts_empty;
                    alt_args.extend(args);
                }
                ParsingResult::Concat { args, accepts_empty } => {
                    alt_args.push(AltArg::Concat { args, accepts_empty });
                }
                ParsingResult::Star(arg) => {
                    alt_args.push(AltArg::Star(arg));
                }
            }
            if !self.check('|') {
                break;
            }
        }
        if alt_args.len() == 1 {
            let last = alt_args.pop().unwrap();
            match last {
                AltArg::Concat { args, accepts_empty } => {
                    return ParsingResult::Concat { args, accepts_empty }
                }
                AltArg::Star(arg) => return ParsingResult::Star(arg),
            }
        }
        ParsingResult::Alt { args: alt_args, accepts_empty: alt_accepts_empty }
    }

    // UNARY ::= (CONCAT "*"+)* CONCAT "*"*
    fn expect_unary(&mut self) -> ParsingResult {
        let mut unary_concat_args: Vec<ConcatArg> = vec![];
        let mut unary_concat_accepts_empty = true;
        loop {
            let x = self.expect_concat();
            if !self.check('*') {
                match x {
                    ParsingResult::Alt { args, accepts_empty } => {
                        unary_concat_accepts_empty &= accepts_empty;
                        unary_concat_args
                            .push(ConcatArg::Alt { args, accepts_empty });
                    }
                    ParsingResult::Concat { mut args, accepts_empty } => {
                        unary_concat_accepts_empty &= accepts_empty;
                        unary_concat_args.append(&mut args);
                    }
                    ParsingResult::Star(arg) => {
                        unary_concat_args.push(ConcatArg::Star(arg));
                    }
                }
                break;
            }

            self.advance();
            while self.check('*') {
                self.advance();
            }

            match x {
                ParsingResult::Alt { args, accepts_empty } => {
                    unary_concat_args
                        .push(ConcatArg::Star(Box::new(StarArg::Alt(args))));
                }
                ParsingResult::Concat { args, accepts_empty } => {
                    unary_concat_args
                        .push(ConcatArg::Star(Box::new(StarArg::Concat(args))));
                }
                ParsingResult::Star(arg) => {
                    unary_concat_args.push(ConcatArg::Star(arg));
                }
            }
            if self.check(')') || self.check('|') {
                break;
            }
        }
        if unary_concat_args.len() == 1 {
            let last = unary_concat_args.pop().unwrap();
            match last {
                ConcatArg::Alt { args, accepts_empty } => {
                    return ParsingResult::Alt { args, accepts_empty };
                }
                ConcatArg::Star(arg) => {
                    return ParsingResult::Star(arg);
                }
                ConcatArg::Char(c) => {
                    return ParsingResult::Concat {
                        args: vec![ConcatArg::Char(c)],
                        accepts_empty: false,
                    };
                }
            }
        }
        ParsingResult::Concat {
            args: unary_concat_args,
            accepts_empty: unary_concat_accepts_empty,
        }
    }

    fn expect_concat(&mut self) -> ParsingResult {
        let mut concat_args: Vec<ConcatArg> = vec![];
        let mut concat_accepts_empty = true;
        loop {
            let next_opt = self.peek();
            if next_opt.is_none() {
                break;
            }
            let next = next_opt.unwrap();
            match next {
                ')' | '|' | '*' => break,
                '(' => {
                    self.advance();
                    let inner = self.expect_alternative();
                    self.consume(')');
                    match inner {
                        ParsingResult::Alt { args, accepts_empty } => {
                            concat_accepts_empty &= accepts_empty;
                            concat_args
                                .push(ConcatArg::Alt { args, accepts_empty })
                        }
                        ParsingResult::Concat { args, accepts_empty } => {
                            concat_accepts_empty &= accepts_empty;
                            concat_args.extend(args.into_iter());
                        }
                        ParsingResult::Star(arg) => {
                            concat_args.push(ConcatArg::Star(arg));
                        }
                    }
                }
                c => {
                    concat_args.push(ConcatArg::Char(c));
                    concat_accepts_empty = false;
                    self.advance();
                }
            }
        }
        if concat_args.len() == 1 {
            let last = concat_args.pop().unwrap();
            match last {
                ConcatArg::Alt { args, accepts_empty } => {
                    return ParsingResult::Alt { args, accepts_empty }
                }
                ConcatArg::Star(arg) => {
                    return ParsingResult::Star(arg);
                }
                ConcatArg::Char(_) => (),
            }
        }
        ParsingResult::Concat {
            args: concat_args,
            accepts_empty: concat_accepts_empty,
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
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Char('t'),
                ConcatArg::Char('e'),
                ConcatArg::Char('s'),
                ConcatArg::Char('t'),
                ConcatArg::Char('_'),
                ConcatArg::Char('r'),
                ConcatArg::Char('e'),
                ConcatArg::Char('g'),
                ConcatArg::Char('e'),
                ConcatArg::Char('x'),
            ],
            accepts_empty: false,
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
                AltArg::Concat {
                    args: vec![
                        ConcatArg::Char('a'),
                        ConcatArg::Char('b'),
                        ConcatArg::Char('c'),
                    ],
                    accepts_empty: false,
                },
                AltArg::Concat {
                    args: vec![
                        ConcatArg::Char('c'),
                        ConcatArg::Char('d'),
                        ConcatArg::Char('e'),
                    ],
                    accepts_empty: false,
                },
            ],
            accepts_empty: false,
        };
        assert_eq!(expected, res);
    }

    // #[test]
    // fn basic_concat_test() {
    //     let expr = "abc(cde)efg";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Regex {
    //                 arg: "abc".to_string(),
    //                 parenthesized: false,
    //             },
    //             ConcatArg::Regex {
    //                 arg: "cde".to_string(),
    //                 parenthesized: true,
    //             },
    //             ConcatArg::Regex {
    //                 arg: "efg".to_string(),
    //                 parenthesized: false,
    //             },
    //         ],
    //         body_accepts_empty: false,
    //         tail_accepts_empty: false,
    //         parenthesized: false,
    //     };
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn basic_star_test() {
    //     let expr = "(abc)*";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected =
    //         ParsingResult::Star(Box::new(StarArg::Regex("abc".to_string())));
    //
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn concat_with_star() {
    //     let expr = "(abc)*(cde)";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Star(Box::new(StarArg::Regex("abc".to_string()))),
    //             ConcatArg::Regex {
    //                 arg: "cde".to_string(),
    //                 parenthesized: true,
    //             },
    //         ],
    //         body_accepts_empty: true,
    //         tail_accepts_empty: false,
    //         parenthesized: false,
    //     };
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn star_concat() {
    //     let expr = "(ab)*(ed)*";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Star(Box::new(StarArg::Regex("ab".to_string()))),
    //             ConcatArg::Star(Box::new(StarArg::Regex("ed".to_string()))),
    //         ],
    //         body_accepts_empty: true,
    //         tail_accepts_empty: true,
    //         parenthesized: false,
    //     };
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn double_star() {
    //     let expr = "(abc)**";
    //     let mut parser = Parser::default();
    //     let res = parser.parse(expr);
    //
    //     let expected =
    //         ParsingResult::Star(Box::new(StarArg::Regex("abc".to_string())));
    //
    //     assert_eq!(expected, res)
    // }
    //
    // #[test]
    // fn non_greedy_star() {
    //     let expr = "(cd)qa*";
    //     let mut parser = Parser::default();
    //     let res = parser.parse(expr);
    //
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Regex { arg: "cd".to_string(), parenthesized: true },
    //             ConcatArg::Regex { arg: "q".to_string(), parenthesized: false },
    //             ConcatArg::Star(Box::new(StarArg::Regex("a".to_string()))),
    //         ],
    //         body_accepts_empty: false,
    //         tail_accepts_empty: true,
    //         parenthesized: false,
    //     };
    //
    //     assert_eq!(expected, res)
    // }
    //
    // #[test]
    // fn concat_double_star() {
    //     let expr = "(abc)*(cde)**";
    //     let mut parser = Parser::default();
    //     let res = parser.parse(expr);
    //
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Star(Box::new(StarArg::Regex("abc".to_string()))),
    //             ConcatArg::Star(Box::new(StarArg::Regex("cde".to_string()))),
    //         ],
    //         body_accepts_empty: true,
    //         tail_accepts_empty: true,
    //         parenthesized: false,
    //     };
    //
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn trailing_concat_in_unary() {
    //     let expr = "a*abc(q)r";
    //     let mut parser = Parser::default();
    //     let res = parser.parse(expr);
    //
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Star(Box::new(StarArg::Regex("a".to_string()))),
    //             ConcatArg::Regex {
    //                 arg: "abc".to_string(),
    //                 parenthesized: false,
    //             },
    //             ConcatArg::Regex { arg: "q".to_string(), parenthesized: true },
    //             ConcatArg::Regex { arg: "r".to_string(), parenthesized: false },
    //         ],
    //         body_accepts_empty: false,
    //         tail_accepts_empty: false,
    //         parenthesized: false,
    //     };
    //
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn the_test() {
    //     let expr = "(abc)*((cde)|(edf))**|(qrp)";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = ParsingResult::Alt {
    //         args: vec![
    //             AltArg::Concat {
    //                 args: vec![
    //                     ConcatArg::Star(Box::new(StarArg::Regex(
    //                         "abc".to_string(),
    //                     ))),
    //                     ConcatArg::Star(Box::new(StarArg::Alt {
    //                         args: vec![
    //                             AltArg::Regex {
    //                                 arg: "cde".to_string(),
    //                                 parenthesized: true,
    //                             },
    //                             AltArg::Regex {
    //                                 arg: "edf".to_string(),
    //                                 parenthesized: true,
    //                             },
    //                         ],
    //                         accepts_empty: false,
    //                     })),
    //                 ],
    //                 body_accepts_empty: true,
    //                 tail_accepts_empty: true,
    //             },
    //             AltArg::Regex { arg: "qrp".to_string(), parenthesized: true },
    //         ],
    //         accepts_empty: false,
    //     };
    //
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn ssnf_on_concat() {
    //     let expr = "((abc)*(cde)*)*";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //
    //     let expected = ParsingResult::Star(Box::new(StarArg::Alt {
    //         args: vec![
    //             AltArg::Regex { arg: "abc".to_string(), parenthesized: true },
    //             AltArg::Regex { arg: "cde".to_string(), parenthesized: true },
    //         ],
    //         accepts_empty: false,
    //     }));
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn nested_alternative() {
    //     let expr = "(a|(b|(c|d)))";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = ParsingResult::Alt {
    //         args: vec![
    //             AltArg::Regex { arg: "a".to_string(), parenthesized: false },
    //             AltArg::Regex { arg: "b".to_string(), parenthesized: false },
    //             AltArg::Regex { arg: "c".to_string(), parenthesized: false },
    //             AltArg::Regex { arg: "d".to_string(), parenthesized: false },
    //         ],
    //         accepts_empty: false,
    //     };
    //     assert_eq!(expected, res);
    // }
    //
    // #[test]
    // fn star_simplification() {
    //     let expr = "((bcd)*(abc)*)**a***(((abc)*)**)***";
    //     let mut parser = Parser::default();
    //
    //     let res = parser.parse(expr);
    //     let expected = ParsingResult::Concat {
    //         args: vec![
    //             ConcatArg::Star(Box::new(StarArg::Alt {
    //                 args: vec![
    //                     AltArg::Regex {
    //                         arg: "abc".to_string(),
    //                         parenthesized: true,
    //                     },
    //                     AltArg::Regex {
    //                         arg: "bcd".to_string(),
    //                         parenthesized: true,
    //                     },
    //                 ],
    //                 accepts_empty: false,
    //             })),
    //             ConcatArg::Star(Box::new(StarArg::Regex("a".to_string()))),
    //             ConcatArg::Star(Box::new(StarArg::Regex("abc".to_string()))),
    //         ],
    //         body_accepts_empty: true,
    //         tail_accepts_empty: true,
    //         parenthesized: false,
    //     };
    //
    //     assert_eq!(expected, res);
    // }
}
