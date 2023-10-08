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
                .conflicts_with_all(["inner", "chunk"])
                .required_unless_present_any(["inner", "chunk"]),
        )
        .arg(
            Arg::new("inner")
                .short('i')
                .long("inner-prover")
                .help("build inner prover runner")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["mock", "chunk"])
                .required_unless_present_any(["mock", "chunk"]),
        )
        .arg(
            Arg::new("chunk")
                .short('c')
                .long("chunk-prover")
                .help("build chunk prover runner")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["mock", "inner"])
                .required_unless_present_any(["mock", "inner"]),
        )
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
    if args.get_flag("inner") {
        manifest
            .features
            .get_mut("default")
            .unwrap()
            .push("inner-prove".to_string());
    }
    if args.get_flag("chunk") {
        manifest
            .features
            .get_mut("default")
            .unwrap()
            .push("chunk-prove".to_string());
    }

    write(
        "mock-runner/Cargo.toml",
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
}
