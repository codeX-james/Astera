#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Symbol, Vec,
};

/// Annual yield in basis points (800 = 8% APY)
const DEFAULT_YIELD_BPS: u32 = 800;
const BPS_DENOM: u32 = 10_000;
const SECS_PER_YEAR: u64 = 31_536_000;

#[contracttype]
#[derive(Clone)]
pub struct PoolConfig {
    pub usdc_token: Address,
    pub invoice_contract: Address,
    pub admin: Address,
    pub yield_bps: u32,
    pub total_deposited: i128,
    pub total_deployed: i128,
    pub total_paid_out: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct InvestorPosition {
    pub deposited: i128,
    pub available: i128,
    pub deployed: i128,
    pub earned: i128,
    pub deposit_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct FundedInvoice {
    pub invoice_id: u64,
    pub sme: Address,
    /// Total funding target for this invoice
    pub principal: i128,
    /// Amount committed by co-funders so far; equals principal when fully funded
    pub committed: i128,
    /// Ledger timestamp when fully funded (0 while still open for commitments)
    pub funded_at: u64,
    pub due_date: u64,
    pub repaid: bool,
}

/// Composite key for per-investor, per-invoice share records
#[contracttype]
#[derive(Clone)]
pub struct CoFundKey {
    pub invoice_id: u64,
    pub investor: Address,
}

#[contracttype]
pub enum DataKey {
    Config,
    Investor(Address),
    FundedInvoice(u64),
    /// Vec<Address> of all investors who committed to this invoice
    CoFunders(u64),
    /// i128 principal share committed by a specific investor to a specific invoice
    CoFundShare(CoFundKey),
    Initialized,
}

const EVT: Symbol = symbol_short!("POOL");

#[contract]
pub struct FundingPool;

#[contractimpl]
impl FundingPool {
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        invoice_contract: Address,
    ) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }

        let config = PoolConfig {
            usdc_token,
            invoice_contract,
            admin,
            yield_bps: DEFAULT_YIELD_BPS,
            total_deposited: 0,
            total_deployed: 0,
            total_paid_out: 0,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    /// Investor deposits USDC into the pool
    pub fn deposit(env: Env, investor: Address, amount: i128) {
        investor.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");

        let token_client = token::Client::new(&env, &config.usdc_token);
        token_client.transfer(&investor, &env.current_contract_address(), &amount);

        let mut position = env
            .storage()
            .persistent()
            .get(&DataKey::Investor(investor.clone()))
            .unwrap_or(InvestorPosition {
                deposited: 0,
                available: 0,
                deployed: 0,
                earned: 0,
                deposit_count: 0,
            });

        position.deposited += amount;
        position.available += amount;
        position.deposit_count += 1;

        env.storage()
            .persistent()
            .set(&DataKey::Investor(investor.clone()), &position);

        config.total_deposited += amount;
        env.storage().instance().set(&DataKey::Config, &config);

        env.events()
            .publish((EVT, symbol_short!("deposit")), (investor, amount));
    }

    /// Admin registers an invoice for co-funding, establishing the principal target.
    /// Investors then call commit_to_invoice to fill their shares.
    pub fn init_co_funding(
        env: Env,
        admin: Address,
        invoice_id: u64,
        principal: i128,
        sme: Address,
        due_date: u64,
    ) {
        admin.require_auth();

        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");
        if admin != config.admin {
            panic!("unauthorized");
        }
        if principal <= 0 {
            panic!("principal must be positive");
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::FundedInvoice(invoice_id))
        {
            panic!("invoice already registered for funding");
        }

        let record = FundedInvoice {
            invoice_id,
            sme,
            principal,
            committed: 0,
            funded_at: 0,
            due_date,
            repaid: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::FundedInvoice(invoice_id), &record);

        let co_funders: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::CoFunders(invoice_id), &co_funders);
    }

