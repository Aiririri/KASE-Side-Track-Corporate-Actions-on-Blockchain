use anchor_lang::prelude::*;

#[constant]
pub const BOND_SEED: &[u8] = b"bond";

#[constant]
pub const MINT_SEED: &[u8] = b"bond_mint";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

#[constant]
pub const BPS_DENOMINATOR: u64 = 10_000;
