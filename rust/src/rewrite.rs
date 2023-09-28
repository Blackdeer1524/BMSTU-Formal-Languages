use std::{str::Chars, usize, vec};

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
    Regex {
        arg: String,
        parenthesized: bool,
    },
}

#[derive(Default)]
struct Parser<'a> {
    index: usize,
    expr_iter: Option<Chars<'a>>,
    next_char: Option<char>,
}

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
    pub fn parse(&mut self, regex: &'a str) {
        self.index = 0;
        self.expr_iter = Some(regex.chars());
        self.next_char = None;
    }

    fn expect_alternative(&mut self) -> ParsingResult {
        let mut alt_args: Vec<AltArgs> = vec![];
        let mut alt_accepts_empty = true;
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
                    parenthesized: _,
                } => {
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

        ParsingResult::Alt { args: alt_args, accepts_empty: alt_accepts_empty }
    }

    fn expect_unary(&mut self) -> ParsingResult {
        let subexpr = self.expect_concat();
        if !self.check('*') {
            return subexpr;
        }
        self.advance();

        match subexpr {
            ParsingResult::Alt { args, accepts_empty } => {
                ParsingResult::Star(Box::new(StarArg::Alt {
                    args,
                    accepts_empty,
                }))
            }
            ParsingResult::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized: true,
            } => ParsingResult::Star(Box::new(StarArg::Concat {
                args,
                body_accepts_empty,
                tail_accepts_empty,
            })),
            ParsingResult::Concat {
                mut args,
                body_accepts_empty,
                mut tail_accepts_empty,
                parenthesized: false,
            } => {
                let mut last =
                    args.pop().expect("Concat always have at least 2 items");
                match last {
                    ConcatArgs::Alt {
                        args: last_args,
                        accepts_empty: last_accepts_empty,
                    } => {
                        tail_accepts_empty = true;
                        last = ConcatArgs::Star(Box::new(StarArg::Alt {
                            args: last_args,
                            accepts_empty: last_accepts_empty,
                        }));
                    }
                    ConcatArgs::Star(_) => (),
                    ConcatArgs::Regex { arg, parenthesized } => {
                        tail_accepts_empty = true;
                        last = ConcatArgs::Star(Box::new(StarArg::Regex {
                            arg,
                            parenthesized,
                        }));
                    }
                    ConcatArgs::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        last = ConcatArgs::Star(Box::new(StarArg::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        }))
                    }
                };
                args.push(last);
                ParsingResult::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                    parenthesized: true,
                }
            }
            ParsingResult::Star(_) => subexpr,
            ParsingResult::Regex { arg, parenthesized } => {
                ParsingResult::Star(Box::new(StarArg::Regex {
                    arg,
                    parenthesized,
                }))
            }
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
            self.advance();
            let next = next_opt.unwrap();
            match next {
                '(' => {
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
                        ParsingResult::Regex { arg, parenthesized } => {
                            main_args
                                .push(ConcatArgs::Regex { arg, parenthesized });
                            main_body_accepts_empty &= main_tail_accepts_empty;
                            main_tail_accepts_empty = true;
                        }
                    };
                }
                c => {
                    const_regex.push(c);
                }
            }
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
                    parenthesized: true,
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
}
