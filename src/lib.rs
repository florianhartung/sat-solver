use std::{
    collections::HashMap,
    mem,
    num::{NonZeroI32, NonZeroU32},
    ops::{AddAssign, BitOr, Neg, Not},
};

mod dimacs_parser;
mod solver;

pub use dimacs_parser::parse_from_dimacs_str;
pub use solver::solve;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Satisfiable,
    Unsatisfiable,
}

impl BitOr for Outcome {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Outcome::Unsatisfiable, Outcome::Unsatisfiable) => Outcome::Unsatisfiable,
            _ => Outcome::Satisfiable,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CNF {
    num_variables: u32, // u32 is not correct
    clauses: Vec<Clause>,
}

impl CNF {
    fn remove_clauses_containing(&mut self, literal: Literal) {
        self.clauses = self
            .clauses
            .iter_mut()
            .map(|clause| mem::replace(clause, Clause(Vec::new())))
            .filter(|clause| !clause.contains(literal))
            .collect()
    }

    fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    fn remove_from_clauses(&mut self, literal: Literal) {
        for clause in &mut self.clauses {
            clause.remove(literal);
        }
    }

    fn contains_empty_clause(&self) -> bool {
        self.clauses.iter().any(|clause| clause.is_empty())
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

#[derive(Debug, Clone)]
struct Clause(Vec<Literal>);

impl Clause {
    fn iter_literals(&self) -> impl Iterator<Item = Literal> {
        self.0.iter().copied()
    }

    fn contains(&self, literal: Literal) -> bool {
        self.0.contains(&literal)
    }

    fn remove(&mut self, literal_to_remove: Literal) {
        self.0.retain(|literal| literal != &literal_to_remove);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn as_unit_clause(&self) -> Option<Literal> {
        match self.0.as_slice() {
            &[unit_literal] => Some(unit_literal),
            _ => None,
        }
    }
}

/// Inner value is always guaranteed to be positive
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Variable(NonZeroI32);

impl Variable {
    /// Returns `None' if index is too large. Only `NonZeroI32::MAX` variables
    /// are allowed.
    pub fn new(inner: NonZeroU32) -> Option<Self> {
        NonZeroI32::try_from(inner).ok().map(Self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Literal(NonZeroI32);

impl Literal {
    fn new(inner: NonZeroI32) -> Self {
        Literal(inner)
    }

    fn new_from_variable(variable: Variable, is_not: bool) -> Self {
        let as_literal = Literal(variable.0);
        if is_not { as_literal.not() } else { as_literal }
    }

    fn into_variable(self) -> Variable {
        Variable(self.0.abs())
    }

    fn is_neg(self) -> bool {
        self.0.is_negative()
    }
}

impl Not for Literal {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.neg())
    }
}

#[derive(Debug, Clone, Default)]
struct Assignment(Vec<Literal>);

impl Assignment {
    fn iter_literals(&self) -> impl Iterator<Item = Literal> {
        self.0.iter().copied()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl AddAssign<Literal> for Assignment {
    fn add_assign(&mut self, rhs: Literal) {
        if !self.contains_variable(rhs.into_variable()) {
            self.0.push(rhs);
        }
    }
}

impl Assignment {
    fn contains_variable(&self, variable: Variable) -> bool {
        self.0
            .iter()
            .any(|literal| literal.into_variable() == variable)
    }
}
