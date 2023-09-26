use std::{fmt::format, str::Chars, vec};

enum Operation {
    Concat(Vec<OperationArgument>),
    Star(Box<OperationArgument>),
    Alternative(Vec<OperationArgument>),
}

enum OperationArgument {
    Operation(Operation),
    RawRegex(String),
}

#[derive(Clone)]
enum NodeVariant {
    Node(Node),
    Regex(String),
}

#[derive(Clone)]
struct Node {
    children: Vec<NodeVariant>,
    operation: Operation,
}

struct Parser<'a> {
    regex_iter: Option<Chars<'a>>,
    next_char: Option<char>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn parse(&mut self, regex: &'a str) {
        self.index = 0;
        self.next_char = None;
        self.regex_iter = Some(regex.chars());
    }

    fn expect_regex(&mut self) -> NodeVariant {
        let mut cur_regex = String::new();
        let mut current_node_opt: Option<NodeVariant> = None;
        loop {
            let next_opt = self.peek();
            if next_opt.is_none() {
                break;
            }
            let next = next_opt.unwrap();
            match next {
                '(' => {
                    self.advance();
                    let subexpr = self.expect_alternative();
                    self.consume(')');

                    if current_node_opt.is_none() {
                        if cur_regex.is_empty() {
                            current_node_opt = Some(subexpr);
                        } else {
                            current_node_opt = Some(NodeVariant::Node(Node {
                                children: vec![NodeVariant::Regex(cur_regex)],
                                operation: Operation::Concat,
                            }));
                            cur_regex = String::new();
                        }
                    } else {
                        let node_variant = current_node_opt.as_mut().unwrap();
                        match node_variant {
                            NodeVariant::Node(node) => {
                                let last = node
                                    .children
                                    .pop()
                                    .expect("Nodes have at least one child");
                                node.children.push(NodeVariant::Node(Node {
                                    children: vec![last, subexpr],
                                    operation: Operation::Concat,
                                }));
                            }
                            NodeVariant::Regex(expr) => {
                                current_node_opt =
                                    Some(NodeVariant::Node(Node {
                                        children: vec![
                                            NodeVariant::Regex(expr.clone()),
                                            subexpr,
                                        ],
                                        operation: Operation::Concat,
                                    }))
                            }
                        }
                    }
                }
                ')' => {
                    break;
                }
                '|' => {
                    break;
                }
                '*' => {
                    if current_node_opt.is_none() {
                        self.report("star operation on empty expression");
                    }
                    let node_variant = current_node_opt.as_mut().unwrap();
                    match node_variant {
                        NodeVariant::Node(node) => {
                            let last = node
                                .children
                                .pop()
                                .expect("Nodes have at least one child");
                            node.children.push(NodeVariant::Node(Node {
                                children: vec![last],
                                operation: Operation::Star,
                            }));
                        }
                        NodeVariant::Regex(regex) => {
                            current_node_opt = Some(NodeVariant::Node(Node {
                                children: vec![NodeVariant::Regex(
                                    regex.clone(),
                                )],
                                operation: Operation::Star,
                            }))
                        }
                    }
                }
                c => {
                    cur_regex.push(c);
                }
            }
        }
        current_node_opt.expect("expected to parse anything")
    }

    fn report(&self, message: &str) -> ! {
        panic!("[col {}] {}", self.index, message);
    }

    fn expect_alternative(&mut self) -> NodeVariant {
        let node = self.expect_regex();
        if !self.expect('|') {
            return node;
        }
        self.advance();

        let mut children = vec![node];
        loop {
            let subexpr = self.expect_regex();
            children.push(subexpr);
            if !self.expect('|') {
                break;
            }
            self.advance();
        }

        NodeVariant::Node(Node {
            children,
            operation: Operation::Alternative,
        })
    }

    fn peek(&mut self) -> Option<char> {
        if self.next_char.is_none() {
            self.next_char = self
                .regex_iter
                .as_mut()
                .expect("expeted initialized regex iter")
                .next();
            self.index += 1;
        }
        self.next_char
    }

    fn advance(&mut self) {
        if self.next_char.is_none() {
            self.regex_iter
                .as_mut()
                .expect("expeted initialized regex iter")
                .next();
            self.index += 1;
        } else {
            self.next_char = None;
        }
    }

    fn consume(&mut self, expected: char) {
        if let Some(next) = self.peek() {
            if next != expected {
                self.report(
                    format!("expected {}, but {} found", expected, next)
                        .as_str(),
                );
            }
            self.advance();
        } else {
            self.report(
                format!("expected {}, but EOF found", expected).as_str(),
            );
        }
    }

    fn expect(&mut self, expected: char) -> bool {
        let next_opt = self.peek();
        if let Some(next) = next_opt {
            expected == next
        } else {
            false
        }
    }
}
