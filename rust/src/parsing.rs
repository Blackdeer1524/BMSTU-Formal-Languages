use std::{ops::Deref, str::Chars, vec};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Operation {
    Concat(Vec<OperationArg>),
    Alternative(Vec<OperationArg>),
    Star(Box<OperationArg>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OperationArg {
    Const { expr: String, parenthesized: bool },
    Operation(Operation),
}

#[derive(Default)]
struct Parser<'a> {
    regex_iter: Option<Chars<'a>>,
    next_char: Option<char>,
    index: usize,
}

fn concat_const(
    node_opt: &mut Option<OperationArg>,
    cur_regex: &mut String,
) -> bool {
    if !cur_regex.is_empty() {
        if node_opt.is_some() {
            let node_variant = node_opt.as_mut().unwrap();
            match node_variant {
                OperationArg::Const { .. } => {
                    // TODO: здесь можно упростить
                    *node_variant =
                        OperationArg::Operation(Operation::Concat(vec![
                            node_variant.deref().clone(),
                            OperationArg::Const {
                                expr: cur_regex.clone(),
                                parenthesized: false,
                            },
                        ]))
                }
                OperationArg::Operation(operation) => match operation {
                    Operation::Concat(args) => {
                        args.push(OperationArg::Const {
                            expr: cur_regex.clone(),
                            parenthesized: false,
                        });
                    }
                    Operation::Alternative(_) | Operation::Star(_) => {
                        *node_variant =
                            OperationArg::Operation(Operation::Concat(vec![
                                node_variant.deref().clone(),
                                OperationArg::Const {
                                    expr: cur_regex.clone(),
                                    parenthesized: false,
                                },
                            ]))
                    }
                },
            }
        } else {
            *node_opt = Some(OperationArg::Const {
                expr: cur_regex.clone(),
                parenthesized: false,
            });
        }
        *cur_regex = String::new();
        true
    } else {
        false
    }
}

fn propogate_star(outer: &mut OperationArg) {
    match outer {
        OperationArg::Const { .. } => (),
        OperationArg::Operation(op) => match op {
            Operation::Concat(_) => (),
            Operation::Alternative(args) => {
                args.iter_mut().for_each(propogate_star)
            }
            Operation::Star(arg) => {
                propogate_star(arg);
                *outer = arg.deref().deref().clone();
            }
        },
    }
}

impl<'a> Parser<'a> {
    fn parse(&mut self, regex: &'a str) -> OperationArg {
        self.index = 0;
        self.next_char = None;
        self.regex_iter = Some(regex.chars());
        self.expect_alternative()
    }

    fn expect_regex(&mut self) -> OperationArg {
        let mut cur_regex = String::new();
        let mut node_opt: Option<OperationArg> = None;
        loop {
            let next_opt = self.peek();
            if next_opt.is_none() {
                break;
            }
            let next = next_opt.unwrap();
            match next {
                '(' => {
                    self.advance();
                    let mut subexpr = self.expect_alternative();
                    if let OperationArg::Const {
                        expr,
                        parenthesized: _,
                    } = subexpr
                    {
                        subexpr = OperationArg::Const {
                            expr,
                            parenthesized: true,
                        }
                    }
                    self.consume(')');

                    if !concat_const(&mut node_opt, &mut cur_regex)
                        && node_opt.is_none()
                    {
                        node_opt = Some(subexpr);
                        continue;
                    }

                    let node_variant = node_opt.as_mut().unwrap();
                    match node_variant {
                        OperationArg::Operation(operation_type) => {
                            match operation_type {
                                Operation::Concat(operands) => {
                                    operands.push(subexpr);
                                }
                                Operation::Alternative(_)
                                | Operation::Star(_) => {
                                    *node_variant = OperationArg::Operation(
                                        Operation::Concat(vec![
                                            node_variant.clone(),
                                            subexpr,
                                        ]),
                                    );
                                }
                            }
                        }
                        OperationArg::Const { .. } => {
                            *node_variant =
                                OperationArg::Operation(Operation::Concat(
                                    vec![node_variant.clone(), subexpr],
                                ));
                        }
                    }
                }
                ')' | '|' => {
                    break;
                }
                '*' => {
                    self.advance();
                    concat_const(&mut node_opt, &mut cur_regex);

                    let node_variant = node_opt.as_mut().unwrap();
                    match node_variant {
                        OperationArg::Const {
                            expr,
                            parenthesized,
                        } => {
                            if *parenthesized {
                                *node_variant =
                                    OperationArg::Operation(Operation::Star(
                                        Box::new(OperationArg::Const {
                                            expr: expr.clone(),
                                            parenthesized: true,
                                        }),
                                    ))
                            } else {
                                let last = expr.pop().unwrap();
                                *node_variant = OperationArg::Operation(
                                    Operation::Concat(vec![
                                        OperationArg::Const {
                                            expr: expr.clone(),
                                            parenthesized: false,
                                        },
                                        OperationArg::Operation(
                                            Operation::Star(Box::new(
                                                OperationArg::Const {
                                                    expr: last.to_string(),
                                                    parenthesized: false,
                                                },
                                            )),
                                        ),
                                    ]),
                                )
                            }
                        }
                        OperationArg::Operation(op) => {
                            match op {
                                Operation::Concat(args) => {
                                    let last = args.last_mut().unwrap();
                                    match last {
                                        OperationArg::Const {
                                            expr,
                                            parenthesized,
                                        } => {
                                            if *parenthesized || expr.len() == 1
                                            {
                                                *last = OperationArg::Operation(
                                                    Operation::Star(Box::new(
                                                        last.deref().clone(),
                                                    )),
                                                )
                                            } else {
                                                let last_char =
                                                    expr.pop().unwrap();
                                                args.push(OperationArg::Operation(Operation::Star(
                                                    Box::new(OperationArg::Const { expr: last_char.to_string(), parenthesized: false })
                                                )));
                                            }
                                        }
                                        OperationArg::Operation(_) => {
                                            propogate_star(last);
                                            *last = OperationArg::Operation(
                                                Operation::Star(Box::new(
                                                    last.deref().clone(),
                                                )),
                                            );
                                        }
                                    }
                                }
                                Operation::Alternative(args) => {
                                    args.iter_mut().for_each(propogate_star);
                                    *node_variant = OperationArg::Operation(
                                        Operation::Star(Box::new(
                                            OperationArg::Operation(
                                                op.deref().clone(),
                                            ),
                                        )),
                                    )
                                }
                                Operation::Star(arg) => {
                                    propogate_star(arg);
                                }
                            }
                        }
                    }
                }
                c => {
                    cur_regex.push(c);
                    self.advance();
                }
            }
        }

        concat_const(&mut node_opt, &mut cur_regex);
        node_opt.expect("expected to parse anything")
    }

    fn expect_alternative(&mut self) -> OperationArg {
        let node = self.expect_regex();
        if !self.expect('|') {
            return node;
        }
        self.advance();

        let mut children = vec![node];
        loop {
            let subexpr = self.expect_regex();
            if let OperationArg::Operation(Operation::Alternative(mut x)) =
                subexpr
            {
                children.append(&mut x);
            } else {
                children.push(subexpr);
            }

            if !self.expect('|') {
                break;
            }
            self.advance();
        }

        OperationArg::Operation(Operation::Alternative(children))
    }

    fn report(&self, message: &str) -> ! {
        panic!("[col {}] {}", self.index, message);
    }

    fn peek(&mut self) -> Option<char> {
        if self.next_char.is_none() {
            self.next_char = self
                .regex_iter
                .as_mut()
                .expect("expeted initialized regex iter")
                .next();
            self.index += 1;
        }
        self.next_char
    }

    fn advance(&mut self) {
        if self.next_char.is_none() {
            self.regex_iter
                .as_mut()
                .expect("expeted initialized regex iter")
                .next();
            self.index += 1;
        } else {
            self.next_char = None;
        }
    }

    fn consume(&mut self, expected: char) {
        if let Some(next) = self.peek() {
            if next != expected {
                self.report(
                    format!("expected {}, but {} found", expected, next)
                        .as_str(),
                );
            }
            self.advance();
        } else {
            self.report(
                format!("expected {}, but EOF found", expected).as_str(),
            );
        }
    }

    fn expect(&mut self, expected: char) -> bool {
        let next_opt = self.peek();
        if let Some(next) = next_opt {
            expected == next
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::parsing::{Operation, OperationArg};

    use super::Parser;

    #[test]
    fn basic_const_test() {
        let expr = "test_regex";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Const {
            expr: expr.to_string(),
            parenthesized: false,
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_alternative_test() {
        let expr = "abc|cde";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Alternative(vec![
            OperationArg::Const {
                expr: "abc".to_string(),
                parenthesized: false,
            },
            OperationArg::Const {
                expr: "cde".to_string(),
                parenthesized: false,
            },
        ]));
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_concat_test() {
        let expr = "abc(cde)efg";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Concat(vec![
            OperationArg::Const {
                expr: "abc".to_string(),
                parenthesized: false,
            },
            OperationArg::Const {
                expr: "cde".to_string(),
                parenthesized: true,
            },
            OperationArg::Const {
                expr: "efg".to_string(),
                parenthesized: false,
            },
        ]));
        assert_eq!(expected, res);
    }

    #[test]
    fn basic_star_test() {
        let expr = "(abc)*";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Star(Box::new(
            OperationArg::Const {
                expr: "abc".to_string(),
                parenthesized: true,
            },
        )));
        assert_eq!(expected, res);
    }

    #[test]
    fn concat_with_star() {
        let expr = "(abc)*(cde)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Concat(vec![
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "abc".to_string(),
                    parenthesized: true,
                },
            ))),
            OperationArg::Const {
                expr: "cde".to_string(),
                parenthesized: true,
            },
        ]));
        assert_eq!(expected, res);
    }

    #[test]
    fn star_concat() {
        let expr = "(ab)*(ed)*";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Concat(vec![
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "ab".to_string(),
                    parenthesized: true,
                },
            ))),
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "ed".to_string(),
                    parenthesized: true,
                },
            ))),
        ]));
        assert_eq!(expected, res);
    }

    #[test]
    fn double_star() {
        let expr = "(abc)**";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = OperationArg::Operation(Operation::Star(Box::new(
            OperationArg::Const {
                expr: "abc".to_string(),
                parenthesized: true,
            },
        )));

        assert_eq!(expected, res)
    }

    #[test]
    fn non_greedy_star() {
        let expr = "(cd)qa*";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = OperationArg::Operation(Operation::Concat(vec![
            OperationArg::Const {
                expr: "cd".to_string(),
                parenthesized: true,
            },
            OperationArg::Const {
                expr: "q".to_string(),
                parenthesized: false,
            },
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "a".to_string(),
                    parenthesized: false,
                },
            ))),
        ]));

        assert_eq!(expected, res)
    }

    #[test]
    fn concat_double_star() {
        let expr = "(abc)*(cde)**";
        let mut parser = Parser::default();
        let res = parser.parse(expr);

        let expected = OperationArg::Operation(Operation::Concat(vec![
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "abc".to_string(),
                    parenthesized: true,
                },
            ))),
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "cde".to_string(),
                    parenthesized: true,
                },
            ))),
        ]));

        assert_eq!(expected, res);
    }

    #[test]
    fn the_test() {
        let expr = "(abc)*((cde)|(edf))**|(qrp)";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Alternative(vec![
            OperationArg::Operation(Operation::Concat(vec![
                OperationArg::Operation(Operation::Star(Box::new(
                    OperationArg::Const {
                        expr: "abc".to_string(),
                        parenthesized: true,
                    },
                ))),
                OperationArg::Operation(Operation::Star(Box::new(
                    OperationArg::Operation(Operation::Alternative(vec![
                        OperationArg::Const {
                            expr: "cde".to_string(),
                            parenthesized: true,
                        },
                        OperationArg::Const {
                            expr: "edf".to_string(),
                            parenthesized: true,
                        },
                    ])),
                ))),
            ])),
            OperationArg::Const {
                expr: "qrp".to_string(),
                parenthesized: true,
            },
        ]));
        assert_eq!(expected, res);
    }

    #[test]
    fn star_simplification() {
        let expr = "((abc)*|(bcd)*)**a***(((abc)*)**)***";
        let mut parser = Parser::default();

        let res = parser.parse(expr);
        let expected = OperationArg::Operation(Operation::Concat(vec![
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Operation(Operation::Alternative(vec![
                    OperationArg::Const {
                        expr: "abc".to_string(),
                        parenthesized: true,
                    },
                    OperationArg::Const {
                        expr: "bcd".to_string(),
                        parenthesized: true,
                    },
                ])),
            ))),
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "a".to_string(),
                    parenthesized: false,
                },
            ))),
            OperationArg::Operation(Operation::Star(Box::new(
                OperationArg::Const {
                    expr: "abc".to_string(),
                    parenthesized: true,
                },
            ))),
        ]));
        assert_eq!(expected, res);
    }
}
