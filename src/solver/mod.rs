use std::{num::NonZeroU32, ops::Not};

use crate::{Assignment, CNF, Literal, Outcome, Variable};

pub fn solve(cnf: CNF) -> Outcome {
    dpll(cnf, Assignment::default())
}

fn dpll(formula: CNF, mut assignment: Assignment) -> Outcome {
    let before_propagation = assignment.len();
    loop {
        let unit_clauses = formula.get_unit_clauses(&assignment);

        if unit_clauses.is_empty() {
            break;
        } else {
            for unit_clause_literal in unit_clauses {
                assignment += unit_clause_literal;
            }
        }
    }
    println!(
        "Propagated {} assignments",
        assignment.len() - before_propagation
    );

    if let Some(outcome) = sat(formula.clone(), &assignment) {
        return outcome;
    }

    let next_literal = get_next_literal(&formula, &assignment);

    let mut assignment2 = assignment.clone();
    assignment += next_literal;
    assignment2 += !next_literal;

    println!("Guessing {:?}", next_literal);
    let first = dpll(formula.clone(), assignment2);
    println!("Guessing {:?}", !next_literal);
    let second = dpll(formula, assignment);
    first | second
}

fn get_next_literal(formula: &CNF, assignment: &Assignment) -> Literal {
    for variable_index in 1..=formula.num_variables {
        let variable_index = NonZeroU32::new(variable_index).expect("this to never be zero");
        let variable = Variable::new(variable_index).expect("TODO make this panic impossible");
        if !assignment.contains_variable(variable) {
            return Literal::new_from_variable(variable, false);
        }
    }

    unreachable!("because the assignment is full, the solver should have returned already")
}

fn sat(mut formula: CNF, assignment: &Assignment) -> Option<Outcome> {
    for literal in assignment.iter_literals() {
        formula.remove_clauses_containing(literal);
    }

    if formula.is_empty() {
        return Some(Outcome::Satisfiable);
    }

    for literal in assignment.iter_literals() {
        formula.remove_from_clauses(literal.not());
    }

    if formula.contains_empty_clause() {
        return Some(Outcome::Unsatisfiable);
    }

    None
}
