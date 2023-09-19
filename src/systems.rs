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

pub fn generate_system(eq: &ParsedEquation) -> String {
    let ParsedEquation { lhs, rhs } = eq;
    let traversed_lhs = lhs.distribute();
    let mut traversed_rhs: TraversedExpr = rhs.distribute();

    let mut system = String::new();
    let mut strict_decreasing = String::new();

    for (lhs_var_name, lhs_coefs) in traversed_lhs.var_nodes.iter() {
        let (_, rhs_coefs) = traversed_rhs
            .var_nodes
            .remove_entry(lhs_var_name)
            .unwrap_or_else(|| (*lhs_var_name, vec![vec!["0".to_string()]]));

        let lhs_smt_coefs = smt_coefs(&lhs_coefs);
        let rhs_smt_coefs = smt_coefs(&rhs_coefs);
        system.push_str(
            format!(
                "(>= {} {})",
                lhs_smt_coefs.as_str(),
                rhs_smt_coefs.as_str()
            )
            .as_str(),
        );

        strict_decreasing.push_str(
            format!(
                "(> {} {})",
                lhs_smt_coefs.as_str(),
                rhs_smt_coefs.as_str()
            )
            .as_str(),
        )
    }
    for (_, rhs_coefs) in traversed_rhs.var_nodes.iter() {
        let lhs_coefs = vec![vec!["0".to_string()]];
        let lhs_smt_coefs = smt_coefs(&lhs_coefs);
        let rhs_smt_coefs = smt_coefs(&rhs_coefs);
        system.push_str(
            format!(
                "(>= {} {})",
                lhs_smt_coefs.as_str(),
                rhs_smt_coefs.as_str()
            )
            .as_str(),
        );

        strict_decreasing.push_str(
            format!(
                "(> {} {})",
                lhs_smt_coefs.as_str(),
                rhs_smt_coefs.as_str()
            )
            .as_str(),
        )
    }

    let lhs_smt_consts = smt_coefs(&traversed_lhs.constant);
    let rhs_smt_consts = smt_coefs(&traversed_rhs.constant);
    system.push_str(
        format!(
            "(>= {} {})",
            lhs_smt_consts.as_str(),
            rhs_smt_consts.as_str()
        )
        .as_str(),
    );
    strict_decreasing = format!(
        "(or (and {}) (> {} {}))",
        strict_decreasing, lhs_smt_consts, rhs_smt_consts
    );

    let res = format!("(and {} {})", system, strict_decreasing);
    res
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::smt_coefs;
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
}
