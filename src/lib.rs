use std::{mem, num::NonZeroI64};

pub enum Outcome {
    Satisfiable,
    Unsatisfiable,
}

#[derive(Debug)]
pub struct CNF {
    num_variables: u32,
    clauses: Vec<Clause>,
}

#[derive(Debug)]
struct Clause(Vec<Literal>);

#[derive(Debug)]
struct Literal(NonZeroI64);

pub fn solve(cnf: CNF) -> Outcome {
    todo!("write solver")
}

pub fn parse_from_dimacs_str(input: &str) -> Result<CNF, String> {
    let non_empty_lines = input.lines().filter(|line| !line.trim().is_empty());

    let mut non_comment_lines = non_empty_lines.filter(|line| !line.starts_with('c'));

    // Parse the problem line
    // It is of the form: p <NUM_VARIABLES> <NUM_CLAUSES>

    let first_line = non_comment_lines.next().ok_or("no problem line found")?;

    let mut first_line_parts = first_line.split_whitespace();

    let first_part = first_line_parts
        .next()
        .expect("that no line contains only whitespaces because this was previously checked for");
    if first_part != "p" {
        Err("no problem line found")?;
    }

    let format = first_line_parts
        .next()
        .ok_or("missing format in problem line")?;
    if format != "cnf" {
        Err(format!("unsupported format {format}"))?;
    }

    let num_variables: u32 = first_line_parts
        .next()
        .ok_or("missing variable count in problem line")?
        .parse()
        .map_err(|parse_err| {
            format!("failed to parse variable count in problem line (max is 2^32): {parse_err}")
        })?;

    let num_clauses: usize = first_line_parts
        .next()
        .ok_or("missing clause count in problem line")?
        .parse()
        .map_err(|parse_err| {
            format!("failed to parse clause count in problem line: {parse_err}")
        })?;

    if first_line_parts.next().is_some() {
        Err("problem line contains additional characters")?;
    }

    // Now parse the clauses and literals. Every clause is a list of
    // whitespace-delimited integers. Multiple clauses are delimited by zeros.

    let mut literals = non_comment_lines
        .flat_map(str::split_whitespace)
        .map(|literal| {
            str::parse::<i64>(literal)
                .map_err(|parse_err| format!("failed to parse literal: {parse_err}"))
        });

    let mut clauses: Vec<Clause> = Vec::new();
    let mut current_clause: Vec<Literal> = Vec::new();

    while let Some(literal) = literals.next() {
        match NonZeroI64::new(literal?) {
            Some(new_literal) => {
                current_clause.push(Literal(new_literal));
            }
            None => {
                if !current_clause.is_empty() {
                    let new_clause = mem::take(&mut current_clause);
                    clauses.push(Clause(new_clause));
                }
            }
        }
    }
    if !current_clause.is_empty() {
        clauses.push(Clause(current_clause));
    }

    // This check doesn't hurt I guess
    if num_clauses != clauses.len() {
        Err(format!(
            "invalid number of clauses. the problem line specified {} but only {} clauses were found",
            num_clauses,
            clauses.len()
        ))?;
    }

    Ok(CNF {
        num_variables,
        clauses,
    })
}
