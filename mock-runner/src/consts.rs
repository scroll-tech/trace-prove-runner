use bus_mapping::circuit_input_builder::{CircuitsParams, PrecompileEcParams};

////// params for degree = 20 ////////////
pub const MAX_TXS: usize = 100;
pub const MAX_INNER_BLOCKS: usize = 100;
pub const MAX_EXP_STEPS: usize = 10_000;
pub const MAX_CALLDATA: usize = 600_000;
pub const MAX_BYTECODE: usize = 600_000;
pub const MAX_MPT_ROWS: usize = 1_000_000;
pub const MAX_KECCAK_ROWS: usize = 1_000_000;
pub const MAX_POSEIDON_ROWS: usize = 1_000_000;
pub const MAX_VERTICLE_ROWS: usize = 1_000_000;
pub const MAX_RWS: usize = 1_000_000;
pub const MAX_PRECOMPILE_EC_ADD: usize = 50;
pub const MAX_PRECOMPILE_EC_MUL: usize = 50;
pub const MAX_PRECOMPILE_EC_PAIRING: usize = 2;

pub trait CircuitsParamsConstants {
    fn circuit_params(super_circuit: bool, txs: usize) -> CircuitsParams {
        if super_circuit {
            // params for super circuit
            Self::super_circuit_params()
        } else {
            Self::sub_circuit_params(txs)
        }
    }

    fn super_circuit_params() -> CircuitsParams {
        CircuitsParams {
            max_evm_rows: MAX_RWS,
            max_rws: MAX_RWS,
            max_copy_rows: MAX_RWS,
            max_txs: MAX_TXS,
            max_calldata: MAX_CALLDATA,
            max_bytecode: MAX_BYTECODE,
            max_inner_blocks: MAX_INNER_BLOCKS,
            max_keccak_rows: MAX_KECCAK_ROWS,
            max_poseidon_rows: MAX_POSEIDON_ROWS,
            max_vertical_circuit_rows: MAX_VERTICLE_ROWS,
            max_exp_steps: MAX_EXP_STEPS,
            max_mpt_rows: MAX_MPT_ROWS,
            max_rlp_rows: MAX_CALLDATA,
            max_ec_ops: PrecompileEcParams {
                ec_add: MAX_PRECOMPILE_EC_ADD,
                ec_mul: MAX_PRECOMPILE_EC_MUL,
                ec_pairing: MAX_PRECOMPILE_EC_PAIRING,
            },
        }
    }

    fn sub_circuit_params(txs: usize) -> CircuitsParams {
        CircuitsParams {
            max_txs: txs,
            max_rws: 0,      // dynamic
            max_calldata: 0, // dynamic
            max_bytecode: 5000,
            max_mpt_rows: 5000,
            max_copy_rows: 0, // dynamic
            max_evm_rows: 0,  // dynamic
            max_exp_steps: 5000,
            max_keccak_rows: 0, // dynamic?
            max_poseidon_rows: 0,
            max_vertical_circuit_rows: 0,
            max_inner_blocks: 64,
            max_rlp_rows: 6000,
            max_ec_ops: PrecompileEcParams {
                ec_add: 50,
                ec_mul: 50,
                ec_pairing: 2,
            },
        }
    }
}

impl CircuitsParamsConstants for CircuitsParams {}
