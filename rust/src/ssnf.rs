use super::parser::{AltArg, ConcatArg, ParsingResult, StarArg};

pub fn ssnf(arg: ParsingResult) -> ParsingResult {
    let result: ParsingResult;
    match arg {
        ParsingResult::Alt { args, accepts_empty } => {
            result = ParsingResult::Alt {
                args: args
                    .into_iter()
                    .map(|item| {
                        let res = ssnf(ParsingResult::from(item));
                        AltArg::from(res)
                    })
                    .collect(),
                accepts_empty,
            }
        }
        ParsingResult::Concat {
            args,
            mut body_accepts_empty,
            mut tail_accepts_empty,
            parenthesized,
        } => {
            body_accepts_empty = true;
            tail_accepts_empty = true;
            let new_args: Vec<ConcatArg> = args
                .into_iter()
                .map(|item| {
                    let res = ssnf(ParsingResult::from(item));
                    body_accepts_empty &= tail_accepts_empty;
                    match res {
                        ParsingResult::Alt { args, accepts_empty } => {
                            tail_accepts_empty = accepts_empty;
                            ConcatArg::Alt { args, accepts_empty }
                        }
                        ParsingResult::Concat {
                            args,
                            body_accepts_empty: inner_body_accepts_empty,
                            tail_accepts_empty: inner_tail_accepts_empty,
                            parenthesized,
                        } => {
                            tail_accepts_empty = inner_body_accepts_empty
                                && inner_tail_accepts_empty;
                            ConcatArg::Concat {
                                args,
                                body_accepts_empty: inner_body_accepts_empty,
                                tail_accepts_empty: inner_tail_accepts_empty,
                            }
                        }
                        ParsingResult::Star(arg) => {
                            tail_accepts_empty = true;
                            ConcatArg::Star(arg)
                        }
                        ParsingResult::Regex { arg, parenthesized } => {
                            tail_accepts_empty = false;
                            ConcatArg::Regex { arg, parenthesized }
                        }
                    }
                })
                .collect();
            result = ParsingResult::Concat {
                args: new_args,
                body_accepts_empty,
                tail_accepts_empty,
                parenthesized,
            };
        }
        ParsingResult::Star(arg) => {
            let intermediate = ss(ParsingResult::from(*arg));
            result = ParsingResult::Star(Box::new(StarArg::from(intermediate)));
        }
        ParsingResult::Regex { arg, parenthesized } => {
            result = ParsingResult::Regex { arg, parenthesized }
        }
    }
    result
}

fn ss(arg: ParsingResult) -> ParsingResult {
    let result: ParsingResult;
    match arg {
        ParsingResult::Alt { mut args, mut accepts_empty } => {
            accepts_empty = true;
            let mut tail_args: Vec<AltArg> = vec![];
            let mut new_args = args
                .into_iter()
                .map(|item| {
                    let res = ss(ParsingResult::from(item));
                    match res {
                        ParsingResult::Alt {
                            mut args,
                            accepts_empty: inner_accepts_empty,
                        } => {
                            accepts_empty &= inner_accepts_empty;
                            let alt_arg = args.pop().expect(
                                "there will always be at least 2 arguments",
                            );
                            tail_args.append(&mut args);
                            alt_arg
                        }
                        ParsingResult::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                            parenthesized,
                        } => {
                            accepts_empty &=
                                body_accepts_empty && tail_accepts_empty;
                            AltArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            }
                        }
                        ParsingResult::Star(arg) => AltArg::Star(arg),
                        ParsingResult::Regex { arg, parenthesized } => {
                            accepts_empty = false;
                            AltArg::Regex { arg, parenthesized }
                        }
                    }
                })
                .collect::<Vec<AltArg>>();
            new_args.extend(tail_args.into_iter());
            new_args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            });
            result = ParsingResult::Alt { args: new_args, accepts_empty };
        }
        ParsingResult::Concat {
            mut args,
            body_accepts_empty,
            tail_accepts_empty,
            parenthesized,
        } => {
            if body_accepts_empty && tail_accepts_empty {
                let mut new_accepts_empty = true;
                let mut tail_alt_arg: Vec<AltArg> = vec![];
                let mut new_args = args
                    .into_iter()
                    .map(|item| {
                        let res = ss(ParsingResult::from(item));
                        match res {
                            ParsingResult::Alt { mut args, accepts_empty } => {
                                new_accepts_empty &= accepts_empty;
                                let last = args.pop().expect(
                                    "there will always be at least 2 arguments",
                                );
                                tail_alt_arg.append(&mut args);
                                last
                            }
                            ParsingResult::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                                parenthesized,
                            } => {
                                new_accepts_empty &=
                                    body_accepts_empty && tail_accepts_empty;
                                AltArg::Concat {
                                    args,
                                    body_accepts_empty,
                                    tail_accepts_empty,
                                }
                            }
                            ParsingResult::Star(arg) => AltArg::Star(arg),
                            ParsingResult::Regex { arg, parenthesized } => {
                                new_accepts_empty = false;
                                AltArg::Regex { arg, parenthesized }
                            }
                        }
                    })
                    .collect::<Vec<AltArg>>();
                new_args.append(&mut tail_alt_arg);
                new_args.sort_unstable_by(|left, right| {
                    left.to_string().cmp(&right.to_string())
                });
                result = ParsingResult::Alt {
                    args: new_args,
                    accepts_empty: new_accepts_empty,
                };
            } else {
                let mut new_tail_accepts_empty = true;
                let mut new_body_accepts_empty = true;
                args = args
                    .into_iter()
                    .map(|item| {
                        let res = ssnf(ParsingResult::from(item));
                        new_body_accepts_empty &= new_tail_accepts_empty;
                        match res {
                            ParsingResult::Alt { args, accepts_empty } => {
                                new_tail_accepts_empty &= accepts_empty;
                                ConcatArg::Alt { args, accepts_empty }
                            }
                            ParsingResult::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                                parenthesized,
                            } => {
                                new_tail_accepts_empty &=
                                    body_accepts_empty && tail_accepts_empty;
                                ConcatArg::Concat {
                                    args,
                                    body_accepts_empty,
                                    tail_accepts_empty,
                                }
                            }
                            ParsingResult::Star(arg) => ConcatArg::Star(arg),
                            ParsingResult::Regex { arg, parenthesized } => {
                                new_tail_accepts_empty = false;
                                ConcatArg::Regex { arg, parenthesized }
                            }
                        }
                    })
                    .collect();
                result = ParsingResult::Concat {
                    args,
                    body_accepts_empty: new_body_accepts_empty,
                    tail_accepts_empty: new_tail_accepts_empty,
                    parenthesized,
                }
            }
        }
        ParsingResult::Star(arg) => {
            result = ParsingResult::from(*arg);
        }
        ParsingResult::Regex { arg, parenthesized } => {
            result = ParsingResult::Regex { arg, parenthesized }
        }
    };
    result
}
