use std::ops::{Deref, DerefMut};

use super::parser::{AltArg, ConcatArg, ParsingResult, StarArg};

// fn alt_ssnf(root: &mut AltArg) {
//     match root {
//         AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
//             args.iter_mut().for_each(concat_ssnf);
//         }
//         AltArg::Star(arg) => {
//             star_ss(arg);
//         }
//         AltArg::Regex { arg, parenthesized } => (),
//     }
// }
// fn concat_ssnf(root: &mut ConcatArg) {
//     match root {
//         ConcatArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
//             args.iter_mut().for_each(concat_ssnf);
//         }
//         ConcatArg::Alt { args, accepts_empty } => {
//             args.iter_mut().for_each(alt_ssnf);
//         }
//         ConcatArg::Star(arg) => star_ss(arg),
//         ConcatArg::Regex { arg, parenthesized } => (),
//     }
// }
//
// fn star_ss(root: &mut StarArg) {
//     match root {
//         StarArg::Alt { args, accepts_empty } => {
//             alter_ss(args);
//         }
//         StarArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
//             if *body_accepts_empty && *tail_accepts_empty {
//                 args.iter_mut().for_each(concat_ss);
//                 *root = StarArg::Alt { args, accepts_empty: todo!() };
//             } else {
//                 args.iter_mut().for_each(concat_ssnf)
//             }
//         }
//         StarArg::Regex(_) => (),
//     }
// }
//
// fn alter_ss(root: &mut Vec<AltArg>) {
//     let mut res: Vec<AltArg> = vec![];
//     for i in 0..root.len() {
//         let arg = root.get_mut(i).unwrap();
//         match arg {
//             AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
//                 if !*body_accepts_empty || !*tail_accepts_empty {
//                     args.iter_mut().for_each(concat_ssnf);
//                 } else {
//                     res.extend(args.iter_mut().flat_map(|item| {
//                         concat_ss(item);
//                         todo!("функция concat_ss отдаёт вектор");
//                         match item {
//                             ConcatArg::Concat {
//                                 args,
//                                 body_accepts_empty,
//                                 tail_accepts_empty,
//                             } => vec![AltArg::Concat {
//                                 args: args.clone(),
//                                 body_accepts_empty: *body_accepts_empty,
//                                 tail_accepts_empty: *tail_accepts_empty,
//                             }],
//                             ConcatArg::Alt { args, accepts_empty } => {
//                                 args.clone()
//                             }
//                             ConcatArg::Star(arg) => {
//                                 vec![AltArg::Star(arg.clone())]
//                             }
//                             ConcatArg::Regex { arg, parenthesized } => {
//                                 vec![AltArg::Regex {
//                                     arg: arg.clone(),
//                                     parenthesized: *parenthesized,
//                                 }]
//                             }
//                         }
//                     }));
//                 }
//             }
//             AltArg::Star(star_arg) => match star_arg.as_mut() {
//                 StarArg::Alt { args, accepts_empty } => {
//                     todo!();
//                 }
//                 StarArg::Concat {
//                     args,
//                     body_accepts_empty,
//                     tail_accepts_empty,
//                 } => todo!(),
//                 StarArg::Regex(_) => todo!(),
//             },
//             AltArg::Regex { arg, parenthesized } => todo!(),
//         }
//     }
//     *root = res;
// }
//
// fn concat_ss(item: &mut ConcatArg) {
//     match item {
//         ConcatArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {}
//         ConcatArg::Alt { args, accepts_empty } => todo!(),
//         ConcatArg::Star(_) => todo!(),
//         ConcatArg::Regex { arg, parenthesized } => todo!(),
//     }
// }

fn ssnf_alt(arg: &mut AltArg) {
    match arg {
        AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            args.iter_mut().for_each(ssnf_concat);
        }
        AltArg::Alt { args, accepts_empty } => {
            args.iter_mut().for_each(ssnf_alt);
        }
        AltArg::Star(star_arg) => {
            ss_star(star_arg);
            *arg = match star_arg.as_mut() {
                StarArg::Alt { args, accepts_empty } => AltArg::Alt {
                    args: args.clone(),
                    accepts_empty: *accepts_empty,
                },
                StarArg::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => AltArg::Concat {
                    args: args.clone(),
                    body_accepts_empty: *body_accepts_empty,
                    tail_accepts_empty: *tail_accepts_empty,
                },
                StarArg::Regex(regex_str) => AltArg::Regex {
                    arg: regex_str.clone(),
                    parenthesized: true,
                },
            }
        }
        AltArg::Regex { arg, parenthesized } => todo!(),
    }
}

fn ssnf_concat(arg: &mut ConcatArg) {
    match arg {
        ConcatArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            args.iter_mut().for_each(ssnf_concat);
        }
        ConcatArg::Alt { args, accepts_empty } => {
            args.iter_mut().for_each(ssnf_alt)
        }
        ConcatArg::Star(star_arg) => {
            ss_star(star_arg);
            *arg = match star_arg.as_mut() {
                StarArg::Alt { args, accepts_empty } => ConcatArg::Alt {
                    args: args.clone(),
                    accepts_empty: *accepts_empty,
                },
                StarArg::Concat {
                    args,
                    body_accepts_empty,
                    tail_accepts_empty,
                } => ConcatArg::Concat {
                    args: args.clone(),
                    body_accepts_empty: *body_accepts_empty,
                    tail_accepts_empty: *tail_accepts_empty,
                },
                StarArg::Regex(regex_str) => ConcatArg::Regex {
                    arg: regex_str.clone(),
                    parenthesized: true,
                },
            }
        }
        ConcatArg::Regex { arg, parenthesized } => (),
    }
}

fn ss_star(arg: &mut StarArg) {
    match arg {
        StarArg::Alt { args, accepts_empty } => {
            args.iter_mut().for_each(ss_alt);
        }
        StarArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            if *body_accepts_empty && *tail_accepts_empty {
                let new_args: Vec<AltArg> = args
                    .iter_mut()
                    .map(|item| {
                        ssnf_concat(item);
                        match item {
                            ConcatArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            } => AltArg::Concat {
                                args: args.clone(),
                                body_accepts_empty: *body_accepts_empty,
                                tail_accepts_empty: *tail_accepts_empty,
                            },
                            ConcatArg::Alt { args, accepts_empty } => {
                                AltArg::Alt {
                                    args: args.clone(),
                                    accepts_empty: *accepts_empty,
                                }
                            }
                            ConcatArg::Star(arg) => AltArg::Star(arg.clone()),
                            ConcatArg::Regex { arg, parenthesized } => {
                                AltArg::Regex {
                                    arg: arg.clone(),
                                    parenthesized: *parenthesized,
                                }
                            }
                        }
                    })
                    .collect();
                *arg = StarArg::Alt { args: new_args, accepts_empty: todo!() }
            } else {
                args.iter_mut().for_each(ssnf_concat);
            }
        }
        StarArg::Regex(_) => todo!(),
    }
}

fn ss_alt(arg: &mut AltArg) {}

pub fn apply_ssnf(root: &mut ParsingResult) {
    match root {
        ParsingResult::Alt { args, accepts_empty } => todo!(),
        ParsingResult::Concat {
            args,
            body_accepts_empty,
            tail_accepts_empty,
            parenthesized,
        } => todo!(),
        ParsingResult::Star(_) => todo!(),
        ParsingResult::Regex { arg, parenthesized } => todo!(),
    }
}
