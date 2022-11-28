use dag::*;
use parser::run_parser;
use constraint_generation::extract_dag;

const VERSION: &'static str = "2.1.0";

pub fn parse_circom(file: String) -> DAG {
    let parse_result = run_parser(file, VERSION, vec![]);
    let program = match parse_result {
        Ok(r) => r.0,
        _ => panic!("Parse error")
    };

    extract_dag(program)
}
