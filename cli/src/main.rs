use ovejas::project::find_project_root;
use ovejas::executor::python_executor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = clap::Command::new("ovejas")
        .bin_name("ovejas")
        .subcommand_required(true)
        .subcommand(
            clap::command!("up").arg(
                clap::arg!(-e --"env" <NAME>)
                .value_parser(clap::value_parser!(String)),
            )
        )
        .subcommand(
            clap::command!("preview").arg(
                clap::arg!(-e --"env" <NAME>)
                .value_parser(clap::value_parser!(String)),
            )
        )
        .subcommand(
            clap::command!("down").arg(
                clap::arg!(-e --"env" <NAME>)
                .value_parser(clap::value_parser!(String)),
            )
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("up", matches)) => {
            let environment_name = matches.get_one::<String>("env");
            let project_root_dir = find_project_root();
            let target_state = python_executor(project_root_dir.expect("Could not find project root")).unwrap();
            let client = reqwest::blocking::Client::new();
            let response = client.post("http://httpbin.org/anything")
                .body(target_state)
                .send();

            let response = response?.text();

            println!("{response:?}");
        },
        Some(("preview", matches)) => {
            let environment_name = matches.get_one::<String>("env");
            let project_root_dir = find_project_root();
            let target_state = python_executor(project_root_dir.expect("Could not find project root")).unwrap();
            println!("{target_state}")
        },
        Some(("down", matches)) => {
            let environment_name = matches.get_one::<String>("env");
        },
        _ => unreachable!("clap should ensure we don't get here"),
    };

    Ok(())
}
