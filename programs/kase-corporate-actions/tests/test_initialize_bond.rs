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
    svm.expire_blockhash();
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

// ---------- record date ----------

fn set_time(svm: &mut LiteSVM, ts: i64) {
    let mut clock = svm.get_sysvar::<anchor_lang::prelude::Clock>();
    clock.unix_timestamp = ts;
    svm.set_sysvar(&clock);
}

fn round_pda(bond: &Pubkey, idx: u32) -> Pubkey {
    Pubkey::find_program_address(
        &[ROUND_SEED, bond.as_ref(), &idx.to_le_bytes()],
        &kase_corporate_actions::id(),
    )
    .0
}

fn read_round(svm: &LiteSVM, addr: &Pubkey) -> CouponRound {
    let acc = svm.get_account(addr).unwrap();
    let mut data: &[u8] = &acc.data;
    CouponRound::try_deserialize(&mut data).unwrap()
}

fn read_bond(svm: &LiteSVM, addr: &Pubkey) -> BondSeries {
    let acc = svm.get_account(addr).unwrap();
    let mut data: &[u8] = &acc.data;
    BondSeries::try_deserialize(&mut data).unwrap()
}

fn open_round_ix(
    signer: &Pubkey,
    issuer: &Pubkey,
    series_id: u64,
    round_idx: u32,
    record_ts: i64,
) -> Instruction {
    let (bond_series, _, _) = pdas(issuer, series_id);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::OpenCouponRound { record_ts }.data(),
        kase_corporate_actions::accounts::OpenCouponRound {
            authority: *signer,
            bond_series,
            coupon_round: round_pda(&bond_series, round_idx),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn freeze_ix(
    signer: &Pubkey,
    issuer: &Pubkey,
    series_id: u64,
    round_idx: u32,
    holder: &Pubkey,
) -> Instruction {
    let (bond_series, bond_mint, _) = pdas(issuer, series_id);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::FreezeHolder {}.data(),
        kase_corporate_actions::accounts::FreezeHolder {
            authority: *signer,
            bond_series,
            bond_mint,
            coupon_round: round_pda(&bond_series, round_idx),
            holder_token_account: ata(holder, &bond_mint),
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None),
    )
}

fn finalize_ix(signer: &Pubkey, issuer: &Pubkey, series_id: u64, round_idx: u32) -> Instruction {
    let (bond_series, bond_mint, _) = pdas(issuer, series_id);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::FinalizeRecordDate {}.data(),
        kase_corporate_actions::accounts::FinalizeRecordDate {
            authority: *signer,
            bond_series,
            bond_mint,
            coupon_round: round_pda(&bond_series, round_idx),
        }
        .to_account_metas(None),
    )
}

#[test]
fn record_date_flow() {
    let (mut svm, authority, payment_mint) = setup();
    let a = authority.pubkey();
    assert!(send(&mut svm, &authority, init_ix(&a, payment_mint, 1, 1000, 2, MATURITY_TS)));

    let h1 = Keypair::new();
    let h2 = Keypair::new();
    assert!(send(&mut svm, &authority, issue_ix(&a, &a, 1, &h1.pubkey(), 10)));
    assert!(send(&mut svm, &authority, issue_ix(&a, &a, 1, &h2.pubkey(), 5)));

    let (bond, bond_mint, _) = pdas(&a, 1);
    let round = round_pda(&bond, 0);

    // Открываем период: record date = 2000
    set_time(&mut svm, 1_000);
    assert!(send(&mut svm, &authority, open_round_ix(&a, &a, 1, 0, 2_000)));
    let r = read_round(&svm, &round);
    assert_eq!(r.coupon_per_bond, 50_000_000); // $50 на облигацию
    assert_eq!(10 * r.coupon_per_bond, 500_000_000); // $500 за 10 штук, как в задании
    assert!(r.status == RoundStatus::Open);

    // До record date заморозка запрещена
    assert!(!send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h1.pubkey())));

    set_time(&mut svm, 2_000);

    // Реестр пуст, финализация невозможна
    assert!(!send(&mut svm, &authority, finalize_ix(&a, &a, 1, 0)));

    // Замораживаем первого холдера
    assert!(send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h1.pubkey())));
    assert_eq!(read_round(&svm, &round).frozen_supply, 10);
    let h1_ata = ata(&h1.pubkey(), &bond_mint);
    assert_eq!(svm.get_account(&h1_ata).unwrap().data[108], 2); // 2 = frozen

    // Второй ещё не заморожен: реестр неполный
    assert!(!send(&mut svm, &authority, finalize_ix(&a, &a, 1, 0)));

    assert!(send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h2.pubkey())));
    // Повторная заморозка запрещена
    assert!(!send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h2.pubkey())));

    // Теперь реестр полный
    assert!(send(&mut svm, &authority, finalize_ix(&a, &a, 1, 0)));
    let r = read_round(&svm, &round);
    assert_eq!(r.snapshot_supply, 15);
    assert_eq!(r.total_due, 750_000_000); // $750 всего
    assert!(r.status == RoundStatus::Snapshotted);
}

