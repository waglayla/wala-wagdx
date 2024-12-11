use crate::imports::*;
use super::*;
/// Estimate the transaction mass based on the GeneratorSummary. Based on mainnet params
/// 
/// Assumes typical input and output sizes:
/// - Input: 144 bytes
/// - Output: 31 bytes
/// - Script multiplier: 10
/// - Signature cost: 1000 mass units per UTXO
use waglayla_wallet_core::{
  tx::GeneratorSummary,
};

const HASH_SIZE: u64 = 32;

pub const MASS_PER_TX_BYTE: u64 = 1;
pub const MASS_PER_SCRIPT_PUB_KEY_BYTE: u64 = 10;
pub const MASS_PER_SIG_OP: u64 = 1000;

/// Estimate the transaction mass from a `GeneratorSummary`
pub fn estimate_tx_mass_from_summary(
  summary: &GeneratorSummary,
  sig_op_count_per_input: usize,
  payload_size: usize,
) -> u64 {
  let inputs = summary.aggregated_utxos;
  let outputs = summary.number_of_generated_transactions; // Assuming each transaction is a single output
  let sig_op_count = inputs * sig_op_count_per_input;

  estimate_tx_mass(inputs, outputs, 25, inputs, payload_size)
}

/// Core transaction mass estimation logic
pub fn estimate_tx_mass(
  inputs: usize,
  outputs: usize,
  script_pub_key_size: usize,
  sig_op_count: usize,
  payload_size: usize,
) -> u64 {
  let size = estimate_tx_size(inputs, outputs, script_pub_key_size, payload_size);
  let mass_for_size = size * MASS_PER_TX_BYTE;

  let total_script_pub_key_mass =
      (outputs as u64 * script_pub_key_size as u64) * MASS_PER_SCRIPT_PUB_KEY_BYTE;

  let total_sigops_mass = sig_op_count as u64 * MASS_PER_SIG_OP;

  mass_for_size + total_script_pub_key_mass + total_sigops_mass
}

/// Estimate the transaction size
fn estimate_tx_size(
  inputs: usize,
  outputs: usize,
  script_pub_key_size: usize,
  payload_size: usize,
) -> u64 {
  let mut size: u64 = 0;
  size += 2; // Tx version (u16)
  size += 8; // Number of inputs (u64)
  size += inputs as u64 * input_size();

  size += 8; // Number of outputs (u64)
  size += outputs as u64 * output_size(script_pub_key_size);

  size += 8; // Lock time (u64)
  size += 20;
  size += 8; // Gas (u64)
  size += HASH_SIZE as u64; // Payload hash

  size += 8; // Length of the payload (u64)
  size += payload_size as u64;
  size
}

/// Estimate the size of a transaction input
fn input_size() -> u64 {
  let mut size = 0;
  size += HASH_SIZE as u64; // Outpoint transaction ID
  size += 4; // Outpoint index (u32)
  size += 8; // Length of signature script (u64)
  size += 64; // Average signature script size (adjust based on actual data)
  size += 8; // Sequence (u64)
  size
}

/// Estimate the size of a transaction output
fn output_size(script_pub_key_size: usize) -> u64 {
  let mut size = 0;
  size += 8; // Value (u64)
  size += 2; // Script Public Key Version (u16)
  size += 8; // Length of Script Public Key (u64)
  size += script_pub_key_size as u64;
  size
}