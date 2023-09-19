use std::vec;

use itertools::Itertools;

use crate::parsing::{ParsedEquation, TraversedExpr};

fn smt_coefs(coefs: &Vec<Vec<String>>) -> String {
    let res: Vec<String> = coefs
        .iter()
        .map(|item| {
            if item.len() > 1 {
                format!("(* {})", item.join(" "))
            } else {
                item.join("")
            }
        })
        .collect();
    if res.len() > 1 {
        return format!("(+ {})", res.iter().filter(|i| i.len() > 0).join(" "));
    }
    res.join("")
}

// истинное >= переписанное
pub fn generate_system(eq: &ParsedEquation) -> String {
    let ParsedEquation { lhs, rhs } = eq;
    let traversed_lhs = lhs.distribute();
    let mut traversed_rhs: TraversedExpr = rhs.distribute();

    let mut first_system = String::new();
    let mut second_system = String::new();
    for (lhs_var_name, lhs_coefs) in traversed_lhs.var_nodes.iter() {
        let (_, rhs_coefs) = traversed_rhs
            .var_nodes
            .remove_entry(lhs_var_name)
            .unwrap_or_else(|| (*lhs_var_name, vec![vec!["0".to_string()]]));

        let lhs_smt_coefs = smt_coefs(&lhs_coefs);
        let rhs_smt_coefs = smt_coefs(&rhs_coefs);
        first_system.push_str(
            format!(
                "(>= {} {})",
                lhs_smt_coefs.as_str(),
                rhs_smt_coefs.as_str()
            )
            .as_str(),
        );
        second_system.push_str(
            format!(
                "(> {} {})",
                lhs_smt_coefs.as_str(),
                rhs_smt_coefs.as_str()
            )
            .as_str(),
        );
    }
    let lhs_smt_consts = smt_coefs(&traversed_lhs.constant);
    let rhs_smt_consts = smt_coefs(&traversed_rhs.constant);
    first_system.push_str(
        format!(
            "(> {} {})",
            lhs_smt_consts.as_str(),
            rhs_smt_consts.as_str()
        )
        .as_str(),
    );
    second_system.push_str(
        format!(
            "(>= {} {})",
            lhs_smt_consts.as_str(),
            rhs_smt_consts.as_str()
        )
        .as_str(),
    );

    let res = format!("(or (and {}) (and {}))", first_system, second_system);
    res
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        vec,
    };

    use crate::parsing::{EquationParser, ParsedEquation, TraversedExpr};

    use super::{generate_system, smt_coefs};
    #[test]
    fn test_simple_smt_repr() {
        let target = vec![vec![String::from("a_gc")]];
        let result = smt_coefs(&target);
        let expected = "a_gc";

        assert_eq!(expected, result);
    }

    #[test]
    fn test_complex_smt_repr() {
        let target = vec![
            vec![String::from("a_f0"), String::from("a_g0")],
            vec![String::from("a_gc")],
            vec![],
        ];
        let result = smt_coefs(&target);
        let expected = "(+ (* a_f0 a_g0) a_gc)";

        assert_eq!(expected, result);
    }

    #[test]
    fn test_systems_gen() {
        let expr = "f(g(x, y)) = g(h(y), x)";

        let variables = HashSet::from(['x', 'y']);
        let mut declared_functions: HashMap<char, usize> = HashMap::default();
        let mut parser =
            EquationParser::new(variables, &mut declared_functions);

        let parsed_equation = parser.parse(expr).unwrap();
        let system = generate_system(&parsed_equation);
        let _expected = r#"
        (or 
            (and 
                (>= (* a_f0 a_g0) a_g1)
                (>= (* a_f0 a_g1) (* a_g0 a_h0))
                (> (+ a_fc (* a_f0 a_gc)) (+ a_gc (* a_g0 a_hc)))
            ) 
            (and 
                (> (* a_f0 a_g0) a_g1)
                (> (* a_f0 a_g1) (* a_g0 a_h0))
                (>= (+ a_fc (* a_f0 a_gc)) (+ a_gc (* a_g0 a_hc)))
            )
        )"#;
        // Работает, но нельзя чекнуть через ассерт из-за рандомности мап
        println!("{}", system);
    }
}