    /// Investor commits a portion of their available pool balance toward an invoice.
    /// When the total committed reaches the principal target, USDC is disbursed to
    /// the SME and a "funded" event is emitted.
    pub fn commit_to_invoice(
        env: Env,
        investor: Address,
        invoice_id: u64,
        amount: i128,
    ) {
        investor.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");

        let mut record: FundedInvoice = env
            .storage()
            .persistent()
            .get(&DataKey::FundedInvoice(invoice_id))
            .expect("invoice not registered for co-funding");

        if record.funded_at != 0 {
            panic!("invoice already fully funded");
        }
        if record.repaid {
            panic!("invoice already repaid");
        }

        let remaining = record.principal - record.committed;
        if amount > remaining {
            panic!("amount exceeds remaining funding gap");
        }

        let mut position: InvestorPosition = env
            .storage()
            .persistent()
            .get(&DataKey::Investor(investor.clone()))
            .expect("investor has no position");

        if position.available < amount {
            panic!("insufficient available balance");
        }

        position.available -= amount;
        position.deployed += amount;
        env.storage()
            .persistent()
            .set(&DataKey::Investor(investor.clone()), &position);

        // Accumulate this investor's share; add to the co-funders list on first commit
        let share_key = CoFundKey {
            invoice_id,
            investor: investor.clone(),
        };
        let existing_share: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::CoFundShare(share_key.clone()))
            .unwrap_or(0);

