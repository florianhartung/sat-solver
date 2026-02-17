use anyhow::{Context, Result, anyhow, bail};
use envconfig::Envconfig;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sat_solver_core::{Outcome, parse_from_dimacs_str, solve};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "SOLVER_TIMEOUT", default = "10")]
    pub solver_timeout: u64,
}

#[test]
fn run() -> anyhow::Result<()> {
    let config = Config::init_from_env()?;

    let tests = discover_tests_recursively(Path::new("./tests/testcases/"))?;

    let results: Vec<Result<()>> = tests
        .into_par_iter()
        .map(|(test_path, expected_outcome)| run_test(&config, test_path, expected_outcome))
        .collect();

    let num_ok = results.iter().filter(|result| result.is_ok()).count();

    println!("PASSED: {num_ok} / {}", results.len());

    if num_ok != results.len() {
        let num_failed = results.len() - num_ok;

        println!("FAILED TESTS:");

        results
            .into_iter()
            .filter_map(|result| result.err())
            .for_each(|error| println!("- {error}"));

        bail!("{num_failed} test failed");
    }

    Ok(())
}

fn run_test(config: &Config, test_path: PathBuf, expected_outcome: Outcome) -> Result<()> {
    let dimacs_file_contents = std::fs::read_to_string(&test_path)?;
    let cnf = parse_from_dimacs_str(&dimacs_file_contents).unwrap();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || tx.send(solve(cnf)));

    match rx.recv_timeout(Duration::from_secs(config.solver_timeout)) {
        Ok(outcome) => {
            if outcome == expected_outcome {
                Ok(())
            } else {
                Err(anyhow!("{} - Mismatched outcomes", test_path.display()))
            }
        }
        Err(_) => Err(anyhow!("{} - Solver timeout", test_path.display())),
    }
}

fn discover_tests_recursively(path: &Path) -> Result<Vec<(PathBuf, Outcome)>> {
    fn inner(discovered_tests: &mut Vec<(PathBuf, Outcome)>, path: &Path) -> Result<()> {
        let entries = std::fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;

            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                inner(discovered_tests, &entry.path())?;
            } else {
                let entry_path = entry.path();
                if entry_path.extension() == Some(OsStr::new("dimacs")) {
                    let path_with_ans = {
                        let mut path = entry_path.clone();
                        path.set_extension("ans");
                        path
                    };
                    let answer = std::fs::read_to_string(&path_with_ans)
                        .with_context(|| anyhow!("failed to read answer file {path_with_ans:?}"))?;
                    let outcome = match answer.trim() {
                        "SATISFIABLE" => Outcome::Satisfiable,
                        "UNSATISFIABLE" => Outcome::Unsatisfiable,
                        other => bail!("Invalid answer: {other}"),
                    };

                    discovered_tests.push((entry_path, outcome));
                }
            }
        }

        Ok(())
    }

    let mut discovered_tests = Vec::new();
    inner(&mut discovered_tests, path)?;
    Ok(discovered_tests)
}