#[test]
fn only_authority_can_open_round() {
    let (mut svm, authority, payment_mint) = setup();
    let a = authority.pubkey();
    assert!(send(&mut svm, &authority, init_ix(&a, payment_mint, 1, 1000, 2, MATURITY_TS)));

    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();
    let ix = open_round_ix(&attacker.pubkey(), &a, 1, 0, 2_000);
    assert!(!send(&mut svm, &attacker, ix));
}

#[test]
fn rounds_are_sequential() {
    let (mut svm, authority, payment_mint) = setup();
    let a = authority.pubkey();
    assert!(send(&mut svm, &authority, init_ix(&a, payment_mint, 1, 1000, 2, MATURITY_TS)));

    assert!(send(&mut svm, &authority, open_round_ix(&a, &a, 1, 0, 2_000)));
    assert!(send(&mut svm, &authority, open_round_ix(&a, &a, 1, 1, 3_000)));
    // Пропустить номер нельзя
    assert!(!send(&mut svm, &authority, open_round_ix(&a, &a, 1, 5, 4_000)));

    let (bond, _, _) = pdas(&a, 1);
    assert_eq!(read_bond(&svm, &bond).next_round, 2);
}

#[test]
fn rejects_record_date_after_maturity() {
    let (mut svm, authority, payment_mint) = setup();
    let a = authority.pubkey();
    assert!(send(&mut svm, &authority, init_ix(&a, payment_mint, 1, 1000, 2, MATURITY_TS)));
    let ix = open_round_ix(&a, &a, 1, 0, MATURITY_TS + 1);
    assert!(!send(&mut svm, &authority, ix));
}

// ---------- coupon payment ----------

