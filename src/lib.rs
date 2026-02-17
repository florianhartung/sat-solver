mod cnf;
mod dimacs_parser;
mod solver;

// Parsing
pub use dimacs_parser::parse_from_dimacs_str;

// Solving
pub use cnf::CNF;
pub use solver::{outcome::Outcome, solve};