        if existing_share == 0 {
            let mut co_funders: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::CoFunders(invoice_id))
                .unwrap_or_else(|| Vec::new(&env));
            co_funders.push_back(investor.clone());
            env.storage()
                .persistent()
                .set(&DataKey::CoFunders(invoice_id), &co_funders);
        }

        env.storage()
            .persistent()
            .set(&DataKey::CoFundShare(share_key), &(existing_share + amount));

        record.committed += amount;
        config.total_deployed += amount;

        // Disburse to SME once fully funded
        if record.committed == record.principal {
            let token_client = token::Client::new(&env, &config.usdc_token);
            token_client.transfer(
                &env.current_contract_address(),
                &record.sme,
                &record.principal,
            );

            record.funded_at = env.ledger().timestamp();
            env.events().publish(
                (EVT, symbol_short!("funded")),
                (invoice_id, record.sme.clone(), record.principal),
            );
        }

        env.storage()
            .persistent()
            .set(&DataKey::FundedInvoice(invoice_id), &record);
        env.storage().instance().set(&DataKey::Config, &config);
    }

    /// SME repays the invoice; principal and pro-rata yield are credited back to
    /// each co-funder's available balance.
    pub fn repay_invoice(env: Env, invoice_id: u64, payer: Address) {
        payer.require_auth();

        let mut config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");

        let mut record: FundedInvoice = env
            .storage()
            .persistent()
            .get(&DataKey::FundedInvoice(invoice_id))
            .expect("invoice not found");

        if record.funded_at == 0 {
            panic!("invoice not fully funded yet");
        }
        if record.repaid {
            panic!("already repaid");
        }

        let now = env.ledger().timestamp();
        let elapsed_secs = now - record.funded_at;
        let total_interest = (record.principal as u128
            * config.yield_bps as u128
            * elapsed_secs as u128)
            / (BPS_DENOM as u128 * SECS_PER_YEAR as u128);
        let total_due = record.principal + total_interest as i128;

        let token_client = token::Client::new(&env, &config.usdc_token);
        token_client.transfer(&payer, &env.current_contract_address(), &total_due);

        // Distribute principal + proportional yield to every co-funder
        let co_funders: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::CoFunders(invoice_id))
            .unwrap_or_else(|| Vec::new(&env));

        for investor_addr in co_funders.iter() {
            let share_key = CoFundKey {
                invoice_id,
                investor: investor_addr.clone(),
            };
            let share: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::CoFundShare(share_key))
                .unwrap_or(0);
            if share == 0 {
                continue;
            }

            let investor_interest =
                (total_interest * share as u128 / record.principal as u128) as i128;

            let mut pos: InvestorPosition = env
                .storage()
                .persistent()
                .get(&DataKey::Investor(investor_addr.clone()))
                .expect("co-funder position missing");

            pos.available += share + investor_interest;
            pos.deployed -= share;
            pos.earned += investor_interest;
            env.storage()
                .persistent()
                .set(&DataKey::Investor(investor_addr), &pos);
        }

        record.repaid = true;
        env.storage()
            .persistent()
            .set(&DataKey::FundedInvoice(invoice_id), &record);

        config.total_deployed -= record.principal;
        config.total_paid_out += total_due;
        config.total_deposited += total_interest as i128;
        env.storage().instance().set(&DataKey::Config, &config);

        env.events().publish(
            (EVT, symbol_short!("repaid")),
            (invoice_id, record.principal, total_interest as i128),
        );
    }

    /// Investor withdraws their available (undeployed) USDC
    pub fn withdraw(env: Env, investor: Address, amount: i128) {
        investor.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let mut config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");

        let mut position: InvestorPosition = env
            .storage()
            .persistent()
            .get(&DataKey::Investor(investor.clone()))
            .expect("no position found");

        if position.available < amount {
            panic!("insufficient available balance");
        }

        let token_client = token::Client::new(&env, &config.usdc_token);
        token_client.transfer(&env.current_contract_address(), &investor, &amount);

        position.available -= amount;
        position.deposited -= amount;
        env.storage()
            .persistent()
            .set(&DataKey::Investor(investor.clone()), &position);

        config.total_deposited -= amount;
        env.storage().instance().set(&DataKey::Config, &config);

        env.events()
            .publish((EVT, symbol_short!("withdraw")), (investor, amount));
    }

    /// Admin updates the pool yield rate
    pub fn set_yield(env: Env, admin: Address, yield_bps: u32) {
        admin.require_auth();
        let mut config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");
        if admin != config.admin {
            panic!("unauthorized");
        }
        if yield_bps > 5_000 {
            panic!("yield cannot exceed 50%");
        }
        config.yield_bps = yield_bps;
        env.storage().instance().set(&DataKey::Config, &config);
    }

    // ---- Views ----

    pub fn get_config(env: Env) -> PoolConfig {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized")
    }

    pub fn get_position(env: Env, investor: Address) -> Option<InvestorPosition> {
        env.storage()
            .persistent()
            .get(&DataKey::Investor(investor))
    }

    pub fn get_funded_invoice(env: Env, invoice_id: u64) -> Option<FundedInvoice> {
        env.storage()
            .persistent()
            .get(&DataKey::FundedInvoice(invoice_id))
    }

    /// Returns the USDC amount this investor has committed to a specific invoice
    pub fn get_co_fund_share(env: Env, invoice_id: u64, investor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::CoFundShare(CoFundKey { invoice_id, investor }))
            .unwrap_or(0)
    }

    /// Available undeployed liquidity in the pool
    pub fn available_liquidity(env: Env) -> i128 {
        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");
        config.total_deposited - config.total_deployed
    }

    /// Estimate total repayment for an invoice at current time
    pub fn estimate_repayment(env: Env, invoice_id: u64) -> i128 {
        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .expect("not initialized");
        let record: FundedInvoice = env
            .storage()
            .persistent()
            .get(&DataKey::FundedInvoice(invoice_id))
            .expect("invoice not funded");

        if record.funded_at == 0 {
            return record.principal;
        }

        let now = env.ledger().timestamp();
        let elapsed = now - record.funded_at;
        let interest = (record.principal as u128
            * config.yield_bps as u128
            * elapsed as u128)
            / (BPS_DENOM as u128 * SECS_PER_YEAR as u128);

        record.principal + interest as i128
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env,
    };

    fn setup(env: &Env) -> (FundingPoolClient<'_>, Address, Address) {
        // Start with a non-zero timestamp so funded_at can be distinguished from "unset"
        env.ledger().with_mut(|l| l.timestamp = 100_000);

        let contract_id = env.register(FundingPool, ());
        let client = FundingPoolClient::new(env, &contract_id);

        let admin = Address::generate(env);
        let token_admin = Address::generate(env);
        let usdc_id = env.register_stellar_asset_contract_v2(token_admin).address();
        let invoice_contract = Address::generate(env);

        client.initialize(&admin, &usdc_id, &invoice_contract);
        (client, admin, usdc_id)
    }

    fn mint(env: &Env, token_id: &Address, to: &Address, amount: i128) {
        soroban_sdk::token::StellarAssetClient::new(env, token_id).mint(to, &amount);
    }

    #[test]
    fn test_deposit_and_withdraw() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _admin, usdc_id) = setup(&env);
        let investor = Address::generate(&env);
        mint(&env, &usdc_id, &investor, 5_000_000_000);

        client.deposit(&investor, &2_000_000_000);

        let pos = client.get_position(&investor).unwrap();
        assert_eq!(pos.deposited, 2_000_000_000);
        assert_eq!(pos.available, 2_000_000_000);

        client.withdraw(&investor, &500_000_000);
        let pos = client.get_position(&investor).unwrap();
        assert_eq!(pos.available, 1_500_000_000);

        let config = client.get_config();
        assert_eq!(config.total_deposited, 1_500_000_000);
        assert_eq!(config.yield_bps, DEFAULT_YIELD_BPS);
    }

    #[test]
    fn test_co_funding_two_investors() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, usdc_id) = setup(&env);
        let investor1 = Address::generate(&env);
        let investor2 = Address::generate(&env);
        let sme = Address::generate(&env);

        mint(&env, &usdc_id, &investor1, 2_000_000_000);
        mint(&env, &usdc_id, &investor2, 1_000_000_000);

        client.deposit(&investor1, &2_000_000_000);
        client.deposit(&investor2, &1_000_000_000);

        let invoice_id: u64 = 1;
        let principal: i128 = 3_000_000_000;
        let due_date = env.ledger().timestamp() + 2_592_000;

        client.init_co_funding(&admin, &invoice_id, &principal, &sme, &due_date);

        // Partial commit — invoice not yet funded
        client.commit_to_invoice(&investor1, &invoice_id, &2_000_000_000);
        let record = client.get_funded_invoice(&invoice_id).unwrap();
        assert_eq!(record.committed, 2_000_000_000);
        assert_eq!(record.funded_at, 0); // still open

        // Second investor fills the remaining gap
        client.commit_to_invoice(&investor2, &invoice_id, &1_000_000_000);
        let record = client.get_funded_invoice(&invoice_id).unwrap();
        assert_eq!(record.committed, principal);
        assert!(record.funded_at != 0); // now fully funded

        // Investor positions reflect deployed capital
        let pos1 = client.get_position(&investor1).unwrap();
        assert_eq!(pos1.available, 0);
        assert_eq!(pos1.deployed, 2_000_000_000);

        let pos2 = client.get_position(&investor2).unwrap();
        assert_eq!(pos2.available, 0);
        assert_eq!(pos2.deployed, 1_000_000_000);

        // Individual shares are recorded correctly
        assert_eq!(client.get_co_fund_share(&invoice_id, &investor1), 2_000_000_000);
        assert_eq!(client.get_co_fund_share(&invoice_id, &investor2), 1_000_000_000);

        let config = client.get_config();
        assert_eq!(config.total_deployed, principal);
    }

    #[test]
    fn test_yield_split_proportional() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, usdc_id) = setup(&env);
        let investor1 = Address::generate(&env);
        let investor2 = Address::generate(&env);
        let sme = Address::generate(&env);

        mint(&env, &usdc_id, &investor1, 2_000_000_000);
        mint(&env, &usdc_id, &investor2, 1_000_000_000);
        // SME needs enough to repay principal + 30-day interest at 8% APY
        mint(&env, &usdc_id, &sme, 3_100_000_000);

        let invoice_id: u64 = 1;
        let principal: i128 = 3_000_000_000;
        let funded_ts = env.ledger().timestamp();
        let due_date = funded_ts + 2_592_000;

        client.deposit(&investor1, &2_000_000_000);
        client.deposit(&investor2, &1_000_000_000);
        client.init_co_funding(&admin, &invoice_id, &principal, &sme, &due_date);
        client.commit_to_invoice(&investor1, &invoice_id, &2_000_000_000);
        client.commit_to_invoice(&investor2, &invoice_id, &1_000_000_000);

        // Advance 30 days
        env.ledger().with_mut(|l| l.timestamp += 2_592_000);

        client.repay_invoice(&invoice_id, &sme);

        let pos1 = client.get_position(&investor1).unwrap();
        let pos2 = client.get_position(&investor2).unwrap();

        // Both investors receive their principal back
        assert!(pos1.available >= 2_000_000_000);
        assert!(pos2.available >= 1_000_000_000);

        // Both earned positive yield
        assert!(pos1.earned > 0);
        assert!(pos2.earned > 0);

        // Yield is proportional: investor1 held 2/3 of principal → earns exactly 2× investor2
        assert_eq!(pos1.earned, pos2.earned * 2);

        // No capital remains deployed
        assert_eq!(pos1.deployed, 0);
        assert_eq!(pos2.deployed, 0);

        let record = client.get_funded_invoice(&invoice_id).unwrap();
        assert!(record.repaid);
    }

    #[test]
    fn test_incremental_commits_same_investor() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, usdc_id) = setup(&env);
        let investor = Address::generate(&env);
        let sme = Address::generate(&env);

        mint(&env, &usdc_id, &investor, 3_000_000_000);
        client.deposit(&investor, &3_000_000_000);

        let due_date = env.ledger().timestamp() + 2_592_000;
        client.init_co_funding(&admin, &1, &3_000_000_000, &sme, &due_date);

        // Same investor commits in two batches
        client.commit_to_invoice(&investor, &1, &1_000_000_000);
        client.commit_to_invoice(&investor, &1, &2_000_000_000);

        assert_eq!(client.get_co_fund_share(&1, &investor), 3_000_000_000);

        let record = client.get_funded_invoice(&1).unwrap();
        assert!(record.funded_at != 0);
    }

    #[test]
    #[should_panic(expected = "amount exceeds remaining funding gap")]
    fn test_cannot_over_commit() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, usdc_id) = setup(&env);
        let investor = Address::generate(&env);
        let sme = Address::generate(&env);

        mint(&env, &usdc_id, &investor, 5_000_000_000);
        client.deposit(&investor, &5_000_000_000);

        let due_date = env.ledger().timestamp() + 2_592_000;
        client.init_co_funding(&admin, &1, &3_000_000_000, &sme, &due_date);

        // Try to commit more than the invoice needs
        client.commit_to_invoice(&investor, &1, &4_000_000_000);
    }

    #[test]
    #[should_panic(expected = "invoice not fully funded yet")]
    fn test_cannot_repay_before_fully_funded() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, usdc_id) = setup(&env);
        let investor = Address::generate(&env);
        let sme = Address::generate(&env);

        mint(&env, &usdc_id, &investor, 5_000_000_000);
        mint(&env, &usdc_id, &sme, 5_000_000_000);
        client.deposit(&investor, &5_000_000_000);

        let due_date = env.ledger().timestamp() + 2_592_000;
        client.init_co_funding(&admin, &1, &3_000_000_000, &sme, &due_date);
        // Only partial funding — do not fill completely
        client.commit_to_invoice(&investor, &1, &1_000_000_000);

        // Repayment should fail
        client.repay_invoice(&1, &sme);
    }
}
