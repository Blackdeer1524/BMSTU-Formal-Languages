use std::{collections::HashSet, str::Chars};

use itertools::{self, PeekingNext};

#[derive(Debug, Clone)]
enum NodeValue {
    Value(Vec<Vec<String>>),
    Func(Box<FunctionNode>),
}

#[derive(Default, Debug, Clone)]
struct FunctionNode {
    name: char,
    x_node: Option<NodeValue>,
    y_node: Option<NodeValue>,
    constant: Vec<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct TraversedExpr {
    x_node: Vec<Vec<String>>,
    y_node: Vec<Vec<String>>,
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
        if let Some(value_enum) = &self.x_node {
            match value_enum {
                NodeValue::Value(value) => {
                    res.x_node.extend(
                        value
                            .iter()
                            .map(|item| itertools::concat(vec![prefix.clone(), item.clone()]))
                            .collect::<Vec<Vec<String>>>(),
                    );
                }
                NodeValue::Func(function) => {
                    prefix.push(format!("a_{}x", self.name));
                    let x_res = function.distribute(prefix.clone());
                    prefix.pop();
                    res.constant.extend(x_res.constant);
                    res.x_node.extend(x_res.x_node);
                    res.y_node.extend(x_res.y_node);
                }
            }
        };
        if let Some(value_enum) = &self.y_node {
            match value_enum {
                NodeValue::Value(value) => {
                    res.y_node.extend(
                        value
                            .iter()
                            .map(|item| itertools::concat(vec![prefix.clone(), item.clone()]))
                            .collect::<Vec<Vec<String>>>(),
                    );
                }
                NodeValue::Func(function) => {
                    prefix.push(format!("a_{}y", self.name));
                    let y_res = function.distribute(prefix.clone());
                    prefix.pop();
                    res.constant.extend(y_res.constant);
                    res.x_node.extend(y_res.x_node);
                    res.y_node.extend(y_res.y_node);
                }
            }
        };
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
    use std::vec;

    #[test]
    fn test_propogation() {
        let left = FunctionNode {
            name: 'g',
            x_node: Some(NodeValue::Value(vec![vec!["a_gx".to_string()]])),
            y_node: Some(NodeValue::Value(vec![vec!["a_gy".to_string()]])),
            constant: vec![vec!["a_gc".to_string()]],
        };
        let root = FunctionNode {
            name: 'f',
            x_node: Some(NodeValue::Func(Box::new(left.clone()))),
            y_node: Some(NodeValue::Func(Box::new(left.clone()))),
            constant: vec![vec!["a_fc".to_string()]],
        };

        let expected = TraversedExpr {
            x_node: vec![
                vec!["a_fx".to_string(), "a_gx".to_string()],
                vec!["a_fy".to_string(), "a_gx".to_string()],
            ],
            y_node: vec![
                vec!["a_fx".to_string(), "a_gy".to_string()],
                vec!["a_fy".to_string(), "a_gy".to_string()],
            ],
            constant: vec![
                vec!["a_fc".to_string()],
                vec!["a_fx".to_string(), "a_gc".to_string()],
                vec!["a_fy".to_string(), "a_gc".to_string()],
            ],
        };

        let res = root.distribute(vec![]);
        assert_eq!(expected, res);
    }
}
