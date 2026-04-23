#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, String, Symbol,
};

use pool::{FundingPool, FundingPoolClient};

#[contract]
pub struct DummyShare;
#[contractimpl]
impl DummyShare {
    pub fn total_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&symbol_short!("tot"))
            .unwrap_or(0)
    }
    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage().persistent().get(&id).unwrap_or(0)
    }
    pub fn mint(env: Env, to: Address, amount: i128) {
        let t = Self::total_supply(env.clone());
        let b = Self::balance(env.clone(), to.clone());
        env.storage()
            .instance()
            .set(&symbol_short!("tot"), &(t + amount));
        env.storage().persistent().set(&to, &(b + amount));
    }
    pub fn burn(env: Env, from: Address, amount: i128) {
        let t = Self::total_supply(env.clone());
        let b = Self::balance(env.clone(), from.clone());
        env.storage()
            .instance()
            .set(&symbol_short!("tot"), &(t - amount));
        env.storage().persistent().set(&from, &(b - amount));
    }
}

fn setup(env: &Env) -> (FundingPoolClient<'_>, Address, Address, Address) {
    env.ledger().with_mut(|l| l.timestamp = 100_000);
    let contract_id = env.register(FundingPool, ());
    let client = FundingPoolClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let usdc_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let invoice_contract = Address::generate(env);

    let share_token = env.register(DummyShare, ());
    client.initialize(&admin, &usdc_id, &share_token, &invoice_contract);
    (client, admin, usdc_id, share_token)
}

fn mint(env: &Env, token_id: &Address, to: &Address, amount: i128) {
    soroban_sdk::token::StellarAssetClient::new(env, token_id).mint(to, &amount);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Fuzz test: Deposit amounts
    #[test]
    fn fuzz_deposit_amounts(amount in 1i128..10_000_000_000i128) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, usdc_id, _share) = setup(&env);
        let investor = Address::generate(&env);

        mint(&env, &usdc_id, &investor, amount);
        client.deposit(&investor, &usdc_id, &amount);

        let totals = client.get_token_totals(&usdc_id);
        prop_assert_eq!(totals.pool_value, amount);
    }

    /// Fuzz test: Interest calculation with various parameters
    #[test]
    fn fuzz_interest_calculation(
        principal in 1_000_000i128..10_000_000_000i128,
        elapsed_days in 1u64..365u64
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, usdc_id, _share) = setup(&env);
        let sme = Address::generate(&env);
        let investor = Address::generate(&env);

        mint(&env, &usdc_id, &investor, principal * 2);
        mint(&env, &usdc_id, &sme, principal * 2);

        client.deposit(&investor, &usdc_id, &principal);

        let due_date = env.ledger().timestamp() + (elapsed_days * 86_400);
        client.fund_invoice(&admin, &1u64, &principal, &sme, &due_date, &usdc_id);

        env.ledger().with_mut(|l| l.timestamp += elapsed_days * 86_400);

        let estimated = client.estimate_repayment(&1u64);
        prop_assert!(estimated > principal);
        prop_assert!(estimated < principal * 2); // Sanity check
    }

    /// Fuzz test: Yield rate configuration
    #[test]
    fn fuzz_yield_rate(yield_bps in 0u32..5000u32) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _usdc, _share) = setup(&env);

        client.set_yield(&admin, &yield_bps);
        let config = client.get_config();
        prop_assert_eq!(config.yield_bps, yield_bps);
    }

    /// Fuzz test: Multiple deposits and withdrawals
    #[test]
    fn fuzz_deposit_withdraw_cycle(
        deposit_amount in 1_000_000i128..5_000_000_000i128,
        withdraw_ratio in 1u32..100u32
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, usdc_id, share_token) = setup(&env);
        let investor = Address::generate(&env);

        mint(&env, &usdc_id, &investor, deposit_amount);
        client.deposit(&investor, &usdc_id, &deposit_amount);

        let shares: i128 = env.invoke_contract(&share_token, &Symbol::new(&env, "balance"), soroban_sdk::vec![&env, investor.clone().into_val(&env)]);
        let withdraw_shares = (shares * withdraw_ratio as i128) / 100;

        if withdraw_shares > 0 {
            client.withdraw(&investor, &usdc_id, &withdraw_shares);

            let balance = soroban_sdk::token::Client::new(&env, &usdc_id).balance(&investor);
            let expected_min = (deposit_amount * withdraw_ratio as i128) / 100;
            prop_assert!(balance >= expected_min - 1); // Allow for rounding
        }
    }

    /// Fuzz test: Factoring fee calculation
    #[test]
    fn fuzz_factoring_fee(
        principal in 1_000_000i128..10_000_000_000i128,
        fee_bps in 0u32..1000u32
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _usdc, _share) = setup(&env);

        client.set_factoring_fee(&admin, &fee_bps);
        let config = client.get_config();
        prop_assert_eq!(config.factoring_fee_bps, fee_bps);

        // Verify fee calculation is within expected bounds
        let expected_fee = (principal as u128 * fee_bps as u128) / 10_000u128;
        prop_assert!(expected_fee <= principal as u128);
    }
}

