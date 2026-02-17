use std::ops::BitOr;

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
