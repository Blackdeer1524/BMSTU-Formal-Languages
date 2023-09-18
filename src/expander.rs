use std::{
    collections::{HashMap, HashSet},
    str::Chars,
};

use itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Value { name: char, coef: Vec<Vec<String>> },
    Func(Box<FunctionNode>),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct FunctionNode {
    name: char,
    nodes: Vec<Node>,
    constant: Vec<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TraversedExpr {
    pub var_nodes: HashMap<char, Vec<Vec<String>>>,
    pub constant: Vec<Vec<String>>,
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
                Node::Value { name, coef } => {
                    res.var_nodes.entry(*name).or_default().extend(
                        coef.iter()
                            .map(|item| {
                                itertools::concat(vec![
                                    prefix.clone(),
                                    item.clone(),
                                ])
                            })
                            .collect::<Vec<Vec<String>>>(),
                    );
                }
                Node::Func(function) => {
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
pub enum ParsedTerm {
    Variable(char),
    FunctionCall(Box<FunctionNode>),
}

impl ParsedTerm {
    pub(crate) fn distribute(&self) -> TraversedExpr {
        match &self {
            ParsedTerm::Variable(c) => TraversedExpr {
                var_nodes: HashMap::from([(*c, vec![vec![String::from("1")]])]),
                constant: vec![],
            },
            ParsedTerm::FunctionCall(boxed_func) => {
                boxed_func.distribute(vec![])
            }
        }
    }
}

pub struct EquationParser<'a, 'b: 'a> {
    variables: HashSet<char>,
    next_char: Option<char>,
    rule_iter: Option<Chars<'a>>,
    pub declared_functions: &'b mut HashMap<char, usize>,
}

pub struct ParsedEquation {
    pub lhs: ParsedTerm,
    pub rhs: ParsedTerm,
}

impl<'a, 'b: 'a> EquationParser<'a, 'b> {
    pub fn new(
        variables: HashSet<char>,
        declared_functions: &'b mut HashMap<char, usize>,
    ) -> EquationParser<'a, 'b> {
        EquationParser {
            variables,
            next_char: None,
            rule_iter: None,
            declared_functions,
        }
    }

    pub fn parse(&mut self, equation: &'a str) -> Option<ParsedEquation> {
        self.next_char = None;
        self.rule_iter = Some(equation.chars());
        let lhs = self.expect_call();
        if let None = self.consume('=') {
            return None;
        }
        let rhs = self.expect_call();
        return Some(ParsedEquation { lhs, rhs });
    }

    fn expect_call(&mut self) -> ParsedTerm {
        let func_symbol = self.peek().unwrap();
        self.advance();
        if self.variables.contains(&func_symbol) {
            return ParsedTerm::Variable(func_symbol);
        }
        let mut func_node = Box::new(FunctionNode::new(func_symbol));
        if let None = self.peek() {
            panic!("Parser couldn't find any tokens");
        }
        let next = self.peek().unwrap();
        if next != '(' {
            self.declared_functions.insert(func_symbol, 0);
            return ParsedTerm::FunctionCall(func_node);
        }
        self.advance();
        let next = self.peek().unwrap();
        if next == ')' {
            self.advance();
            self.declared_functions.insert(func_symbol, 0);
            return ParsedTerm::FunctionCall(func_node);
        }

        let inner_call = self.expect_call();
        func_node.nodes.push(match inner_call {
            ParsedTerm::Variable(c) => Node::Value {
                name: c,
                coef: vec![vec![format!("a_{}{}", func_symbol, 0)]],
            },
            ParsedTerm::FunctionCall(call) => Node::Func(call),
        });
        let mut i: usize = 1;
        while self.assert(',') {
            self.advance();
            let inner_call = self.expect_call();
            func_node.nodes.push(match inner_call {
                ParsedTerm::Variable(c) => Node::Value {
                    name: c,
                    coef: vec![vec![format!("a_{}{}", func_symbol, i)]],
                },
                ParsedTerm::FunctionCall(call) => Node::Func(call),
            });
            i += 1;
        }
        self.consume(')');

        self.declared_functions.insert(func_symbol, i);
        ParsedTerm::FunctionCall(func_node)
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
    use crate::expander::{ParsedEquation, ParsedTerm};

    use super::{EquationParser, FunctionNode, Node, TraversedExpr};
    use std::{
        collections::{HashMap, HashSet},
        vec,
    };

    #[test]
    fn test_propogation() {
        // f(g(x, y), g(x, y))
        let child = FunctionNode {
            name: 'g',
            nodes: Vec::from([
                Node::Value {
                    name: 'x',
                    coef: vec![vec![String::from("a_g0")]],
                },
                Node::Value {
                    name: 'y',
                    coef: vec![vec![String::from("a_g1")]],
                },
            ]),
            constant: vec![vec![String::from("a_gc")]],
        };
        let root = FunctionNode {
            name: 'f',
            nodes: Vec::from([
                Node::Func(Box::new(child.clone())),
                Node::Func(Box::new(child.clone())),
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
        let mut declared_functions: HashMap<char, usize> = HashMap::new();

        let mut parser = EquationParser::new(
            HashSet::from(['x', 'y', 'z']),
            &mut declared_functions,
        );
        let ParsedEquation { lhs, rhs } = parser.parse(s).unwrap();
        let expected_lhs = ParsedTerm::FunctionCall(Box::new(FunctionNode {
            name: 'f',
            nodes: vec![
                Node::Func(Box::new(FunctionNode {
                    name: 'g',
                    nodes: vec![
                        Node::Value {
                            name: 'x',
                            coef: vec![vec![String::from("a_g0")]],
                        },
                        Node::Value {
                            name: 'y',
                            coef: vec![vec![String::from("a_g1")]],
                        },
                    ],
                    constant: vec![vec![String::from("a_gc")]],
                })),
                Node::Value {
                    name: 'z',
                    coef: vec![vec![String::from("a_f1")]],
                },
            ],
            constant: vec![vec![String::from("a_fc")]],
        }));
        assert_eq!(expected_lhs, lhs);

        let expected_rhs = ParsedTerm::FunctionCall(Box::new(FunctionNode {
            name: 'g',
            nodes: vec![
                Node::Value {
                    name: 'z',
                    coef: vec![vec![String::from("a_g0")]],
                },
                Node::Value {
                    name: 'y',
                    coef: vec![vec![String::from("a_g1")]],
                },
            ],
            constant: vec![vec![String::from("a_gc")]],
        }));
        assert_eq!(expected_rhs, rhs);

        let expected_declared_functions =
            HashMap::from([('f', 2usize), ('g', 2usize)]);
        assert_eq!(expected_declared_functions, declared_functions);
    }
}
