use std::vec;

use super::parser::{AltArg, ConcatArg, ParsingResult, StarArg};

pub fn ssnf(arg: ParsingResult) -> ParsingResult {
    match arg {
        ParsingResult::Alt { args, accepts_empty } => ParsingResult::Alt {
            args: args
                .into_iter()
                .map(|item| AltArg::from(ssnf(ParsingResult::from(item))))
                .collect(),
            accepts_empty,
        },
        ParsingResult::Concat { args, accepts_empty } => {
            ParsingResult::Concat {
                args: args
                    .into_iter()
                    .map(|item| match item {
                        ConcatArg::Alt { args, accepts_empty } => {
                            ConcatArg::from(ssnf(ParsingResult::Alt {
                                args,
                                accepts_empty,
                            }))
                        }
                        ConcatArg::Star(arg) => {
                            ConcatArg::from(ssnf(ParsingResult::Star(arg)))
                        }
                        ConcatArg::Char(c) => ConcatArg::Char(c),
                    })
                    .collect(),
                accepts_empty,
            }
        }
        ParsingResult::Star(arg) => ParsingResult::Star(Box::new(
            StarArg::from(ss(ParsingResult::from(*arg))),
        )),
    }
}

fn ss(arg: ParsingResult) -> ParsingResult {
    match arg {
        ParsingResult::Alt { args, accepts_empty } => {
            let mut new_args: Vec<AltArg> = vec![];
            let mut new_accepts_empty = true;
            args.into_iter().for_each(|item| {
                let res = ss(ParsingResult::from(item));
                match res {
                    ParsingResult::Alt { args, accepts_empty } => {
                        new_accepts_empty &= accepts_empty;
                        new_args.extend(args);
                    }
                    ParsingResult::Concat { args, accepts_empty } => {
                        new_accepts_empty &= accepts_empty;
                        new_args.push(AltArg::Concat { args, accepts_empty })
                    }
                    ParsingResult::Star(arg) => {
                        unreachable!();
                    }
                }
            });
            ParsingResult::Alt {
                args: new_args,
                accepts_empty: new_accepts_empty,
            }
        }
        ParsingResult::Concat { args, accepts_empty } => {
            if accepts_empty {
                let mut alt_args: Vec<AltArg> = vec![];
                let mut alt_accepts_empty = false;
                args.into_iter().for_each(|item| {
                    let res = ss(ParsingResult::from(item));
                    match res {
                        ParsingResult::Alt { args, accepts_empty } => {
                            alt_accepts_empty &= accepts_empty;
                            alt_args.extend(args);
                        }
                        ParsingResult::Concat { args, accepts_empty } => {
                            alt_accepts_empty &= accepts_empty;
                            alt_args
                                .push(AltArg::Concat { args, accepts_empty });
                        }
                        ParsingResult::Star(arg) => {
                            unreachable!();
                        }
                    }
                });
                ParsingResult::Alt {
                    args: alt_args,
                    accepts_empty: alt_accepts_empty,
                }
            } else {
                ParsingResult::Concat {
                    args: args
                        .into_iter()
                        .map(|item| match item {
                            ConcatArg::Alt { args, accepts_empty } => {
                                ConcatArg::from(ssnf(ParsingResult::Alt {
                                    args,
                                    accepts_empty,
                                }))
                            }
                            ConcatArg::Star(arg) => {
                                ConcatArg::from(ssnf(ParsingResult::Star(arg)))
                            }
                            ConcatArg::Char(c) => ConcatArg::Char(c),
                        })
                        .collect(),
                    accepts_empty,
                }
            }
        }
        ParsingResult::Star(arg) => ss(ParsingResult::from(*arg)),
    }
}
