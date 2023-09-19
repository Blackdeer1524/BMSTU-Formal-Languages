use std::{
    collections::{HashMap, HashSet},
    usize,
};

use crate::{parsing::EquationParser, systems::generate_system};

fn parse_variables(vars_decl_line: &str) -> HashSet<char> {
    let eq_sign_index = vars_decl_line
        .find('=')
        .expect("= is expected in variables declaration.");
    let (_, variables_str) = vars_decl_line.split_at(eq_sign_index + 1);
    let vars: HashSet<char> = variables_str
        .split(',')
        .map(|item| item.trim().chars().next().unwrap())
        .collect();
    vars
}

fn declare_functions(declared_funcs: &HashMap<char, usize>) -> String {
    let mut r = String::new();
    for (fn_name, arity) in declared_funcs {
        if *arity == 0usize {
            r.push_str(format!("(declare-const a_{}c Int)", fn_name).as_str());

            // нестрогая монотонность
            r.push_str(format!("(assert  (>= a_{}c 0))\n", fn_name).as_str());
            // строгая монотонность
            r.push_str(format!("(assert  (> a_{}c 0))\n", fn_name).as_str());
        } else if *arity == 1usize {
            r.push_str(
                format!("(declare-const a_{}0 Int)\n", fn_name).as_str(),
            );
            r.push_str(
                format!("(declare-const a_{}c Int)\n", fn_name).as_str(),
            );

            // нестрогая монотонность
            r.push_str(format!("(assert  (>= a_{}0 1))\n", fn_name).as_str());
            r.push_str(format!("(assert  (>= a_{}c 0))\n", fn_name).as_str());
            // строгая монотонность
            r.push_str(
                format!(
                    "(assert (or (> a_{}0 1) (> a_{}c 0)))\n",
                    fn_name, fn_name
                )
                .as_str(),
            );
        } else if *arity == 2usize {
            r.push_str(
                format!("(declare-const a_{}0 Int)\n", fn_name).as_str(),
            );
            r.push_str(
                format!("(declare-const a_{}1 Int)\n", fn_name).as_str(),
            );
            r.push_str(
                format!("(declare-const a_{}c Int)\n", fn_name).as_str(),
            );

            // нестрогая монотонность
            r.push_str(format!("(assert  (>= a_{}0 1))\n", fn_name).as_str());
            r.push_str(format!("(assert  (>= a_{}1 1))\n", fn_name).as_str());
            r.push_str(format!("(assert  (>= a_{}c 0))\n", fn_name).as_str());
            // строгая монотонность
            r.push_str(
                format!(
                    "(assert (or (and (> a_{}0 1) (> a_{}1 1)) (> a_{}c 0)))\n",
                    fn_name, fn_name, fn_name
                )
                .as_str(),
            );
        } else {
            panic!("Unexpected arity of {}: {}", fn_name, arity);
        }
        r.push('\n');
    }
    r
}

pub fn generate(input: &str) -> String {
    let mut lines_iter = input.lines().filter(|line| !line.trim().is_empty());
    let vars_decl_line = lines_iter
        .next()
        .expect("Variables declaration are expected.");
    let variables = parse_variables(vars_decl_line);

    let mut declared_functions: HashMap<char, usize> = HashMap::default();
    let mut parser = EquationParser::new(variables, &mut declared_functions);

    let mut systems = String::new();
    for line in lines_iter {
        let parsed_eq = parser.parse(line).expect("Valid rewrite is expected");
        let system = generate_system(&parsed_eq);
        systems.push_str(format!("(assert {})\n", system).as_str());
    }
    let declarations = declare_functions(parser.declared_functions);

    format!(
        r#"
(set-logic QF_NIA)
{}
{}
(check-sat)
(get-model)
"#,
        declarations, systems
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{generate, parse_variables};

    #[test]
    fn test_vars_parsing() {
        let t = "variables = x, y, z";
        let res = parse_variables(t);
        let expected = HashSet::from(['x', 'y', 'z']);
        assert_eq!(expected, res);
    }

    #[test]
    fn test_pipeline() {
        let t = r#"
        variables = x, y
        f(g(x, y)) = g(h(y), x)
        "#;

        let res = generate(t);
        println!("{}", res);
    }
}