/// Кладём готовый токен-аккаунт в мини-блокчейн (165 байт по формату SPL Token)
fn put_token_account(svm: &mut LiteSVM, addr: Pubkey, mint: &Pubkey, owner: &Pubkey, amount: u64) {
    let mut data = vec![0u8; 165];
    data[0..32].copy_from_slice(mint.as_ref());
    data[32..64].copy_from_slice(owner.as_ref());
    data[64..72].copy_from_slice(&amount.to_le_bytes());
    data[108] = 1; // initialized
    svm.set_account(
        addr,
        Account {
            lamports: 2_039_280,
            data,
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn receipt_pda(round: &Pubkey, holder_ata: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[RECEIPT_SEED, round.as_ref(), holder_ata.as_ref()],
        &kase_corporate_actions::id(),
    )
    .0
}

fn fund_ix(signer: &Pubkey, issuer: &Pubkey, series_id: u64, source: Pubkey, amount: u64) -> Instruction {
    let (bond_series, _, vault) = pdas(issuer, series_id);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::FundVault { amount }.data(),
        kase_corporate_actions::accounts::FundVault {
            authority: *signer,
            bond_series,
            source,
            vault,
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None),
    )
}

fn pay_ix(
    signer: &Pubkey,
    issuer: &Pubkey,
    series_id: u64,
    round_idx: u32,
    holder: &Pubkey,
    holder_payment_account: Pubkey,
) -> Instruction {
    let (bond_series, bond_mint, vault) = pdas(issuer, series_id);
    let round = round_pda(&bond_series, round_idx);
    let holder_ata = ata(holder, &bond_mint);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::PayCoupon {}.data(),
        kase_corporate_actions::accounts::PayCoupon {
            authority: *signer,
            bond_series,
            bond_mint,
            coupon_round: round,
            holder_token_account: holder_ata,
            holder_payment_account,
            vault,
            receipt: receipt_pda(&round, &holder_ata),
            token_program: anchor_spl::token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn thaw_ix(signer: &Pubkey, issuer: &Pubkey, series_id: u64, round_idx: u32, holder: &Pubkey) -> Instruction {
    let (bond_series, bond_mint, _) = pdas(issuer, series_id);
    let round = round_pda(&bond_series, round_idx);
    let holder_ata = ata(holder, &bond_mint);
    Instruction::new_with_bytes(
        kase_corporate_actions::id(),
        &kase_corporate_actions::instruction::ThawHolder {}.data(),
        kase_corporate_actions::accounts::ThawHolder {
            authority: *signer,
            bond_series,
            bond_mint,
            coupon_round: round,
            holder_token_account: holder_ata,
            receipt: receipt_pda(&round, &holder_ata),
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None),
    )
}

/// Выпуск + два холдера (10 и 5 облигаций) + зафиксированный реестр
fn prepare_snapshot() -> (LiteSVM, Keypair, Pubkey, Keypair, Keypair) {
    let (mut svm, authority, payment_mint) = setup();
    let a = authority.pubkey();
    assert!(send(&mut svm, &authority, init_ix(&a, payment_mint, 1, 1000, 2, MATURITY_TS)));
    let h1 = Keypair::new();
    let h2 = Keypair::new();
    assert!(send(&mut svm, &authority, issue_ix(&a, &a, 1, &h1.pubkey(), 10)));
    assert!(send(&mut svm, &authority, issue_ix(&a, &a, 1, &h2.pubkey(), 5)));
    set_time(&mut svm, 1_000);
    assert!(send(&mut svm, &authority, open_round_ix(&a, &a, 1, 0, 2_000)));
    set_time(&mut svm, 2_000);
    assert!(send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h1.pubkey())));
    assert!(send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h2.pubkey())));
    assert!(send(&mut svm, &authority, finalize_ix(&a, &a, 1, 0)));
    (svm, authority, payment_mint, h1, h2)
}

#[test]
fn coupon_payment_flow() {
    let (mut svm, authority, payment_mint, h1, h2) = prepare_snapshot();
    let a = authority.pubkey();
    let (bond, bond_mint, vault) = pdas(&a, 1);
    let round = round_pda(&bond, 0);

    // Платёжные счета: эмитент ($1000), холдеры (пока 0)
    let issuer_pay = Pubkey::new_unique();
    let h1_pay = Pubkey::new_unique();
    let h2_pay = Pubkey::new_unique();
    put_token_account(&mut svm, issuer_pay, &payment_mint, &a, 1_000_000_000);
    put_token_account(&mut svm, h1_pay, &payment_mint, &h1.pubkey(), 0);
    put_token_account(&mut svm, h2_pay, &payment_mint, &h2.pubkey(), 0);

    // Эмитент кладёт в vault ровно сколько нужно: $750
    assert!(send(&mut svm, &authority, fund_ix(&a, &a, 1, issuer_pay, 750_000_000)));
    assert_eq!(token_balance(&svm, &vault), 750_000_000);
    assert_eq!(token_balance(&svm, &issuer_pay), 250_000_000);

    // Холдер 1: 10 облигаций × $1000 × 10% ÷ 2 = $500
    assert!(send(&mut svm, &authority, pay_ix(&a, &a, 1, 0, &h1.pubkey(), h1_pay)));
    assert_eq!(token_balance(&svm, &h1_pay), 500_000_000);

    // Квитанция записана
    let h1_ata = ata(&h1.pubkey(), &bond_mint);
    let acc = svm.get_account(&receipt_pda(&round, &h1_ata)).unwrap();
    let mut data: &[u8] = &acc.data;
    let receipt = PayoutReceipt::try_deserialize(&mut data).unwrap();
    assert_eq!(receipt.bonds, 10);
    assert_eq!(receipt.amount, 500_000_000);
    assert_eq!(receipt.holder, h1.pubkey());

    // Двойная выплата невозможна
    assert!(!send(&mut svm, &authority, pay_ix(&a, &a, 1, 0, &h1.pubkey(), h1_pay)));
    assert_eq!(token_balance(&svm, &h1_pay), 500_000_000);

    // Разморозить без выплаты нельзя (квитанции ещё нет)
    assert!(!send(&mut svm, &authority, thaw_ix(&a, &a, 1, 0, &h2.pubkey())));

    // Холдер 2: 5 облигаций, $250
    assert!(send(&mut svm, &authority, pay_ix(&a, &a, 1, 0, &h2.pubkey(), h2_pay)));
    assert_eq!(token_balance(&svm, &h2_pay), 250_000_000);

    // Vault пуст, период сошёлся
    assert_eq!(token_balance(&svm, &vault), 0);
    let r = read_round(&svm, &round);
    assert_eq!(r.paid_total, r.total_due);
    assert_eq!(r.holders_paid, 2);

    // После выплаты счёт размораживается
    assert!(send(&mut svm, &authority, thaw_ix(&a, &a, 1, 0, &h1.pubkey())));
    assert_eq!(svm.get_account(&h1_ata).unwrap().data[108], 1); // 1 = initialized
}

