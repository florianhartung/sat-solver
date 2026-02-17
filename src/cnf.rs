//! # General-Purpose Data Types for CNFs

use std::{
    num::{NonZeroI32, NonZeroU32},
    ops::{Neg, Not},
};

#[derive(Debug, Clone)]
pub struct OwnedCNF {
    pub(crate) num_variables: u32, // u32 is not correct
    pub(crate) clauses: Vec<OwnedClause>,
}

impl OwnedCNF {
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    pub fn contains_empty_clause(&self) -> bool {
        self.clauses.iter().any(|clause| clause.is_empty())
    }
}

#[derive(Default, Debug, Clone)]
pub struct OwnedClause(pub Vec<Literal>);

impl OwnedClause {
    pub fn iter_literals(&self) -> impl Iterator<Item = Literal> {
        self.0.iter().copied()
    }

    pub fn contains(&self, literal: Literal) -> bool {
        self.0.contains(&literal)
    }

    pub fn remove(&mut self, literal_to_remove: Literal) {
        self.0.retain(|literal| literal != &literal_to_remove)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_unit_clause(&self) -> Option<Literal> {
        match self.0.as_slice() {
            &[unit_literal] => Some(unit_literal),
            _ => None,
        }
    }
}

/// Inner value is always guaranteed to be positive
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Variable(pub NonZeroI32);

impl Variable {
    /// Returns `None' if index is too large. Only `NonZeroI32::MAX` variables
    /// are allowed.
    pub fn new(inner: NonZeroU32) -> Option<Self> {
        NonZeroI32::try_from(inner).ok().map(Self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Literal(pub NonZeroI32);

impl Literal {
    pub fn new(inner: NonZeroI32) -> Self {
        Literal(inner)
    }

    pub fn new_from_variable(variable: Variable, is_not: bool) -> Self {
        let as_literal = Literal(variable.0);
        if is_not { as_literal.not() } else { as_literal }
    }

    pub fn into_variable(self) -> Variable {
        Variable(self.0.abs())
    }

    pub fn is_neg(self) -> bool {
        self.0.is_negative()
    }
}

impl Not for Literal {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.neg())
    }
}