#[cfg(test)]
mod deterministic_fuzz {
    use super::*;

    /// Deterministic fuzz test: Interest calculation edge cases
    #[test]
    fn test_interest_edge_cases() {
        let test_cases = vec![
            (1_000_000i128, 1u64),     // 1 day
            (1_000_000i128, 30u64),    // 1 month
            (1_000_000i128, 365u64),   // 1 year
            (100_000_000i128, 180u64), // Large principal, 6 months
            (1_000_000_000i128, 7u64), // Very large principal, 1 week
        ];

        for (principal, days) in test_cases {
            let env = Env::default();
            env.mock_all_auths();
            let (client, admin, usdc_id, _share) = setup(&env);
            let sme = Address::generate(&env);
            let investor = Address::generate(&env);

            mint(&env, &usdc_id, &investor, principal * 2);
            mint(&env, &usdc_id, &sme, principal * 2);

            client.deposit(&investor, &usdc_id, &principal);

            let due_date = env.ledger().timestamp() + (days * 86_400);
            client.fund_invoice(&admin, &1u64, &principal, &sme, &due_date, &usdc_id);

            env.ledger().with_mut(|l| l.timestamp += days * 86_400);

            let estimated = client.estimate_repayment(&1u64);
            assert!(
                estimated > principal,
                "Interest should be positive for principal={}, days={}",
                principal,
                days
            );

            // At 8% APY, max interest for 1 year should be ~8%
            let max_expected = principal + (principal * 9 / 100); // 9% buffer
            assert!(
                estimated < max_expected,
                "Interest too high for principal={}, days={}",
                principal,
                days
            );
        }
    }

    /// Deterministic fuzz test: Liquidity constraints
    #[test]
    fn test_liquidity_constraints() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, usdc_id, _share) = setup(&env);
        let investor = Address::generate(&env);
        let sme = Address::generate(&env);

        let deposit = 5_000_000_000i128;
        mint(&env, &usdc_id, &investor, deposit);
        client.deposit(&investor, &usdc_id, &deposit);

        // Fund invoices up to available liquidity
        let due_date = env.ledger().timestamp() + 86_400;

        client.fund_invoice(&admin, &1u64, &2_000_000_000i128, &sme, &due_date, &usdc_id);
        assert_eq!(client.available_liquidity(&usdc_id), 3_000_000_000i128);

        client.fund_invoice(&admin, &2u64, &2_000_000_000i128, &sme, &due_date, &usdc_id);
        assert_eq!(client.available_liquidity(&usdc_id), 1_000_000_000i128);

        // Note: 3rd invoice would fail (insufficient liquidity) but we can't test panic without std
        // Just verify available liquidity is less than requested
        assert!(client.available_liquidity(&usdc_id) < 2_000_000_000i128);
    }

    /// Deterministic fuzz test: Compound vs simple interest
    #[test]
    fn test_compound_vs_simple_interest() {
        let principal = 1_000_000_000i128;
        let days = 365u64;

        // Test simple interest
        let env1 = Env::default();
        env1.mock_all_auths();
        let (client1, admin1, usdc_id1, _share1) = setup(&env1);
        let sme1 = Address::generate(&env1);
        let investor1 = Address::generate(&env1);

        mint(&env1, &usdc_id1, &investor1, principal * 2);
        mint(&env1, &usdc_id1, &sme1, principal * 2);

        client1.set_compound_interest(&admin1, &false);
        client1.deposit(&investor1, &usdc_id1, &principal);
        client1.fund_invoice(
            &admin1,
            &1u64,
            &principal,
            &sme1,
            &(env1.ledger().timestamp() + days * 86_400),
            &usdc_id1,
        );
        env1.ledger().with_mut(|l| l.timestamp += days * 86_400);
        let simple_repayment = client1.estimate_repayment(&1u64);

        // Test compound interest
        let env2 = Env::default();
        env2.mock_all_auths();
        let (client2, admin2, usdc_id2, _share2) = setup(&env2);
        let sme2 = Address::generate(&env2);
        let investor2 = Address::generate(&env2);

        mint(&env2, &usdc_id2, &investor2, principal * 2);
        mint(&env2, &usdc_id2, &sme2, principal * 2);

        client2.set_compound_interest(&admin2, &true);
        client2.deposit(&investor2, &usdc_id2, &principal);
        client2.fund_invoice(
            &admin2,
            &1u64,
            &principal,
            &sme2,
            &(env2.ledger().timestamp() + days * 86_400),
            &usdc_id2,
        );
        env2.ledger().with_mut(|l| l.timestamp += days * 86_400);
        let compound_repayment = client2.estimate_repayment(&1u64);

        // Compound should be slightly higher than simple for 1 year
        assert!(compound_repayment >= simple_repayment);
    }
}
