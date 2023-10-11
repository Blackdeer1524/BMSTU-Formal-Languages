use std::{collections::LinkedList, str::Chars, usize, vec};

use super::aci::simplify;
use crate::ssnf::ssnf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AltArg {
    Concat { args: Vec<ConcatArg>, accepts_empty: bool },
    Star(Box<StarArg>),
}

impl ToString for AltArg {
    fn to_string(&self) -> String {
        match self {
            AltArg::Concat { args, accepts_empty } => {
                args.iter().map(|item| item.to_string()).collect::<Vec<String>>().join("")
            }
            AltArg::Star(arg) => format!("({})*", arg.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConcatArg {
    Alt { args: Vec<AltArg>, accepts_empty: bool },
    Star(Box<StarArg>),
    Char(char),
}

impl ToString for ConcatArg {
    fn to_string(&self) -> String {
        match self {
            ConcatArg::Alt { args, accepts_empty } => {
                format!(
                    "({})",
                    args.iter().map(|item| { item.to_string() }).collect::<Vec<String>>().join("|")
                )
            }
            ConcatArg::Star(arg) => format!("({})*", arg.to_string()),
            ConcatArg::Char(c) => c.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StarArg {
    Alt { args: Vec<AltArg>, accepts_empty: bool },
    Concat { args: Vec<ConcatArg>, accepts_empty: bool },
}

impl ToString for StarArg {
    fn to_string(&self) -> String {
        match self {
            StarArg::Concat { args, accepts_empty } => {
                args.iter().map(|item| item.to_string()).collect::<Vec<String>>().join("")
            }
            StarArg::Alt { args, accepts_empty } => {
                args.iter().map(|item| item.to_string()).collect::<Vec<String>>().join("|")
            }
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
    Alt { args: LinkedList<AltArg>, accepts_empty: bool },
    Concat { args: Vec<ConcatArg>, accepts_empty: bool },
    Star(Box<StarArg>),
}

impl From<AltArg> for ParsingResult {
    fn from(value: AltArg) -> Self {
        match value {
            AltArg::Concat { args, accepts_empty } => ParsingResult::Concat { args, accepts_empty },
            AltArg::Star(arg) => ParsingResult::Star(arg),
        }
    }
}

impl From<ConcatArg> for ParsingResult {
    fn from(value: ConcatArg) -> Self {
        match value {
            ConcatArg::Alt { args, accepts_empty } => {
                ParsingResult::Alt { args: LinkedList::from_iter(args), accepts_empty }
            }
            ConcatArg::Star(arg) => ParsingResult::Star(arg),
            ConcatArg::Char(c) => {
                ParsingResult::Concat { args: vec![ConcatArg::Char(c)], accepts_empty: false }
            }
        }
    }
}

impl From<StarArg> for ParsingResult {
    fn from(value: StarArg) -> Self {
        match value {
            StarArg::Alt { args, accepts_empty } => {
                ParsingResult::Alt { args: LinkedList::from_iter(args), accepts_empty }
            }
            StarArg::Concat { args, accepts_empty } => {
                ParsingResult::Concat { args, accepts_empty }
            }
        }
    }
}

impl From<ParsingResult> for ConcatArg {
    fn from(value: ParsingResult) -> Self {
        match value {
            ParsingResult::Alt { args, accepts_empty } => {
                ConcatArg::Alt { args: Vec::from_iter(args), accepts_empty }
            }
            ParsingResult::Concat { args, accepts_empty } => {
                unreachable!("concat cannot have concat as a child");
            }
            ParsingResult::Star(arg) => ConcatArg::Star(arg),
        }
    }
}

impl From<ParsingResult> for AltArg {
    fn from(value: ParsingResult) -> Self {
        match value {
            ParsingResult::Alt { args, accepts_empty } => {
                unreachable!("alternative cannot have alternative as a child")
            }
            ParsingResult::Concat { args, accepts_empty } => AltArg::Concat { args, accepts_empty },
            ParsingResult::Star(arg) => AltArg::Star(arg),
        }
    }
}

impl From<ParsingResult> for StarArg {
    fn from(value: ParsingResult) -> Self {
        match value {
            ParsingResult::Alt { args, accepts_empty } => {
                StarArg::Alt { args: Vec::from_iter(args), accepts_empty }
            }
            ParsingResult::Concat { args, accepts_empty } => {
                StarArg::Concat { args, accepts_empty }
            }
            ParsingResult::Star(arg) => {
                unreachable!("star cannot have star as a child")
            }
        }
    }
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self, regex: &'a str) -> ParsingResult {
        self.index = 0;
        self.expr_iter = Some(regex.chars());
        self.next_char = None;
        let res = self.expect_alternative();
        simplify(ssnf(res))
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
                    alt_accepts_empty &= accepts_empty;
                    alt_args.push(AltArg::Concat { args, accepts_empty });
                }
                ParsingResult::Star(arg) => {
                    alt_args.push(AltArg::Star(arg));
                }
            }
            if !self.check('|') {
                break;
            }
            self.advance();
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
        ParsingResult::Alt {
            args: LinkedList::from_iter(alt_args),
            accepts_empty: alt_accepts_empty,
        }
    }

    fn expect_unary(&mut self) -> ParsingResult {
        let mut unary_concat_args: Vec<ConcatArg> = vec![];
        let mut unary_concat_accepts_empty = true;

        let mut last_concat_portion: Option<ParsingResult> = None;
        'mainloop: loop {
            loop {
                let next_opt = self.peek();
                if next_opt.is_none() || next_opt.unwrap() != '*' {
                    if let Some(last_concat) = last_concat_portion {
                        match last_concat {
                            ParsingResult::Alt { args, accepts_empty } => {
                                unary_concat_accepts_empty &= accepts_empty;
                                unary_concat_args.push(ConcatArg::Alt {
                                    args: Vec::from_iter(args),
                                    accepts_empty,
                                });
                            }
                            ParsingResult::Concat { args, accepts_empty } => {
                                unary_concat_accepts_empty &= accepts_empty;
                                unary_concat_args.extend(args);
                            }
                            ParsingResult::Star(arg) => {
                                unary_concat_args.push(ConcatArg::Star(arg));
                            }
                        }
                        last_concat_portion = None;
                    }
                    if next_opt.is_none() {
                        break 'mainloop;
                    }
                }
                let next = next_opt.unwrap();
                match next {
                    '*' => break,
                    '(' => {
                        self.advance();
                        let portion = self.expect_alternative();
                        self.consume(')');
                        last_concat_portion = Some(portion);
                    }
                    ')' | '|' => {
                        break 'mainloop;
                    }
                    c => {
                        last_concat_portion = Some(ParsingResult::Concat {
                            args: vec![ConcatArg::Char(c)],
                            accepts_empty: false,
                        });
                        self.advance();
                    }
                }
            }

            self.advance();
            while self.check('*') {
                self.advance();
            }

            if last_concat_portion.is_none() {
                self.report("expected something before `*`");
            }
            let last_concat = last_concat_portion.unwrap();
            last_concat_portion = None;

            match last_concat {
                ParsingResult::Alt { args, accepts_empty } => {
                    unary_concat_args.push(ConcatArg::Star(Box::new(StarArg::Alt {
                        args: Vec::from_iter(args),
                        accepts_empty,
                    })));
                }
                ParsingResult::Concat { args, accepts_empty } => {
                    unary_concat_args
                        .push(ConcatArg::Star(Box::new(StarArg::Concat { args, accepts_empty })));
                }
                ParsingResult::Star(arg) => {
                    unary_concat_args.push(ConcatArg::Star(arg));
                }
            }
        }
        if unary_concat_args.len() == 1 {
            let last = unary_concat_args.pop().unwrap();
            match last {
                ConcatArg::Alt { args, accepts_empty } => {
                    return ParsingResult::Alt { args: LinkedList::from_iter(args), accepts_empty };
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
        ParsingResult::Concat { args: unary_concat_args, accepts_empty: unary_concat_accepts_empty }
    }

    fn peek(&mut self) -> Option<char> {
        if self.next_char.is_none() {
            self.next_char = self.expr_iter.as_mut().expect("expected initialized iterator").next();
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
            self.expr_iter.as_mut().expect("expected initialized iterator").next();
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
                self.report(format!("expected '{}', but '{}' found", expected, next).as_str())
            }
        } else {
            self.report(format!("expected '{}', but 'EOF' found", expected).as_str())
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
    use std::collections::{vec_deque, LinkedList};
    use std::vec;

    use super::Parser;
    use super::{AltArg, ConcatArg, ParsingResult, StarArg};

    #[test]
    fn empty() {
        let expr = "";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat { args: vec![], accepts_empty: true };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_const_test() {
        let expr = "abc";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
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
            args: LinkedList::from([
                AltArg::Concat {
                    args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
                    accepts_empty: false,
                },
                AltArg::Concat {
                    args: vec![ConcatArg::Char('c'), ConcatArg::Char('d'), ConcatArg::Char('e')],
                    accepts_empty: false,
                },
            ]),
            accepts_empty: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_concat_test() {
        let expr = "ab(cd)ef";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Char('a'),
                ConcatArg::Char('b'),
                ConcatArg::Char('c'),
                ConcatArg::Char('d'),
                ConcatArg::Char('e'),
                ConcatArg::Char('f'),
            ],
            accepts_empty: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_star_test() {
        let expr = "(abc)*";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Star(Box::new(StarArg::Concat {
            args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
            accepts_empty: false,
        }));

        assert_eq!(expected, res);
    }

    #[test]
    fn concat_with_star() {
        let expr = "(abc)*(cde)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
                    accepts_empty: false,
                })),
                ConcatArg::Char('c'),
                ConcatArg::Char('d'),
                ConcatArg::Char('e'),
            ],
            accepts_empty: false,
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
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a'), ConcatArg::Char('b')],
                    accepts_empty: false,
                })),
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('e'), ConcatArg::Char('d')],
                    accepts_empty: false,
                })),
            ],
            accepts_empty: true,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn double_star() {
        let expr = "(abc)**";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = ParsingResult::Star(Box::new(StarArg::Concat {
            args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
            accepts_empty: false,
        }));

        assert_eq!(expected, res)
    }

