use std::{num::NonZeroU32, ops::Not};

use crate::{Assignment, CNF, Literal, Outcome, Variable};

pub fn solve(cnf: CNF) -> Outcome {
    dpll(cnf, Assignment::default())
}

fn dpll(mut formula: CNF, mut assignment: Assignment) -> Outcome {
    // Pure Literals
    let pure_literals = formula.get_pure_literals();
    if !pure_literals.is_empty() {
        println!("found {} pure literals", pure_literals.len());
    }

    for pure_literal in pure_literals {
        assignment += pure_literal;
    }

    // Unit Propagation
    // let before_propagation = assignment.len();
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
    // println!(
    //     "Propagated {} assignments",
    //     assignment.len() - before_propagation
    // );

    if let Some(outcome) = sat(&mut formula, &assignment) {
        return outcome;
    }

    let next_literal = get_next_literal(&formula, &assignment);

    let mut assignment_with_pos_literal = assignment.clone();
    assignment_with_pos_literal += next_literal;
    let mut assignment_with_neg_literal = assignment;
    assignment_with_neg_literal += next_literal.not();

    // println!("Guessing {:?}", next_literal);
    let pos_literal_outcome = dpll(formula.clone(), assignment_with_pos_literal);

    // println!("Guessing {:?}", !next_literal);
    let neg_literal_outcome = dpll(formula, assignment_with_neg_literal);

    pos_literal_outcome | neg_literal_outcome
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

fn sat(formula: &mut CNF, assignment: &Assignment) -> Option<Outcome> {
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
