use std::collections::LinkedList;
use std::ops::Deref;
use std::vec;

use crate::parser::AltArg;
use crate::parser::{ConcatArg, ParsingResult};

fn take_derivative(arg: ParsingResult, symbol: char) -> Option<ParsingResult> {
    match arg {
        ParsingResult::Alt { args, accepts_empty } => {
            let mut new_args: Vec<AltArg> = vec![];
            let mut new_accepts_empty = true;
            args.into_iter().for_each(|item| match item {
                AltArg::Concat { args, accepts_empty } => {
                    let res_opt =
                        take_derivative(ParsingResult::Concat { args, accepts_empty }, symbol);
                    if let Some(res) = res_opt {
                        match res {
                            ParsingResult::Alt { args, accepts_empty } => {
                                new_accepts_empty &= accepts_empty;
                                new_args.extend(args);
                            }
                            ParsingResult::Concat { args, accepts_empty } => {
                                new_accepts_empty &= accepts_empty;
                                new_args.push(AltArg::Concat { args, accepts_empty });
                            }
                            ParsingResult::Star(arg) => {
                                new_args.push(AltArg::Star(arg));
                            }
                        }
                    }
                }
                AltArg::Star(arg) => {
                    let inner_opt =
                        take_derivative(ParsingResult::from(arg.deref().clone()), symbol);
                    if let Some(inner) = inner_opt {
                        match inner {
                            ParsingResult::Alt { args, accepts_empty } => {
                                new_accepts_empty &= accepts_empty;
                                new_args.push(AltArg::Concat {
                                    args: vec![
                                        ConcatArg::Alt {
                                            args: Vec::from_iter(args),
                                            accepts_empty,
                                        },
                                        ConcatArg::Star(arg),
                                    ],
                                    accepts_empty,
                                });
                            }
                            ParsingResult::Concat { mut args, accepts_empty } => {
                                new_accepts_empty &= accepts_empty;
                                args.push(ConcatArg::Star(arg));
                                new_args.push(AltArg::Concat { args, accepts_empty });
                            }
                            ParsingResult::Star(inner_arg) => new_args.push(AltArg::Concat {
                                args: vec![ConcatArg::Star(inner_arg), ConcatArg::Star(arg)],
                                accepts_empty: true,
                            }),
                        }
                    }
                }
            });
            if new_args.is_empty() {
                return None;
            } else if new_args.len() == 1 {
                let last = new_args.pop().unwrap();
                return Some(ParsingResult::from(last));
            }
            Some(ParsingResult::Alt {
                args: LinkedList::from_iter(new_args),
                accepts_empty: new_accepts_empty,
            })
        }
        ParsingResult::Concat { mut args, accepts_empty: main_accepts_empty } => {
            if args.len() == 1 {
                if let ConcatArg::Char(c) = args.last().unwrap() {
                    if *c == symbol {
                        return Some(ParsingResult::Concat { args: vec![], accepts_empty: true });
                    } else {
                        return None;
                    }
                }
            }

            let mut alt_args: Vec<AltArg> = vec![];
            let mut alt_accepts_empty = main_accepts_empty;

            let mut empty_accepting_prefix_size: usize = 0;
            for item in &args {
                match item {
                    ConcatArg::Alt { args, accepts_empty } => {
                        if !accepts_empty {
                            break;
                        }
                    }
                    ConcatArg::Star(_) => (),
                    ConcatArg::Char(_) => break,
                }
                empty_accepting_prefix_size += 1;
            }
            for _ in 0..empty_accepting_prefix_size {
                let tail = args.split_off(1);
                let item = args.pop().unwrap();

                args = tail.clone();

                let derivative_opt = take_derivative(ParsingResult::from(item.clone()), symbol);
                if derivative_opt.is_none() {
                    break;
                }
                let derivative = derivative_opt.unwrap();
                match derivative {
                    ParsingResult::Alt { args, accepts_empty: inner_accepts_empty } => {
                        if !tail.is_empty() {
                            let mut res = vec![ConcatArg::Alt {
                                args: Vec::from_iter(args),
                                accepts_empty: inner_accepts_empty,
                            }];
                            res.extend(tail);
                            alt_args.push(AltArg::Concat {
                                args: res,
                                accepts_empty: main_accepts_empty && inner_accepts_empty,
                            });
                            alt_accepts_empty |= main_accepts_empty && inner_accepts_empty;
                        } else {
                            alt_args.extend(args);
                            alt_accepts_empty |= inner_accepts_empty;
                        }
                    }
                    ParsingResult::Concat { mut args, accepts_empty: inner_accepts_empty } => {
                        if tail.len() > 0 {
                            args.extend(tail);
                            alt_args.push(AltArg::Concat {
                                args,
                                accepts_empty: main_accepts_empty && inner_accepts_empty,
                            });
                            alt_accepts_empty |= main_accepts_empty && inner_accepts_empty;
                        } else {
                            alt_args
                                .push(AltArg::Concat { args, accepts_empty: inner_accepts_empty });
                            alt_accepts_empty |= inner_accepts_empty;
                        }
                    }
                    ParsingResult::Star(arg) => {
                        if tail.len() > 0 {
                            let mut t = vec![ConcatArg::Star(arg)];
                            t.extend(tail);
                            alt_args
                                .push(AltArg::Concat { args: t, accepts_empty: main_accepts_empty })
                        } else {
                            alt_args.push(AltArg::Concat {
                                args: vec![ConcatArg::Star(arg)],
                                accepts_empty: main_accepts_empty,
                            });
                            alt_accepts_empty = true;
                        }
                    }
                }
            }
            if alt_args.is_empty() {
                return None;
            }
            if alt_args.len() == 1 {
                return Some(ParsingResult::from(alt_args.pop().unwrap()));
            }
            return Some(ParsingResult::Alt {
                args: LinkedList::from_iter(alt_args),
                accepts_empty: alt_accepts_empty,
            });
        }
        ParsingResult::Star(arg) => {
            let res_opt = take_derivative(ParsingResult::from(*arg.clone()), symbol);
            if let Some(res) = res_opt {
                match res {
                    ParsingResult::Alt { args, accepts_empty } => {
                        return Some(ParsingResult::Concat {
                            args: vec![
                                ConcatArg::Alt { args: Vec::from_iter(args), accepts_empty },
                                ConcatArg::Star(arg),
                            ],
                            accepts_empty,
                        })
                    }
                    ParsingResult::Concat { mut args, accepts_empty } => {
                        args.push(ConcatArg::Star(arg));
                        return Some(ParsingResult::Concat { args, accepts_empty });
                    }
                    ParsingResult::Star(inner_star_arg) => {
                        return Some(ParsingResult::Concat {
                            args: vec![ConcatArg::Star(inner_star_arg), ConcatArg::Star(arg)],
                            accepts_empty: true,
                        });
                    }
                }
            }
            None
        }
    }
}
