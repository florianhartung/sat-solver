//! # CNF Utilities For Solving

use std::{borrow::Cow, collections::HashMap, num::NonZeroU32, ops::Not};

use crate::{
    OwnedCNF,
    cnf::{Literal, OwnedClause, Variable},
};

#[derive(Clone)]
pub struct PartiallyAssignedCNF<'cnf> {
    num_variables: u32,
    clauses: Box<[PartiallyAssignedClause<'cnf>]>,
}

impl<'cnf> PartiallyAssignedCNF<'cnf> {
    pub fn new(cnf: OwnedCNF) -> Self {
        let clauses = cnf
            .clauses
            .into_iter()
            .map(|clause| PartiallyAssignedClause::Cow(Cow::Owned(clause)))
            .collect();

        Self {
            num_variables: cnf.num_variables,
            clauses,
        }
    }

    pub fn as_borrowing<'a>(&'a self) -> PartiallyAssignedCNF<'a> {
        PartiallyAssignedCNF {
            num_variables: self.num_variables,
            clauses: self
                .clauses
                .iter()
                .map(|clause| clause.as_borrowing())
                .collect(),
        }
    }

    pub fn iter_variables(&self) -> impl Iterator<Item = Variable> {
        (1..=self.num_variables).map(|variable_idx| {
            Variable::new(NonZeroU32::new(variable_idx).expect("this to be at least 1"))
                .expect("this to never exceed NonZeroI32::MAX")
        })
    }

    pub fn is_satisfied(&self) -> bool {
        self.clauses
            .iter()
            .all(|clause| matches!(clause, PartiallyAssignedClause::Tautology))
    }

    pub fn is_contradicting(&self) -> bool {
        self.clauses
            .iter()
            .any(|clause| matches!(clause, PartiallyAssignedClause::Contradiction))
    }

    pub fn assign(&mut self, literal: Literal) {
        self.remove_clauses_containing(literal);
        self.remove_from_clauses(literal.not());
    }

    pub fn remove_clauses_containing(&mut self, literal: Literal) {
        for clause in &mut self.clauses {
            if clause.contains(literal) {
                // removing clauses is the same as setting them to true
                *clause = PartiallyAssignedClause::Tautology;
            }
        }
    }

    pub fn remove_from_clauses(&mut self, literal: Literal) {
        for clause in &mut self.clauses {
            clause.remove(literal);
        }
    }

    pub fn get_unit_clauses(&self) -> Vec<Literal> {
        self.clauses
            .iter()
            .filter_map(|clause| clause.as_unit_clause())
            .collect()
    }

    pub fn get_pure_literals(&self) -> Vec<Literal> {
        let mut pos_neg_by_variable = HashMap::<Variable, (bool, bool)>::new();

        let all_literals = self.clauses.iter().flat_map(|clause| clause.get_literals());

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

#[derive(Clone)]
enum PartiallyAssignedClause<'cnf> {
    Tautology,
    Contradiction,
    Cow(Cow<'cnf, OwnedClause>),
}

impl PartiallyAssignedClause<'_> {
    fn as_borrowing<'a>(&'a self) -> PartiallyAssignedClause<'a> {
        match self {
            PartiallyAssignedClause::Tautology => PartiallyAssignedClause::Tautology,
            PartiallyAssignedClause::Contradiction => PartiallyAssignedClause::Contradiction,
            PartiallyAssignedClause::Cow(cow) => {
                PartiallyAssignedClause::Cow(Cow::Borrowed(cow.as_ref()))
            }
        }
    }

    fn as_unit_clause(&self) -> Option<Literal> {
        match self {
            PartiallyAssignedClause::Cow(cow) => cow.as_unit_clause(),
            PartiallyAssignedClause::Tautology | PartiallyAssignedClause::Contradiction => None,
        }
    }

    fn contains(&self, literal: Literal) -> bool {
        match self {
            PartiallyAssignedClause::Cow(cow) => cow.contains(literal),
            PartiallyAssignedClause::Tautology | PartiallyAssignedClause::Contradiction => false,
        }
    }

    fn remove(&mut self, literal: Literal) {
        match self {
            PartiallyAssignedClause::Cow(cow) => {
                if cow.contains(literal) {
                    cow.to_mut().remove(literal);
                }
            }
            PartiallyAssignedClause::Tautology | PartiallyAssignedClause::Contradiction => {}
        }

        if let PartiallyAssignedClause::Cow(cow) = self {
            if cow.is_empty() {
                *self = PartiallyAssignedClause::Contradiction;
            }
        }
    }

    fn get_literals(&self) -> Vec<Literal> {
        match self {
            PartiallyAssignedClause::Cow(cow) => cow.iter_literals().collect(),
            PartiallyAssignedClause::Tautology | PartiallyAssignedClause::Contradiction => {
                Vec::new()
            }
        }
    }
}
