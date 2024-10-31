use clap::Parser;
use std::process::Command;
use std::io::{self, Write, Error};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::env;

use pyo3::{prelude::*, types::PyTuple, types::IntoPyDict, types::PyString};

fn run_python() {
    let output = Command::new("poetry")
        .current_dir("../python_example_project")
        .arg("run")
        .arg("python")
        .arg("../python_sdk/ovejas/runtime/language_executor.py")
        .output()
        .expect("no problem");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn python_executor() ->PyResult<String> {
    // Include in binary
    let python_language_executor = include_str!("runtime/language_executor.py");

    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let fun = PyModule::from_code_bound(
            py,
            python_language_executor,
            "",
            "",
        )?
        .getattr("execute")?;

        // Set executor arguments
        let args = ("/home/duskje/Projects/ovejas_project/python_example_project/main.py",
                    "/home/duskje/Projects/ovejas_project/python_example_project/");


        let json = fun.call1(args)?;
        let json = json.extract::<String>()?;

        Ok(json)
    })
}

fn is_project_root(current_dir: &PathBuf) -> bool {
    for file in current_dir.read_dir().expect("Could not read directory") {
        let file_path = file.expect("Could not read file").path();
        let file_name = file_path.file_name().expect("Could not read file");
        let file_name = file_name.to_str().expect("Could not read file");

        if file_name == "pyproject.toml" {
            return true;
        }
    }

    false
}

fn find_project_root() -> Option<String> {
    let home_dir = env::home_dir()?;
    let mut current_dir = env::current_dir().ok()?;

    if is_project_root(&current_dir) {
        return Some(String::from(current_dir.to_str()?));
    }

    while home_dir != current_dir {
        current_dir = current_dir.parent()?.to_path_buf();

        if is_project_root(&current_dir) {
            return Some(String::from(current_dir.to_str()?));
        }
    }

    None
}

fn main(){
    let root = find_project_root();
    println!("{root:?}");
}

// target state snippet
//fn main() {
//    let target_state = python_executor().unwrap();
//    println!("{target_state}")
//}

// cli snippet
//fn main() {
//    run_python_code();
//    let cmd = clap::Command::new("ovejas")
//        .bin_name("ovejas")
//        .subcommand_required(true)
//        .subcommand(
//            clap::command!("up").arg(
//                clap::arg!(-e --"env" <NAME>)
//                    .value_parser(clap::value_parser!(String)),
//                )
//            );
//    let matches = cmd.get_matches();
//    let matches = match matches.subcommand() {
//        Some(("up", matches)) => matches,
//        _ => unreachable!("clap should ensure we don't get here"),
//    };
//
//    let name = matches.get_one::<String>("env").unwrap();
//    println!("{name:?}");
// }
