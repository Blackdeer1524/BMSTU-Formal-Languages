use std::{ops::Deref, os::unix::prelude::OpenOptionsExt, str::Chars, vec};

#[derive(Clone)]
enum Operation {
    Concat(Vec<OperationArg>),
    Alternative(Vec<OperationArg>),
    Star(Box<OperationArg>),
}

#[derive(Clone)]
enum OperationArg {
    Operation(Operation),
    Const { expr: String, parenthesized: bool },
}

struct Parser<'a> {
    regex_iter: Option<Chars<'a>>,
    next_char: Option<char>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn parse(&mut self, regex: &'a str) {
        self.index = 0;
        self.next_char = None;
        self.regex_iter = Some(regex.chars());
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
                        parenthesized,
                    } = subexpr
                    {
                        subexpr = OperationArg::Const {
                            expr,
                            parenthesized: true,
                        }
                    }
                    self.consume(')');

                    if node_opt.is_none() {
                        if cur_regex.is_empty() {
                            node_opt = Some(subexpr);
                        } else {
                            node_opt = Some(OperationArg::Operation(
                                Operation::Concat(vec![
                                    subexpr,
                                    OperationArg::Const {
                                        expr: cur_regex,
                                        parenthesized: false,
                                    },
                                ]),
                            ));
                            cur_regex = String::new();
                        }
                        continue;
                    }
                    let node_variant = node_opt.as_mut().unwrap();
                    match node_variant {
                        OperationArg::Operation(_) => todo!(),
                        OperationArg::Const {
                            expr,
                            parenthesized,
                        } => todo!(),
                    }
                    // match node_variant {
                    //     Operation::Concat(operands) => {
                    //         if !cur_regex.is_empty() {
                    //             operands.push(OperationArg::Const {
                    //                 expr: cur_regex,
                    //                 parenthesized: false,
                    //             });
                    //             cur_regex = String::new();
                    //         }
                    //         operands.push(OperationArg::Operation(subexpr));
                    //     }
                    //     Operation::Alternative(_) => {
                    //         if !cur_regex.is_empty() {
                    //             node_opt = Some(Operation::Concat(vec![
                    //                 OperationArg::Operation(node_opt.unwrap()),
                    //                 OperationArg::Const {
                    //                     expr: cur_regex,
                    //                     parenthesized: false,
                    //                 },
                    //                 OperationArg::Operation(subexpr),
                    //             ]));
                    //             cur_regex = String::new();
                    //         } else {
                    //             node_opt = Some(Operation::Concat(vec![
                    //                 OperationArg::Operation(node_opt.unwrap()),
                    //                 OperationArg::Operation(subexpr),
                    //             ]))
                    //         }
                    //     }
                    //     Operation::Star(operand) => {
                    //         if !cur_regex.is_empty() {
                    //             node_opt = Some(Operation::Concat(vec![
                    //                 OperationArg::Operation(Operation::Star(
                    //                     Box::new(
                    //                         operand.deref().deref().clone(),
                    //                     ),
                    //                 )),
                    //                 OperationArg::Const {
                    //                     expr: cur_regex,
                    //                     parenthesized: false,
                    //                 },
                    //                 OperationArg::Operation(subexpr),
                    //             ]));
                    //             cur_regex = String::new();
                    //         } else {
                    //             node_opt = Some(Operation::Concat(vec![
                    //                 OperationArg::Operation(Operation::Star(
                    //                     Box::new(
                    //                         operand.deref().deref().clone(),
                    //                     ),
                    //                 )),
                    //                 OperationArg::Operation(subexpr),
                    //             ]));
                    //         }
                    //     }
                    // }
                }
                ')' | '|' => {
                    break;
                }
                '*' => {
                    if node_opt.is_none() {
                        if cur_regex.is_empty() {
                            self.report("Star operation on empty expression");
                        } else {
                            let last_char = cur_regex.pop().unwrap();
                            node_opt = Some(Operation::Concat(vec![
                                OperationArg::Const {
                                    expr: cur_regex,
                                    parenthesized: false,
                                },
                                OperationArg::Operation(Operation::Star(
                                    Box::new(OperationArg::Const(
                                        last_char.to_string(),
                                    )),
                                )),
                            ]));
                            cur_regex = String::new();
                        }
                    } else {
                        let mut node_variant = node_opt.as_mut().unwrap();
                        match node_variant {
                            Operation::Concat(operands) => {
                                if !cur_regex.is_empty() {
                                    operands.push(OperationArg::Operation(
                                        Operation::Star(Box::new(
                                            OperationArg::Const(cur_regex),
                                        )),
                                    ));
                                    cur_regex = String::new();
                                } else {
                                    let last = operands.last_mut().unwrap();
                                    match last {
                                        OperationArg::Operation(op) => {
                                            *last = OperationArg::Operation(
                                                Operation::Star(Box::new(
                                                    *last,
                                                )),
                                            );
                                        }
                                        OperationArg::Const(regex_str) => {
                                            let last = regex_str.pop().unwrap();
                                            operands.push(
                                                OperationArg::Operation(
                                                    Operation::Star(Box::new(
                                                        OperationArg::Const(
                                                            last.to_string(),
                                                        ),
                                                    )),
                                                ),
                                            )
                                        }
                                    }
                                }
                            }
                            Operation::Alternative(operands) => {}
                            Operation::Star(operand) => {}
                        }
                    }

                    let node_variant = node_opt.as_mut().unwrap();
                    match node_variant {
                        Operation::Concat(operands)
                        | Operation::Alternative(operands) => {
                            let last = operands
                                .pop()
                                .expect("Nodes have at least one child");
                            operands.push(OperationArg::Operation(
                                Operation::Star(Box::new(last)),
                            ));
                        }
                        Operation::Star(operand) => (), // так мы оптимизируем вложенною звёздочку
                    }
                }
                c => {
                    cur_regex.push(c);
                }
            }
        }
        node_opt.expect("expected to parse anything")
    }

    fn report(&self, message: &str) -> ! {
        panic!("[col {}] {}", self.index, message);
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
            children.push(subexpr);
            if !self.expect('|') {
                break;
            }
            self.advance();
        }

        OperationArg::Operation(Operation::Alternative(children))
        // NodeVariant::Node(Operation {
        //     children,
        //     operation: Operation::Alternative,
        // })
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