    #[test]
    fn non_greedy_star() {
        let expr = "(cd)qa*";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Char('c'),
                ConcatArg::Char('d'),
                ConcatArg::Char('q'),
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a')],
                    accepts_empty: false,
                })),
            ],
            accepts_empty: false,
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
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
                    accepts_empty: false,
                })),
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('c'), ConcatArg::Char('d'), ConcatArg::Char('e')],
                    accepts_empty: false,
                })),
            ],
            accepts_empty: true,
        };

        assert_eq!(expected, res);
    }

    #[test]
    fn nested_alternative() {
        let expr = "(a|(b|(c|d)))";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Alt {
            args: LinkedList::from([
                AltArg::Concat { args: vec![ConcatArg::Char('a')], accepts_empty: false },
                AltArg::Concat { args: vec![ConcatArg::Char('b')], accepts_empty: false },
                AltArg::Concat { args: vec![ConcatArg::Char('c')], accepts_empty: false },
                AltArg::Concat { args: vec![ConcatArg::Char('d')], accepts_empty: false },
            ]),
            accepts_empty: false,
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
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a')],
                    accepts_empty: false,
                })),
                ConcatArg::Char('a'),
                ConcatArg::Char('b'),
                ConcatArg::Char('c'),
                ConcatArg::Char('q'),
                ConcatArg::Char('r'),
            ],
            accepts_empty: false,
        };

        assert_eq!(expected, res);
    }

    #[test]
    fn the_test() {
        let expr = "(abc)*((cde)|(edf))**|(qrp)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Alt {
            args: LinkedList::from([
                AltArg::Concat {
                    args: vec![
                        ConcatArg::Star(Box::new(StarArg::Concat {
                            args: vec![
                                ConcatArg::Char('a'),
                                ConcatArg::Char('b'),
                                ConcatArg::Char('c'),
                            ],
                            accepts_empty: false,
                        })),
                        ConcatArg::Star(Box::new(StarArg::Alt {
                            args: vec![
                                AltArg::Concat {
                                    args: vec![
                                        ConcatArg::Char('c'),
                                        ConcatArg::Char('d'),
                                        ConcatArg::Char('e'),
                                    ],
                                    accepts_empty: false,
                                },
                                AltArg::Concat {
                                    args: vec![
                                        ConcatArg::Char('e'),
                                        ConcatArg::Char('d'),
                                        ConcatArg::Char('f'),
                                    ],
                                    accepts_empty: false,
                                },
                            ],
                            accepts_empty: false,
                        })),
                    ],
                    accepts_empty: true,
                },
                AltArg::Concat {
                    args: vec![ConcatArg::Char('q'), ConcatArg::Char('r'), ConcatArg::Char('p')],
                    accepts_empty: false,
                },
            ]),
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
                AltArg::Concat {
                    args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
                    accepts_empty: false,
                },
                AltArg::Concat {
                    args: vec![ConcatArg::Char('c'), ConcatArg::Char('d'), ConcatArg::Char('e')],
                    accepts_empty: false,
                },
            ],
            accepts_empty: false,
        }));
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_simplification() {
        let expr = "(aqb|arb|ab)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Char('a'),
                ConcatArg::Alt {
                    args: vec![
                        AltArg::Concat { args: vec![], accepts_empty: true },
                        AltArg::Concat { args: vec![ConcatArg::Char('q')], accepts_empty: false },
                        AltArg::Concat { args: vec![ConcatArg::Char('r')], accepts_empty: false },
                    ],
                    accepts_empty: true,
                },
                ConcatArg::Char('b'),
            ],
            accepts_empty: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn star_in_alternative() {
        let expr = "(a*|(a)*)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = ParsingResult::Star(Box::new(StarArg::Concat {
            args: vec![ConcatArg::Char('a')],
            accepts_empty: false,
        }));
        assert_eq!(expected, res);
    }

    #[test]
    fn full_simplification() {
        let expr = "(a|a)";
        let mut parser = Parser::default();
        let res = parser.parse(expr);
        let expected =
            ParsingResult::Concat { args: vec![ConcatArg::Char('a')], accepts_empty: false };
        assert_eq!(expected, res);
    }

    #[test]
    fn hard_simplification() {
        let expr = "(a)*r(c|d)|a*q(d|c)";
        let mut parser = Parser::default();
        let res = parser.parse(expr);
        let expected = ParsingResult::Concat {
            args: vec![
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a')],
                    accepts_empty: false,
                })),
                ConcatArg::Alt {
                    args: vec![
                        AltArg::Concat { args: vec![ConcatArg::Char('q')], accepts_empty: false },
                        AltArg::Concat { args: vec![ConcatArg::Char('r')], accepts_empty: false },
                    ],
                    accepts_empty: false,
                },
                ConcatArg::Alt {
                    args: vec![
                        AltArg::Concat { args: vec![ConcatArg::Char('c')], accepts_empty: false },
                        AltArg::Concat { args: vec![ConcatArg::Char('d')], accepts_empty: false },
                    ],
                    accepts_empty: false,
                },
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
                                ConcatArg::Char('b'),
                                ConcatArg::Char('c'),
                                ConcatArg::Char('d'),
                            ],
                            accepts_empty: false,
                        },
                    ],
                    accepts_empty: false,
                })),
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a')],
                    accepts_empty: false,
                })),
                ConcatArg::Star(Box::new(StarArg::Concat {
                    args: vec![ConcatArg::Char('a'), ConcatArg::Char('b'), ConcatArg::Char('c')],
                    accepts_empty: false,
                })),
            ],
            accepts_empty: true,
        };

        assert_eq!(expected, res);
    }
}
