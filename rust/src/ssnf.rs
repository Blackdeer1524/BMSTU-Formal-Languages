use super::parser::{AltArg, ConcatArg, ParsingResult, StarArg};

fn ssnf_alt(arg: &mut AltArg) {
    match arg {
        AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            *body_accepts_empty = true;
            *tail_accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ssnf_concat(item);
                *body_accepts_empty &= *tail_accepts_empty;
                match item {
                    ConcatArg::Concat {
                        args,
                        body_accepts_empty: inner_body_accepts_empty,
                        tail_accepts_empty: inner_tail_accepts_empty,
                    } => {
                        *tail_accepts_empty = *inner_body_accepts_empty
                            && *inner_tail_accepts_empty;
                    }
                    ConcatArg::Alt { args, accepts_empty } => {
                        *tail_accepts_empty = *accepts_empty;
                    }
                    ConcatArg::Star(_) => {
                        *tail_accepts_empty = true;
                    }
                    ConcatArg::Regex { arg, parenthesized } => {
                        *tail_accepts_empty = false;
                    }
                }
            });
        }
        AltArg::Alt { args, accepts_empty } => {
            *accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ssnf_alt(item);
                match item {
                    AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        *accepts_empty &=
                            *body_accepts_empty && *tail_accepts_empty;
                    }
                    AltArg::Alt {
                        args,
                        accepts_empty: inner_accepts_empty,
                    } => {
                        *accepts_empty &= *inner_accepts_empty;
                    }
                    AltArg::Star(_) => (),
                    AltArg::Regex { arg, parenthesized } => {
                        *accepts_empty = false;
                    }
                }
            });
            args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            })
        }
        AltArg::Star(star_arg) => {
            ss_star(star_arg);
        }
        AltArg::Regex { arg, parenthesized } => (),
    }
}

fn ssnf_concat(arg: &mut ConcatArg) {
    match arg {
        ConcatArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            *body_accepts_empty = true;
            *tail_accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ssnf_concat(item);
                *body_accepts_empty &= *tail_accepts_empty;
                match item {
                    ConcatArg::Concat {
                        args,
                        body_accepts_empty: inner_body_accepts_empty,
                        tail_accepts_empty: inner_tail_accepts_empty,
                    } => {
                        *tail_accepts_empty = *inner_body_accepts_empty
                            && *inner_tail_accepts_empty;
                    }
                    ConcatArg::Alt { args, accepts_empty } => {
                        *tail_accepts_empty = *accepts_empty;
                    }
                    ConcatArg::Star(_) => {
                        *tail_accepts_empty = true;
                    }
                    ConcatArg::Regex { arg, parenthesized } => {
                        *tail_accepts_empty = false;
                    }
                }
            });
        }
        ConcatArg::Alt { args, accepts_empty } => {
            *accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ssnf_alt(item);
                match item {
                    AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        *accepts_empty &=
                            *body_accepts_empty && *tail_accepts_empty;
                    }
                    AltArg::Alt {
                        args,
                        accepts_empty: inner_accepts_empty,
                    } => {
                        *accepts_empty &= *inner_accepts_empty;
                    }
                    AltArg::Star(_) => (),
                    AltArg::Regex { arg, parenthesized } => {
                        *accepts_empty = false;
                    }
                }
            });
            args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            })
        }
        ConcatArg::Star(star_arg) => {
            ss_star(star_arg);
        }
        ConcatArg::Regex { arg, parenthesized } => (),
    }
}

