use cargo_toml::{Dependency, Manifest};
use clap::{Arg, ArgAction, Command};
use std::fs::write;

fn main() {
    let args = Command::new("runner-builder")
        .arg(
            Arg::new("mock")
                .short('m')
                .long("mock-prover")
                .help("build mock prover runner")
                .action(ArgAction::SetTrue)
                .required(true), // .conflicts_with("real")
                                 // .required_unless_present("real"),
        )
        // .arg(
        //     Arg::new("real")
        //         .short('r')
        //         .long("real-prover")
        //         .help("build real prover runner")
        //         .action(ArgAction::SetTrue)
        //         .required_unless_present("mock"),
        // )
        .arg(
            Arg::new("circuits-rev")
                .long("circuits-rev")
                .help("zkevm-circuits git revision used to build")
                .action(ArgAction::Set)
                .required(true),
        )
        .get_matches();
    let mut manifest = Manifest::from_path("mock-runner/Cargo.toml").unwrap();
    for dep in manifest.dependencies.values_mut() {
        if let Dependency::Detailed(dep) = dep {
            if let Some("https://github.com/scroll-tech/zkevm-circuits.git") = dep.git.as_deref() {
                dep.branch = None;
                dep.rev = Some(args.get_one::<String>("circuits-rev").unwrap().clone());
            }
        }
    }

    write(
        "mock-runner/Cargo.toml",
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
}
