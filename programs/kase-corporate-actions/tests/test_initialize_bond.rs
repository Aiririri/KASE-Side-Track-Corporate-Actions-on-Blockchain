use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    kase_corporate_actions::{constants::*, state::*},
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const FACE_VALUE: u64 = 1_000_000_000; // $1000 при 6 знаках (как у USDC)
const MATURITY_TS: i64 = 4_000_000_000;

/// Готовим мини-блокчейн с нашей программой и тестовым USDC
fn setup() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = kase_corporate_actions::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/kase_corporate_actions.so"
    ));
    svm.add_program(program_id, bytes).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    // Тестовый USDC: аккаунт-токен на 82 байта по формату SPL Mint
    let payment_mint = Pubkey::new_unique();
    let mut data = vec![0u8; 82];
    data[36..44].copy_from_slice(&0u64.to_le_bytes()); // supply = 0
    data[44] = 6; // decimals
    data[45] = 1; // is_initialized
    svm.set_account(
        payment_mint,
        Account {
            lamports: 1_461_600,
            data,
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    (svm, authority, payment_mint)
}

fn pdas(authority: &Pubkey, series_id: u64) -> (Pubkey, Pubkey, Pubkey) {
    let program_id = kase_corporate_actions::id();
    let bond = Pubkey::find_program_address(
        &[BOND_SEED, authority.as_ref(), &series_id.to_le_bytes()],
        &program_id,
    )
    .0;
    let mint = Pubkey::find_program_address(&[MINT_SEED, bond.as_ref()], &program_id).0;
    let vault = Pubkey::find_program_address(&[VAULT_SEED, bond.as_ref()], &program_id).0;
    (bond, mint, vault)
}

fn init_ix(
    authority: &Pubkey,
    payment_mint: Pubkey,
    series_id: u64,
    rate_bps: u16,
    per_year: u8,
    maturity: i64,
) -> Instruction {
    let (bond_series, bond_mint, vault) = pdas(authority, series_id);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::InitializeBond {
            series_id,
            face_value: FACE_VALUE,
            coupon_rate_bps: rate_bps,
            coupons_per_year: per_year,
            maturity_ts: maturity,
        }
        .data(),
        kase_corporate_actions::accounts::InitializeBond {
            authority: *authority,
            bond_series,
            bond_mint,
            payment_mint,
            vault,
            token_program: anchor_spl::token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn send(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction) -> bool {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).is_ok()
}

#[test]
fn creates_bond_series() {
    let (mut svm, authority, payment_mint) = setup();
    let ix = init_ix(&authority.pubkey(), payment_mint, 1, 1000, 2, MATURITY_TS);
    assert!(send(&mut svm, &authority, ix));

    let (bond_pda, mint_pda, vault_pda) = pdas(&authority.pubkey(), 1);
    let acc = svm.get_account(&bond_pda).unwrap();
    let mut data: &[u8] = &acc.data;
    let bond = BondSeries::try_deserialize(&mut data).unwrap();

    assert_eq!(bond.authority, authority.pubkey());
    assert_eq!(bond.face_value, FACE_VALUE);
    assert_eq!(bond.coupon_rate_bps, 1000); // 10%
    assert_eq!(bond.coupons_per_year, 2);
    assert_eq!(bond.maturity_ts, MATURITY_TS);
    assert_eq!(bond.bond_mint, mint_pda);
    assert_eq!(bond.vault, vault_pda);
    assert_eq!(bond.payment_mint, payment_mint);
    assert!(bond.status == BondStatus::Active);
}

#[test]
fn rejects_rate_above_100_percent() {
    let (mut svm, authority, payment_mint) = setup();
    let ix = init_ix(&authority.pubkey(), payment_mint, 1, 10_001, 2, MATURITY_TS);
    assert!(!send(&mut svm, &authority, ix));
}

#[test]
fn rejects_bad_coupon_frequency() {
    let (mut svm, authority, payment_mint) = setup();
    let ix = init_ix(&authority.pubkey(), payment_mint, 1, 1000, 3, MATURITY_TS);
    assert!(!send(&mut svm, &authority, ix));
}

// ---------- issue_bonds ----------

fn ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[wallet.as_ref(), anchor_spl::token::ID.as_ref(), mint.as_ref()],
        &anchor_spl::associated_token::ID,
    )
    .0
}

fn issue_ix(
    signer: &Pubkey,
    issuer: &Pubkey,
    series_id: u64,
    holder: &Pubkey,
    amount: u64,
) -> Instruction {
    let (bond_series, bond_mint, _) = pdas(issuer, series_id);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::IssueBonds { amount }.data(),
        kase_corporate_actions::accounts::IssueBonds {
            authority: *signer,
            bond_series,
            bond_mint,
            holder: *holder,
            holder_token_account: ata(holder, &bond_mint),
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn token_balance(svm: &LiteSVM, addr: &Pubkey) -> u64 {
    let acc = svm.get_account(addr).unwrap();
    u64::from_le_bytes(acc.data[64..72].try_into().unwrap())
}

fn mint_supply(svm: &LiteSVM, addr: &Pubkey) -> u64 {
    let acc = svm.get_account(addr).unwrap();
    u64::from_le_bytes(acc.data[36..44].try_into().unwrap())
}

#[test]
fn issues_bonds_to_holder() {
    let (mut svm, authority, payment_mint) = setup();
    let init = init_ix(&authority.pubkey(), payment_mint, 1, 1000, 2, MATURITY_TS);
    assert!(send(&mut svm, &authority, init));

    let holder = Keypair::new();
    let (_, bond_mint, _) = pdas(&authority.pubkey(), 1);

    let ix = issue_ix(&authority.pubkey(), &authority.pubkey(), 1, &holder.pubkey(), 10);
    assert!(send(&mut svm, &authority, ix));
    let ix = issue_ix(&authority.pubkey(), &authority.pubkey(), 1, &holder.pubkey(), 5);
    assert!(send(&mut svm, &authority, ix));

    assert_eq!(token_balance(&svm, &ata(&holder.pubkey(), &bond_mint)), 15);
    assert_eq!(mint_supply(&svm, &bond_mint), 15);
}

#[test]
fn only_authority_can_issue() {
    let (mut svm, authority, payment_mint) = setup();
    let init = init_ix(&authority.pubkey(), payment_mint, 1, 1000, 2, MATURITY_TS);
    assert!(send(&mut svm, &authority, init));

    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();
    let ix = issue_ix(&attacker.pubkey(), &authority.pubkey(), 1, &attacker.pubkey(), 10);
    assert!(!send(&mut svm, &attacker, ix));
}

#[test]
fn rejects_zero_amount() {
    let (mut svm, authority, payment_mint) = setup();
    let init = init_ix(&authority.pubkey(), payment_mint, 1, 1000, 2, MATURITY_TS);
    assert!(send(&mut svm, &authority, init));

    let holder = Keypair::new();
    let ix = issue_ix(&authority.pubkey(), &authority.pubkey(), 1, &holder.pubkey(), 0);
    assert!(!send(&mut svm, &authority, ix));
}
