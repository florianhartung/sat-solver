use std::ops::AddAssign;

use crate::cnf::{Literal, Variable};

#[derive(Debug, Clone, Default)]
pub struct Assignment(Vec<Literal>);

impl Assignment {
    pub fn iter_literals(&self) -> impl Iterator<Item = Literal> {
        self.0.iter().copied()
    }

    #[allow(unused, reason = "needed for debugging")]
    pub fn len(&self) -> usize {
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
    pub fn contains_variable(&self, variable: Variable) -> bool {
        self.0
            .iter()
            .any(|literal| literal.into_variable() == variable)
    }
}
