use std::ops::Not;

use crate::{
    cnf::{Literal, OwnedCNF},
    solver::{assignment::Assignment, cnf::PartiallyAssignedCNF},
};

pub mod assignment;
pub mod cnf;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Satisfiable,
    Unsatisfiable,
}

pub fn solve(cnf: OwnedCNF) -> Outcome {
    let cnf = PartiallyAssignedCNF::new(cnf);
    dpll(cnf, Assignment::default())
}

fn dpll(mut cnf: PartiallyAssignedCNF, mut assignment: Assignment) -> Outcome {
    // Pure Literals
    let pure_literals = cnf.get_pure_literals();

    for pure_literal in pure_literals {
        assignment += pure_literal;
        cnf.remove_clauses_containing(pure_literal);
    }

    // Unit Propagation
    loop {
        let unit_clauses = cnf.get_unit_clauses();

        if unit_clauses.is_empty() {
            break;
        } else {
            for unit_clause_literal in unit_clauses {
                assignment += unit_clause_literal;
                cnf.assign(unit_clause_literal);
            }
        }
    }

    if let Some(outcome) = sat(&cnf) {
        return outcome;
    }

    let next_literal = get_next_literal(&cnf, &assignment);

    let first_outcome = {
        let mut assignment_with_pos_literal = assignment.clone();
        assignment_with_pos_literal += next_literal;
        let mut cnf_with_pos_literal = cnf.as_borrowing();
        cnf_with_pos_literal.assign(next_literal);

        dpll(cnf_with_pos_literal, assignment_with_pos_literal)
    };

    if first_outcome == Outcome::Satisfiable {
        return Outcome::Satisfiable;
    }

    let second_outcome = {
        let mut assignment_with_neg_literal = assignment;
        assignment_with_neg_literal += next_literal.not();
        cnf.assign(next_literal.not());

        dpll(cnf, assignment_with_neg_literal)
    };

    // no need to check first outcome again
    second_outcome
}

fn get_next_literal(cnf: &PartiallyAssignedCNF, assignment: &Assignment) -> Literal {
    let Some(variable) = cnf
        .iter_variables()
        .find(|variable| !assignment.contains_variable(*variable))
    else {
        unreachable!("because the assignment is full, the solver should have returned already")
    };

    Literal::new_from_variable(variable, false)
}

fn sat(formula: &PartiallyAssignedCNF) -> Option<Outcome> {
    if formula.is_satisfied() {
        return Some(Outcome::Satisfiable);
    }

    if formula.is_contradicting() {
        return Some(Outcome::Unsatisfiable);
    }

    None
}
