use std::{
    collections::{HashMap, HashSet},
    str::Chars,
};

use itertools;

#[derive(Debug, Clone)]
enum NodeValue {
    Value(Vec<Vec<String>>),
    Func(Box<FunctionNode>),
}

#[derive(Default, Debug, Clone)]
struct FunctionNode {
    name: char,
    nodes: Vec<(char, NodeValue)>,
    constant: Vec<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct TraversedExpr {
    var_nodes: HashMap<char, Vec<Vec<String>>>,
    constant: Vec<Vec<String>>,
}

impl FunctionNode {
    fn distribute(&self, mut prefix: Vec<String>) -> TraversedExpr {
        let mut res = TraversedExpr::default();
        res.constant = self
            .constant
            .iter()
            .map(|item| itertools::concat(vec![prefix.clone(), item.clone()]))
            .collect();
        for (i, (child_node, node)) in self.nodes.iter().enumerate() {
            match node {
                NodeValue::Value(value) => {
                    res.var_nodes.entry(child_node.clone()).or_default().extend(
                        value
                            .iter()
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

struct Parser<'a> {
    variables: HashSet<char>,
    next_char: Option<char>,
    rule_iter: Chars<'a>,
}

/*
EXPR ::= FUNCTION_CALL | CHAR
FUNCTION_CALL ::= ZERO_ARGS | ONE_ARG | TWO_ARGS
ZERO_ARGS ::= CHAR ("(" ")")?
ONE_ARG ::= CHAR ("(" EXPR ")")
TWO_ARGS ::= CHAR "(" EXPR "," EXPR ")"
*/

impl<'a> Parser<'a> {
    fn parse(&mut self, rule: &'a str) /*  -> (FunctionNode, FunctionNode) */
    {
        self.rule_iter = rule.chars();
        self.expect_call();
    }

    fn expect_call(&mut self) {
        let func_symbol = self.peek().unwrap();
        // if let Some(_) = self.variables.get(&func_symbol) {
        //     return
        // }
    }

    fn peek(&mut self) -> &Option<char> {
        if let None = self.next_char {
            self.next_char = self.rule_iter.next();
            loop {
                if let Some(c) = self.next_char {
                    if c.is_whitespace() {
                        self.next_char = self.rule_iter.next();
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
}

#[cfg(test)]
mod tests {
    use super::{FunctionNode, NodeValue, TraversedExpr};
    use std::{collections::HashMap, vec};

    #[test]
    fn test_propogation() {
        let left = FunctionNode {
            name: 'g',
            nodes: Vec::from([
                ('x', NodeValue::Value(vec![vec![String::from("a_g0")]])),
                ('y', NodeValue::Value(vec![vec![String::from("a_g1")]])),
            ]),
            constant: vec![vec![String::from("a_gc")]],
        };
        let root = FunctionNode {
            name: 'f',
            nodes: Vec::from([
                ('g', NodeValue::Func(Box::new(left.clone()))),
                ('g', NodeValue::Func(Box::new(left.clone()))),
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
}
