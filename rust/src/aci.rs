use std::{
    borrow::Borrow,
    collections::{HashSet, LinkedList},
};

use crate::parser::{AltArg, ConcatArg, StarArg};

use super::parser::ParsingResult;

pub fn simplify(arg: ParsingResult) -> ParsingResult {
    match arg {
        ParsingResult::Alt {
            args: mut alt_args,
            accepts_empty: alt_accepts_empty,
        } => {
            let head_item = alt_args.pop_front().unwrap();
            match head_item {
                AltArg::Concat { args: mut first_args, accepts_empty } => {
                    let mut head_trim_number: usize = 0;
                    let mut head_accepts_empty = true;
                    loop {
                        let mut can_trim_further =
                            head_trim_number != first_args.len();
                        if !can_trim_further {
                            break;
                        }
                        let item_repr =
                            first_args[head_trim_number].to_string();
                        let can_distribute =
                            alt_args.iter().all(|item| match item {
                                AltArg::Concat { args, accepts_empty } => {
                                    if head_trim_number == args.len() {
                                        can_trim_further = false;
                                        false
                                    } else {
                                        item_repr
                                            == args[head_trim_number]
                                                .to_string()
                                    }
                                }
                                AltArg::Star(_) => false,
                            });
                        if can_distribute {
                            match &first_args[head_trim_number] {
                                ConcatArg::Alt { args, accepts_empty } => {
                                    head_accepts_empty &= accepts_empty;
                                }
                                ConcatArg::Star(_) => (),
                                ConcatArg::Char(_) => {
                                    head_accepts_empty = false;
                                }
                            }
                            head_trim_number += 1;
                        }
                        if !can_distribute || !can_trim_further {
                            break;
                        }
                    }
                    let mut tail_trim_number: usize = 0;
                    let mut tail_accepts_empty = true;
                    loop {
                        let mut can_trim_further = head_trim_number
                            + tail_trim_number
                            != first_args.len();
                        if !can_trim_further {
                            break;
                        }
                        let item_repr = first_args
                            [first_args.len() - (tail_trim_number + 1)]
                            .to_string();
                        let can_distribute =
                            alt_args.iter().all(|item| match item {
                                AltArg::Concat { args, accepts_empty } => {
                                    if head_trim_number + tail_trim_number
                                        == args.len()
                                    {
                                        can_trim_further = false;
                                        false
                                    } else {
                                        item_repr
                                            == args[args.len()
                                                - (tail_trim_number + 1)]
                                                .to_string()
                                    }
                                }
                                AltArg::Star(_) => false,
                            });
                        if can_distribute {
                            match &first_args
                                [first_args.len() - (tail_trim_number + 1)]
                            {
                                ConcatArg::Alt { args, accepts_empty } => {
                                    tail_accepts_empty &= accepts_empty;
                                }
                                ConcatArg::Star(_) => (),
                                ConcatArg::Char(_) => {
                                    tail_accepts_empty = false;
                                }
                            }
                            tail_trim_number += 1;
                        }
                        if !can_distribute || !can_trim_further {
                            break;
                        }
                    }

                    if head_trim_number == 0 && tail_trim_number == 0 {
                        alt_args.push_front(AltArg::Concat {
                            args: first_args,
                            accepts_empty,
                        });

                        return ParsingResult::Alt {
                            args: alt_args,
                            accepts_empty: alt_accepts_empty,
                        };
                    }
                    let tail: Vec<ConcatArg>;
                    if tail_trim_number > 0 {
                        tail = first_args
                            .split_off(first_args.len() - tail_trim_number);

                        alt_args.iter_mut().for_each(|item| match item {
                            AltArg::Concat { args, accepts_empty } => {
                                args.truncate(args.len() - tail_trim_number);
                            }
                            AltArg::Star(_) => unreachable!(),
                        })
                    } else {
                        tail = vec![];
                    }
                    let mut head: Vec<ConcatArg>;
                    if head_trim_number > 0 {
                        head = first_args.split_off(head_trim_number);
                        (first_args, head) = (head, first_args);

                        alt_args.iter_mut().for_each(|item| match item {
                            AltArg::Concat { args, accepts_empty } => {
                                let t = args.split_off(head_trim_number);
                                *args = t;
                            }
                            AltArg::Star(_) => todo!(),
                        })
                    } else {
                        head = vec![];
                    }
                    alt_args.push_front(AltArg::Concat {
                        args: first_args,
                        accepts_empty,
                    });

                    let alt_args_original_length = alt_args.len();
                    let mut remainder_accepts_empty = false;
                    let mut seen_alternative_args: HashSet<String> =
                        HashSet::default();
                    alt_args = alt_args
                        .into_iter()
                        .filter(|item| {
                            match item {
                                AltArg::Concat { args, accepts_empty } => {
                                    let mut current_alt_arg_accepts_empty =
                                        true;
                                    args.iter().for_each(|item| match item {
                                        ConcatArg::Alt {
                                            args,
                                            accepts_empty,
                                        } => {
                                            current_alt_arg_accepts_empty &=
                                                accepts_empty;
                                        }
                                        ConcatArg::Star(_) => (),
                                        ConcatArg::Char(_) => {
                                            current_alt_arg_accepts_empty =
                                                false;
                                        }
                                    });
                                    remainder_accepts_empty |=
                                        current_alt_arg_accepts_empty;
                                    if args.len() > 0 {
                                        // отсев повторений в альтернативе
                                        let item_repr = item.to_string();
                                        if seen_alternative_args
                                            .contains(&item_repr)
                                        {
                                            false
                                        } else {
                                            seen_alternative_args
                                                .insert(item_repr);
                                            true
                                        }
                                    } else {
                                        false
                                    }
                                }
                                AltArg::Star(_) => unreachable!(),
                            }
                        })
                        .collect();
                    if alt_args_original_length != alt_args.len() {
                        alt_args.push_front(AltArg::Concat {
                            args: vec![], // типа epsilon
                            accepts_empty: true,
                        });
                    }
                    head.push(ConcatArg::Alt {
                        args: Vec::from_iter(alt_args),
                        accepts_empty: remainder_accepts_empty,
                    });
                    head.extend(tail);
                    return ParsingResult::Concat {
                        args: head,
                        accepts_empty: head_accepts_empty
                            && remainder_accepts_empty
                            && tail_accepts_empty,
                    };
                }
                AltArg::Star(arg) => {
                    let star_repr = format!("({})*", arg.to_string());
                    let can_simplify = alt_args.iter().all(|item| match item {
                        AltArg::Concat { args, accepts_empty } => false,
                        AltArg::Star(arg) => {
                            let inner_repr = format!("({})*", arg.to_string());
                            inner_repr == star_repr
                        }
                    });
                    if can_simplify {
                        return ParsingResult::from(
                            alt_args.pop_front().unwrap(),
                        );
                    } else {
                        return ParsingResult::Alt {
                            args: alt_args,
                            accepts_empty: alt_accepts_empty,
                        };
                    }
                }
            }
        }
        ParsingResult::Concat { args, accepts_empty } => {
            let mut new_concat_args: Vec<ConcatArg> = vec![];
            args.into_iter().for_each(|item| match item {
                ConcatArg::Alt { args, accepts_empty } => {
                    let simplified = simplify(ParsingResult::Alt {
                        args: LinkedList::from_iter(args),
                        accepts_empty,
                    });
                    match simplified {
                        ParsingResult::Alt { args, accepts_empty } => {
                            new_concat_args.push(ConcatArg::Alt {
                                args: Vec::from_iter(args),
                                accepts_empty,
                            })
                        }
                        ParsingResult::Concat { args, accepts_empty } => {
                            new_concat_args.extend(args);
                        }
                        ParsingResult::Star(arg) => {
                            new_concat_args.push(ConcatArg::Star(arg))
                        }
                    }
                }
                ConcatArg::Star(arg) => {
                    let simplified = simplify(ParsingResult::from(*arg));
                    new_concat_args.push(ConcatArg::Star(Box::new(
                        StarArg::from(simplified),
                    )))
                }
                ConcatArg::Char(c) => {
                    new_concat_args.push(ConcatArg::Char(c));
                }
            });
            return ParsingResult::Concat {
                args: new_concat_args,
                accepts_empty,
            };
        }
        ParsingResult::Star(arg) => {
            let simplified = simplify(ParsingResult::from(*arg));
            return ParsingResult::Star(Box::new(StarArg::from(simplified)));
        }
    }
}
