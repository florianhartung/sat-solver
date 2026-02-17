use std::{collections::HashMap, num::NonZeroU32, ops::Not};

use crate::{
    cnf::{CNF, Clause, Literal, Variable},
    solver::{assignment::Assignment, outcome::Outcome},
};

pub mod assignment;
pub mod outcome;

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

impl CNF {
    fn remove_clauses_containing(&mut self, literal: Literal) {
        self.clauses.retain(|clause| !clause.contains(literal));
    }

    fn remove_from_clauses(&mut self, literal: Literal) {
        for clause in &mut self.clauses {
            clause.remove(literal);
        }
    }

    fn get_unit_clauses(&self, assignment: &Assignment) -> Vec<Literal> {
        let mut cloned_formula = self.clone();

        for literal in assignment.iter_literals() {
            cloned_formula.remove_clauses_containing(literal);
        }

        if cloned_formula.is_empty() {
            return Vec::new();
        }

        for literal in assignment.iter_literals() {
            cloned_formula.remove_from_clauses(literal.not());
        }

        cloned_formula
            .clauses
            .into_iter()
            .filter_map(|clause| clause.as_unit_clause())
            .collect()
    }

    fn get_pure_literals(&self) -> Vec<Literal> {
        let mut pos_neg_by_variable = HashMap::<Variable, (bool, bool)>::new();

        let all_literals = self
            .clauses
            .iter()
            .flat_map(|clause| clause.iter_literals());

        for literal in all_literals {
            let entry = pos_neg_by_variable
                .entry(literal.into_variable())
                .or_insert((false, false));

            if literal.is_neg() {
                entry.1 = true;
            } else {
                entry.0 = true;
            }
        }

        pos_neg_by_variable
            .into_iter()
            .filter_map(|(variable, pos_neg)| match pos_neg {
                (true, false) => Some(Literal::new_from_variable(variable, false)),
                (false, true) => Some(Literal::new_from_variable(variable, true)),
                _ => None,
            })
            .collect()
    }
}

impl Clause {
    fn as_unit_clause(&self) -> Option<Literal> {
        match self.0.as_slice() {
            &[unit_literal] => Some(unit_literal),
            _ => None,
        }
    }
}
