#[macro_use]
extern crate tracing;

use crate::consts::*;
use bus_mapping::circuit_input_builder::{CircuitInputBuilder, CircuitsParams};
use clap::{Arg, ArgAction, Command};
use eth_types::l2_types::BlockTrace;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use zkevm_circuits::witness::Block;

mod consts;

pub struct BlockTest {
    pub block: Block,
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
                CircuitInputBuilder::new_from_l2_trace(circuits_params, block_trace, false, false)
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
            block.apply_mpt_updates(&circuit_input_builder.mpt_init_state.as_ref().unwrap());
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
        .arg(
            Arg::new("output")
                .required(true)
                .long("output")
                .value_parser(clap::value_parser!(PathBuf))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("k")
                .short('k')
                .default_value("20")
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set),
        )
        .get_matches();
    let path = PathBuf::from(args.get_one::<String>("trace").unwrap());
    let output_dir = args
        .get_one::<PathBuf>("output")
        .cloned()
        .expect("output dir not set");
    let success_out_dir = output_dir.join("success");
    let failure_out_dir = output_dir.join("failure");
    fs::create_dir_all(success_out_dir.as_path()).expect("cannot create success output dir");
    fs::create_dir_all(failure_out_dir.as_path()).expect("cannot create failure output dir");
    match serde_json::from_str::<BlockTrace>(&fs::read_to_string(path.as_path()).unwrap()) {
        Ok(mut block_trace) => {
            block_trace.chain_id = 0x1;
            let block_test = BlockTest::new(block_trace, CircuitsParams::super_circuit_params());

            #[cfg(not(any(feature = "inner-prove", feature = "chunk-prove")))]
            let result = mock_prove(*args.get_one::<u32>("k").unwrap(), &block_test.block);
            #[cfg(feature = "inner-prove")]
            let result = inner_prove(&path.to_string_lossy(), &block_test.block);
            #[cfg(feature = "chunk-prove")]
            let result = chunk_prove(&path.to_string_lossy(), &block_test.block);

            if result.success {
                fs::write(
                    success_out_dir.join(path.file_name().unwrap()),
                    serde_json::to_string(&result).unwrap(),
                )
                .unwrap();
            } else {
                fs::write(
                    failure_out_dir.join(path.file_name().unwrap()),
                    serde_json::to_string(&result).unwrap(),
                )
                .unwrap();
            }
        }
        Err(e) => {
            fs::write(
                failure_out_dir.join(path.file_name().unwrap()),
                serde_json::to_string(&ProveResult {
                    success: false,
                    error: Some(format!("{:?}", e)),
                    verify_failures: None,
                })
                .unwrap(),
            )
            .unwrap();
        }
    }
}

#[cfg(not(any(feature = "inner-prove", feature = "chunk-prove")))]
fn mock_prove(k: u32, block: &Block) -> ProveResult {
    use halo2_proofs::halo2curves::bn256::Fr;
    use zkevm_circuits::util::SubCircuit;

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

#[cfg(feature = "inner-prove")]
fn inner_prove(test: &str, block: &Block) -> ProveResult {
    match panic_catch(|| prover::test::inner_prove(test, block)) {
        Ok(_) => ProveResult {
            success: true,
            ..Default::default()
        },
        Err(err) => ProveResult {
            error: Some(err),
            ..Default::default()
        },
    }
}

#[cfg(feature = "chunk-prove")]
fn chunk_prove(test: &str, block: &Block) -> ProveResult {
    match panic_catch(|| prover::test::chunk_prove(test, block)) {
        Ok(_) => ProveResult {
            success: true,
            ..Default::default()
        },
        Err(err) => ProveResult {
            error: Some(err),
            ..Default::default()
        },
    }
}

#[cfg(any(feature = "inner-prove", feature = "chunk-prove"))]
fn panic_catch<F: FnOnce() -> R, R>(f: F) -> Result<R, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|err| {
        if let Some(s) = err.downcast_ref::<String>() {
            s.to_string()
        } else if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else {
            format!("unable to get panic info {err:?}")
        }
    })
}