fn ss_star(arg: &mut StarArg) {
    match arg {
        StarArg::Alt { args, accepts_empty } => {
            *accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ss_alt(item);
                match item {
                    AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        *accepts_empty &=
                            *body_accepts_empty && *tail_accepts_empty;
                    }
                    AltArg::Alt {
                        args,
                        accepts_empty: inner_accepts_empty,
                    } => {
                        *accepts_empty &= *inner_accepts_empty;
                    }
                    AltArg::Star(_) => (),
                    AltArg::Regex { arg, parenthesized } => {
                        *accepts_empty = false;
                    }
                }
            });
            args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            })
        }
        StarArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            if *body_accepts_empty && *tail_accepts_empty {
                let mut new_accepts_empty = true;
                let new_args: Vec<AltArg> = args
                    .iter_mut()
                    .map(|item| {
                        ss_concat(item);
                        match item {
                            ConcatArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            } => {
                                new_accepts_empty &=
                                    *body_accepts_empty && *tail_accepts_empty;
                                AltArg::Concat {
                                    args: args.clone(),
                                    body_accepts_empty: *body_accepts_empty,
                                    tail_accepts_empty: *tail_accepts_empty,
                                }
                            }
                            ConcatArg::Alt { args, accepts_empty } => {
                                new_accepts_empty &= *accepts_empty;
                                AltArg::Alt {
                                    args: args.clone(),
                                    accepts_empty: *accepts_empty,
                                }
                            }
                            ConcatArg::Star(arg) => AltArg::Star(arg.clone()),
                            ConcatArg::Regex { arg, parenthesized } => {
                                new_accepts_empty = false;
                                AltArg::Regex {
                                    arg: arg.clone(),
                                    parenthesized: *parenthesized,
                                }
                            }
                        }
                    })
                    .collect();
                args.sort_unstable_by(|left, right| {
                    left.to_string().cmp(&right.to_string())
                });
                *arg = StarArg::Alt {
                    args: new_args,
                    accepts_empty: new_accepts_empty,
                }
            } else {
                let mut new_tail_accepts_empty = true;
                let mut new_body_accepts_empty = true;
                args.iter_mut().for_each(|item| {
                    ssnf_concat(item);
                    new_body_accepts_empty &= new_tail_accepts_empty;
                    match item {
                        ConcatArg::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        } => {
                            new_tail_accepts_empty =
                                *body_accepts_empty && *tail_accepts_empty;
                        }
                        ConcatArg::Alt { args, accepts_empty } => {
                            new_tail_accepts_empty = *accepts_empty;
                        }
                        ConcatArg::Star(_) => {
                            new_tail_accepts_empty = true;
                        }
                        ConcatArg::Regex { arg, parenthesized } => {
                            new_tail_accepts_empty = false;
                        }
                    }
                });
            }
        }
        StarArg::Regex(_) => (),
    }
}

fn ss_alt(arg: &mut AltArg) {
    match arg {
        AltArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            if *body_accepts_empty && *tail_accepts_empty {
                let mut new_accepts_empty = true;
                let new_args: Vec<AltArg> = args
                    .iter_mut()
                    .map(|item| {
                        ss_concat(item);
                        match item {
                            ConcatArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            } => {
                                new_accepts_empty &=
                                    *body_accepts_empty && *tail_accepts_empty;
                                AltArg::Concat {
                                    args: args.clone(),
                                    body_accepts_empty: *body_accepts_empty,
                                    tail_accepts_empty: *tail_accepts_empty,
                                }
                            }
                            ConcatArg::Alt { args, accepts_empty } => {
                                new_accepts_empty &= *accepts_empty;
                                AltArg::Alt {
                                    args: args.clone(),
                                    accepts_empty: *accepts_empty,
                                }
                            }
                            ConcatArg::Star(arg) => AltArg::Star(arg.clone()),
                            ConcatArg::Regex { arg, parenthesized } => {
                                new_accepts_empty = false;
                                AltArg::Regex {
                                    arg: arg.clone(),
                                    parenthesized: *parenthesized,
                                }
                            }
                        }
                    })
                    .collect();
                args.sort_unstable_by(|left, right| {
                    left.to_string().cmp(&right.to_string())
                });
                *arg = AltArg::Alt {
                    args: new_args,
                    accepts_empty: new_accepts_empty,
                }
            } else {
                let mut new_tail_accepts_empty = true;
                let mut new_body_accepts_empty = true;
                args.iter_mut().for_each(|item| {
                    ssnf_concat(item);
                    new_body_accepts_empty &= new_tail_accepts_empty;
                    match item {
                        ConcatArg::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        } => {
                            new_tail_accepts_empty =
                                *body_accepts_empty && *tail_accepts_empty;
                        }
                        ConcatArg::Alt { args, accepts_empty } => {
                            new_tail_accepts_empty = *accepts_empty;
                        }
                        ConcatArg::Star(_) => {
                            new_tail_accepts_empty = true;
                        }
                        ConcatArg::Regex { arg, parenthesized } => {
                            new_tail_accepts_empty = false;
                        }
                    }
                });
            }
        }
        AltArg::Alt { args, accepts_empty } => {
            *accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ss_alt(item);
                match item {
                    AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        *accepts_empty &=
                            *body_accepts_empty && *tail_accepts_empty;
                    }
                    AltArg::Alt {
                        args,
                        accepts_empty: inner_accepts_empty,
                    } => {
                        *accepts_empty &= *inner_accepts_empty;
                    }
                    AltArg::Star(_) => (),
                    AltArg::Regex { arg, parenthesized } => {
                        *accepts_empty = false;
                    }
                }
            });
            args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            })
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
        AltArg::Regex { arg, parenthesized } => (),
    }
}

