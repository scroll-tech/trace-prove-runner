use cargo_toml::{Dependency, Manifest};
use clap::{Arg, ArgAction, Command};
use std::fs::write;

const CIRCUITS_PARTS: &[&str] = &[
    "aggregator",
    "bus-mapping",
    "eth-types",
    "zkevm-circuits",
    "mpt-zktrie",
    "mock",
];

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
    for part in CIRCUITS_PARTS.iter() {
        if let Some(Dependency::Detailed(dep)) = manifest.dependencies.get_mut(*part) {
            dep.branch = None;
            dep.rev = Some(args.get_one::<String>("circuits-rev").unwrap().clone());
        }
    }

    write(
        "mock-runner/Cargo.toml",
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
}
