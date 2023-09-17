use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    slice::Iter,
    str::Chars,
};

use itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeValue {
    Value { name: char, coef: Vec<Vec<String>> },
    Func(Box<FunctionNode>),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct FunctionNode {
    name: char,
    nodes: Vec<NodeValue>,
    constant: Vec<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct TraversedExpr {
    var_nodes: HashMap<char, Vec<Vec<String>>>,
    constant: Vec<Vec<String>>,
}

impl FunctionNode {
    fn new(name: char) -> FunctionNode {
        FunctionNode {
            name,
            nodes: Vec::default(),
            constant: vec![vec![format!("a_{}c", name)]],
        }
    }

    fn distribute(&self, mut prefix: Vec<String>) -> TraversedExpr {
        let mut res = TraversedExpr::default();
        res.constant = self
            .constant
            .iter()
            .map(|item| itertools::concat(vec![prefix.clone(), item.clone()]))
            .collect();
        for (i, node) in self.nodes.iter().enumerate() {
            match node {
                NodeValue::Value { name, coef } => {
                    res.var_nodes.entry(*name).or_default().extend(
                        coef.iter()
                            .map(|item| itertools::concat(vec![prefix.clone(), item.clone()]))
                            .collect::<Vec<Vec<String>>>(),
                    );
                }
                NodeValue::Func(function) => {
                    prefix.push(format!("a_{}{}", self.name, i));
                    let x_res = function.distribute(prefix.clone());
                    prefix.pop();
                    res.constant.extend(x_res.constant);
                    for (k, v) in x_res.var_nodes {
                        res.var_nodes.entry(k).or_default().extend(v);
                    }
                }
            };
        }
        res
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParsingRes {
    VarDependencies(char),
    FunctionCall(Box<FunctionNode>),
}

struct Parser<'a> {
    variables: HashSet<char>,
    next_char: Option<char>,
    rule_iter: Option<Chars<'a>>,
}

impl<'a> Parser<'a> {
    fn new(variables: HashSet<char>) -> Parser<'a> {
        Parser {
            variables,
            next_char: None,
            rule_iter: None,
        }
    }

    fn parse(&mut self, rule: &'a str) -> Option<(ParsingRes, ParsingRes)> {
        self.next_char = None;
        self.rule_iter = Some(rule.chars());
        let lhs = self.expect_call();
        if let None = self.consume('=') {
            return None;
        }
        let rhs = self.expect_call();
        return Some((lhs, rhs));
    }

    fn expect_call(&mut self) -> ParsingRes {
        let func_symbol = self.peek().unwrap();
        self.advance();
        if self.variables.contains(&func_symbol) {
            return ParsingRes::VarDependencies(func_symbol);
        }
        let mut func_node = Box::new(FunctionNode::new(func_symbol));
        if let None = self.peek() {
            panic!("Parser couldn't find any tokens");
        }
        let next = self.peek().unwrap();
        if next != '(' {
            return ParsingRes::FunctionCall(func_node);
        }
        self.advance();
        let inner_call = self.expect_call();
        func_node.nodes.push(match inner_call {
            ParsingRes::VarDependencies(c) => NodeValue::Value {
                name: c,
                coef: vec![vec![format!("a_{}{}", func_symbol, 0)]],
            },
            ParsingRes::FunctionCall(call) => NodeValue::Func(call),
        });
        let mut i: usize = 1;
        while self.assert(',') {
            self.advance();
            let inner_call = self.expect_call();
            func_node.nodes.push(match inner_call {
                ParsingRes::VarDependencies(c) => NodeValue::Value {
                    name: c,
                    coef: vec![vec![format!("a_{}{}", func_symbol, i)]],
                },
                ParsingRes::FunctionCall(call) => NodeValue::Func(call),
            });
            i += 1;
        }
        self.consume(')');

        ParsingRes::FunctionCall(func_node)
    }

    fn peek(&mut self) -> &Option<char> {
        if let None = self.next_char {
            self.next_char = self
                .rule_iter
                .as_mut()
                .expect("expected initialized chars iter")
                .next();
            loop {
                if let Some(c) = self.next_char {
                    if c.is_whitespace() {
                        self.next_char = self
                            .rule_iter
                            .as_mut()
                            .expect("expected initialized chars iter")
                            .next();
                        continue;
                    }
                }
                break;
            }
        };

        &self.next_char
    }

    fn advance(&mut self) {
        self.next_char = None;
    }

    fn consume(&mut self, expected: char) -> Option<()> {
        if let Some(n) = self.peek() {
            if n == &expected {
                self.advance();
                return Some(());
            }
        }
        None
    }

    fn assert(&mut self, expected: char) -> bool {
        if let Some(next) = self.peek() {
            if next == &expected {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::expander::ParsingRes;

    use super::{FunctionNode, NodeValue, Parser, TraversedExpr};
    use std::{
        collections::{HashMap, HashSet},
        vec,
    };

    #[test]
    fn test_propogation() {
        let left = FunctionNode {
            name: 'g',
            nodes: Vec::from([
                NodeValue::Value {
                    name: 'x',
                    coef: vec![vec![String::from("a_g0")]],
                },
                NodeValue::Value {
                    name: 'y',
                    coef: vec![vec![String::from("a_g1")]],
                },
            ]),
            constant: vec![vec![String::from("a_gc")]],
        };
        let root = FunctionNode {
            name: 'f',
            nodes: Vec::from([
                NodeValue::Func(Box::new(left.clone())),
                NodeValue::Func(Box::new(left.clone())),
            ]),
            constant: vec![vec!["a_fc".to_string()]],
        };
        let expected = TraversedExpr {
            var_nodes: HashMap::from([
                (
                    'x',
                    vec![
                        vec![String::from("a_f0"), String::from("a_g0")],
                        vec![String::from("a_f1"), String::from("a_g0")],
                    ],
                ),
                (
                    'y',
                    vec![
                        vec![String::from("a_f0"), String::from("a_g1")],
                        vec![String::from("a_f1"), String::from("a_g1")],
                    ],
                ),
            ]),
            constant: vec![
                vec![String::from("a_fc")],
                vec![String::from("a_f0"), String::from("a_gc")],
                vec![String::from("a_f1"), String::from("a_gc")],
            ],
        };

        let res = root.distribute(vec![]);
        assert_eq!(expected, res);
    }

    #[test]
    fn test_parsing() {
        let s = "f(g(x, y), z) = g(z, y)";
        let mut parser = Parser::new(HashSet::from(['x', 'y', 'z']));
        let (lhs, rhs) = parser.parse(s).unwrap();
        // println!("{:?}", lhs);
        // println!("{:?}", rhs);
        // if let ParsingRes::FunctionCall(call) = lhs {
        //     println!("{:?}", call.distribute(vec![]));
        // }
    }
}