fn ss_concat(arg: &mut ConcatArg) {
    match arg {
        ConcatArg::Concat { args, body_accepts_empty, tail_accepts_empty } => {
            if *body_accepts_empty && *tail_accepts_empty {
                let mut new_accepts_empty = true;
                let new_args: Vec<AltArg> = args
                    .iter_mut()
                    .map(|item| {
                        ss_concat(item);
                        match item {
                            ConcatArg::Concat {
                                args,
                                body_accepts_empty,
                                tail_accepts_empty,
                            } => {
                                new_accepts_empty &=
                                    *body_accepts_empty && *tail_accepts_empty;
                                AltArg::Concat {
                                    args: args.clone(),
                                    body_accepts_empty: *body_accepts_empty,
                                    tail_accepts_empty: *tail_accepts_empty,
                                }
                            }
                            ConcatArg::Alt { args, accepts_empty } => {
                                new_accepts_empty &= *accepts_empty;
                                AltArg::Alt {
                                    args: args.clone(),
                                    accepts_empty: *accepts_empty,
                                }
                            }
                            ConcatArg::Star(arg) => AltArg::Star(arg.clone()),
                            ConcatArg::Regex { arg, parenthesized } => {
                                new_accepts_empty = false;
                                AltArg::Regex {
                                    arg: arg.clone(),
                                    parenthesized: *parenthesized,
                                }
                            }
                        }
                    })
                    .collect();
                args.sort_unstable_by(|left, right| {
                    left.to_string().cmp(&right.to_string())
                });
                *arg = ConcatArg::Alt {
                    args: new_args,
                    accepts_empty: new_accepts_empty,
                }
            } else {
                let mut new_tail_accepts_empty = true;
                let mut new_body_accepts_empty = true;
                args.iter_mut().for_each(|item| {
                    ssnf_concat(item);
                    new_body_accepts_empty &= new_tail_accepts_empty;
                    match item {
                        ConcatArg::Concat {
                            args,
                            body_accepts_empty,
                            tail_accepts_empty,
                        } => {
                            new_tail_accepts_empty =
                                *body_accepts_empty && *tail_accepts_empty;
                        }
                        ConcatArg::Alt { args, accepts_empty } => {
                            new_tail_accepts_empty = *accepts_empty;
                        }
                        ConcatArg::Star(_) => {
                            new_tail_accepts_empty = true;
                        }
                        ConcatArg::Regex { arg, parenthesized } => {
                            new_tail_accepts_empty = false;
                        }
                    }
                });
            }
        }
        ConcatArg::Alt { args, accepts_empty } => {
            *accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ss_alt(item);
                match item {
                    AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        *accepts_empty &=
                            *body_accepts_empty && *tail_accepts_empty;
                    }
                    AltArg::Alt {
                        args,
                        accepts_empty: inner_accepts_empty,
                    } => {
                        *accepts_empty &= *inner_accepts_empty;
                    }
                    AltArg::Star(_) => (),
                    AltArg::Regex { arg, parenthesized } => {
                        *accepts_empty = false;
                    }
                }
            });
            args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            })
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

pub fn apply_ssnf(root: &mut ParsingResult) {
    match root {
        ParsingResult::Alt { args, accepts_empty } => {
            *accepts_empty = true;
            args.iter_mut().for_each(|item| {
                ssnf_alt(item);
                match item {
                    AltArg::Concat {
                        args,
                        body_accepts_empty,
                        tail_accepts_empty,
                    } => {
                        *accepts_empty &=
                            *body_accepts_empty && *tail_accepts_empty;
                    }
                    AltArg::Alt {
                        args,
                        accepts_empty: inner_accepts_empty,
                    } => {
                        *accepts_empty &= *inner_accepts_empty;
                    }
                    AltArg::Star(_) => (),
                    AltArg::Regex { arg, parenthesized } => {
                        *accepts_empty = false;
                    }
                }
            });
            args.sort_unstable_by(|left, right| {
                left.to_string().cmp(&right.to_string())
            })
        }
        ParsingResult::Concat {
            args,
            body_accepts_empty,
            tail_accepts_empty,
            parenthesized,
        } => {
            args.iter_mut().for_each(ssnf_concat);
        }
        ParsingResult::Star(star_arg) => {
            ss_star(star_arg);
        }
        ParsingResult::Regex { arg, parenthesized } => (),
    }
}
