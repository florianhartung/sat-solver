use std::io::{Read, stdin};

use sat_solver_core::Outcome;

fn main() -> Result<(), String> {
    let mut input_dimacs = String::new();
    let _num_bytes_read = stdin()
        .read_to_string(&mut input_dimacs)
        .map_err(|err| format!("failed to read stdin: {err}"))?;

    let cnf = sat_solver_core::parse_from_dimacs_str(&input_dimacs)
        .map_err(|err| format!("failed to parse dimacs file: {err}"))?;

    match sat_solver_core::solve(cnf) {
        Outcome::Satisfiable => println!("s UNSATISFIABLE"),
        Outcome::Unsatisfiable => println!("s SATISFIABLE"),
    }

    Ok(())
}
