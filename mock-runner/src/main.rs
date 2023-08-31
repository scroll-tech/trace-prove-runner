#[macro_use]
extern crate tracing;

use crate::consts::*;
use bus_mapping::circuit_input_builder::{CircuitInputBuilder, CircuitsParams};
use clap::{Arg, ArgAction, Command};
use eth_types::l2_types::BlockTrace;
use halo2_proofs::halo2curves::bn256::Fr;
use serde::Serialize;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use zkevm_circuits::util::SubCircuit;
use zkevm_circuits::witness::Block;

mod consts;

pub struct BlockTest {
    pub block: Block<Fr>,
    pub circuit_input_builder: CircuitInputBuilder,
}

impl BlockTest {
    pub fn new(block_trace: BlockTrace, circuits_params: CircuitsParams) -> Self {
        let circuit_input_builder = {
            std::env::set_var(
                "COINBASE",
                format!("0x{}", hex::encode(block_trace.coinbase.address.unwrap())),
            );
            info!(
                "COINBASE env set to {:?}",
                block_trace.coinbase.address.unwrap()
            );
            std::env::set_var("CHAIN_ID", format!("{}", block_trace.chain_id));
            let mut difficulty_be_bytes = [0u8; 32];
            mock::MOCK_DIFFICULTY_L2GETH.to_big_endian(&mut difficulty_be_bytes);
            std::env::set_var("DIFFICULTY", hex::encode(difficulty_be_bytes));
            let mut circuit_input_builder =
                CircuitInputBuilder::new_from_l2_trace(circuits_params, &block_trace, false, false)
                    .expect("could not handle block tx");
            circuit_input_builder
                .finalize_building()
                .expect("could not finalize building block");
            circuit_input_builder
        };
        let block = {
            let mut block = zkevm_circuits::witness::block_convert(
                &circuit_input_builder.block,
                &circuit_input_builder.code_db,
            )
            .unwrap();
            zkevm_circuits::witness::block_apply_mpt_state(
                &mut block,
                &circuit_input_builder.mpt_init_state,
            );
            block
        };

        Self {
            block,
            circuit_input_builder,
        }
    }
}

#[derive(Default, Serialize)]
struct ProveResult {
    success: bool,
    error: Option<String>,
    verify_failures: Option<Vec<String>>,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::builder().from_env_lossy())
        .init();

    let args = Command::new("trace-prover")
        .arg(Arg::new("trace").required(true).action(ArgAction::Set))
        .arg(Arg::new("output").long("output").action(ArgAction::Set))
        .arg(
            Arg::new("k")
                .short('k')
                .default_value("20")
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set),
        )
        .get_matches();
    let path = PathBuf::from(args.get_one::<String>("trace").unwrap());
    let output = args
        .get_one::<String>("output")
        .cloned()
        .unwrap_or_else(|| {
            format!(
                "{}-result.json",
                path.file_stem().unwrap().to_str().unwrap()
            )
        });
    info!("testing trace: {}", path.display());
    let block_trace: BlockTrace =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let block_test = BlockTest::new(block_trace, CircuitsParams::super_circuit_params());
    let k = *args.get_one::<u32>("k").unwrap();
    let result = run_prover(k, &block_test.block);

    std::fs::write(output, serde_json::to_string(&result).unwrap()).unwrap();
}

fn run_prover(k: u32, block: &Block<Fr>) -> ProveResult {
    let mut result = ProveResult::default();
    let circuit = zkevm_circuits::super_circuit::SuperCircuit::<
        Fr,
        MAX_TXS,
        MAX_CALLDATA,
        MAX_INNER_BLOCKS,
        0x100,
    >::new_from_block(block);
    let instance = circuit.instance();
    let prover = halo2_proofs::dev::MockProver::run(k, &circuit, instance);
    if prover.is_err() {
        result.error = Some(format!("{:?}", prover.err().unwrap()));
        return result;
    }
    let prover = prover.unwrap();
    if let Err(e) = prover.verify_par() {
        result.verify_failures = Some(e.iter().map(|e| format!("{:?}", e)).collect());
        return result;
    }
    result.success = true;
    result
}