#[test]
fn cannot_pay_to_someone_elses_account() {
    let (mut svm, authority, payment_mint, h1, _h2) = prepare_snapshot();
    let a = authority.pubkey();

    let issuer_pay = Pubkey::new_unique();
    put_token_account(&mut svm, issuer_pay, &payment_mint, &a, 1_000_000_000);
    assert!(send(&mut svm, &authority, fund_ix(&a, &a, 1, issuer_pay, 750_000_000)));

    // Счёт, который принадлежит не холдеру
    let thief = Keypair::new();
    let thief_pay = Pubkey::new_unique();
    put_token_account(&mut svm, thief_pay, &payment_mint, &thief.pubkey(), 0);

    assert!(!send(&mut svm, &authority, pay_ix(&a, &a, 1, 0, &h1.pubkey(), thief_pay)));
    assert_eq!(token_balance(&svm, &thief_pay), 0);
}

#[test]
fn cannot_pay_with_underfunded_vault() {
    let (mut svm, authority, payment_mint, h1, _h2) = prepare_snapshot();
    let a = authority.pubkey();

    let issuer_pay = Pubkey::new_unique();
    let h1_pay = Pubkey::new_unique();
    put_token_account(&mut svm, issuer_pay, &payment_mint, &a, 1_000_000_000);
    put_token_account(&mut svm, h1_pay, &payment_mint, &h1.pubkey(), 0);

    // В vault только $100, а нужно $500
    assert!(send(&mut svm, &authority, fund_ix(&a, &a, 1, issuer_pay, 100_000_000)));
    assert!(!send(&mut svm, &authority, pay_ix(&a, &a, 1, 0, &h1.pubkey(), h1_pay)));
    assert_eq!(token_balance(&svm, &h1_pay), 0);
}

#[test]
fn cannot_pay_before_snapshot_is_finalized() {
    let (mut svm, authority, payment_mint) = setup();
    let a = authority.pubkey();
    assert!(send(&mut svm, &authority, init_ix(&a, payment_mint, 1, 1000, 2, MATURITY_TS)));
    let h1 = Keypair::new();
    assert!(send(&mut svm, &authority, issue_ix(&a, &a, 1, &h1.pubkey(), 10)));
    set_time(&mut svm, 1_000);
    assert!(send(&mut svm, &authority, open_round_ix(&a, &a, 1, 0, 2_000)));
    set_time(&mut svm, 2_000);
    assert!(send(&mut svm, &authority, freeze_ix(&a, &a, 1, 0, &h1.pubkey())));
    // freeze сделан, но finalize ещё нет

    let issuer_pay = Pubkey::new_unique();
    let h1_pay = Pubkey::new_unique();
    put_token_account(&mut svm, issuer_pay, &payment_mint, &a, 1_000_000_000);
    put_token_account(&mut svm, h1_pay, &payment_mint, &h1.pubkey(), 0);
    assert!(send(&mut svm, &authority, fund_ix(&a, &a, 1, issuer_pay, 500_000_000)));

    assert!(!send(&mut svm, &authority, pay_ix(&a, &a, 1, 0, &h1.pubkey(), h1_pay)));
}
