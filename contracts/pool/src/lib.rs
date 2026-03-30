#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Symbol, Vec, IntoVal, String
};

const DEFAULT_YIELD_BPS: u32 = 800;
const BPS_DENOM: u32 = 10_000;
const SECS_PER_YEAR: u64 = 31_536_000;

const LEDGERS_PER_DAY: u32 = 17_280;
const ACTIVE_INVOICE_TTL: u32 = LEDGERS_PER_DAY * 365;
const COMPLETED_INVOICE_TTL: u32 = LEDGERS_PER_DAY * 30;
const INSTANCE_BUMP_AMOUNT: u32 = LEDGERS_PER_DAY * 30;
const INSTANCE_LIFETIME_THRESHOLD: u32 = LEDGERS_PER_DAY * 7;

#[contracttype]
#[derive(Clone)]
pub struct PoolConfig {
    pub invoice_contract: Address,
    pub admin: Address,
    pub yield_bps: u32,
    pub compound_interest: bool,
}

#[contracttype]
#[derive(Clone, Default)]
pub struct PoolTokenTotals {
    pub pool_value: i128,
    pub total_deployed: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct FundedInvoice {
    pub invoice_id: u64,
    pub sme: Address,
    pub token: Address,
    pub principal: i128,
    pub funded_at: u64,
    pub due_date: u64,
    pub repaid: bool,
}

#[contracttype]
#[derive(Clone, Default)]
pub struct PoolStorageStats {
    pub total_funded_invoices: u64,
    pub active_funded_invoices: u64,
    pub cleaned_invoices: u64,
}

#[contracttype]
pub enum DataKey {
    Config,
    ShareToken(Address),
    FundedInvoice(u64),
    AcceptedTokens,
    TokenTotals(Address),
    Initialized,
    StorageStats,
}

const EVT: Symbol = symbol_short!("POOL");

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn set_funded_invoice_ttl(env: &Env, invoice_id: u64, is_completed: bool) {
    let ttl = if is_completed {
        COMPLETED_INVOICE_TTL
    } else {
        ACTIVE_INVOICE_TTL
    };
    let key = DataKey::FundedInvoice(invoice_id);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(&key, ttl, ttl);
    }
}

fn calculate_interest(principal: u128, yield_bps: u32, elapsed_secs: u64, is_compound: bool) -> u128 {
    let denominator = BPS_DENOM as u128 * SECS_PER_YEAR as u128;
    if !is_compound {
        return (principal * yield_bps as u128 * elapsed_secs as u128) / denominator;
    }
    let elapsed_days = elapsed_secs / 86400;
    let mut amount = principal;
    let daily_rate_num = yield_bps as u128 * 86400;
    for _ in 0..elapsed_days {
        amount += (amount * daily_rate_num) / denominator;
    }
    let remaining_secs = elapsed_secs % 86400;
    if remaining_secs > 0 {
        amount += (amount * yield_bps as u128 * remaining_secs as u128) / denominator;
    }
    amount - principal
}

#[contract]
pub struct FundingPool;

#[contractimpl]
impl FundingPool {
    pub fn initialize(env: Env, admin: Address, initial_token: Address, initial_share_token: Address, invoice_contract: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }

        let config = PoolConfig {
            invoice_contract,
            admin: admin.clone(),
            yield_bps: DEFAULT_YIELD_BPS,
            compound_interest: false,
        };

        let mut tokens: Vec<Address> = Vec::new(&env);
        tokens.push_back(initial_token.clone());

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::AcceptedTokens, &tokens);
        env.storage().instance().set(
            &DataKey::TokenTotals(initial_token.clone()),
            &PoolTokenTotals::default(),
        );
        env.storage().instance().set(&DataKey::ShareToken(initial_token), &initial_share_token);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage()
            .instance()
            .set(&DataKey::StorageStats, &PoolStorageStats::default());
        bump_instance(&env);
    }

    pub fn add_token(env: Env, admin: Address, token: Address, share_token: Address) {
        admin.require_auth();
        bump_instance(&env);
        Self::require_admin(&env, &admin);

        let mut tokens: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::AcceptedTokens)
            .expect("not initialized");

        for i in 0..tokens.len() {
            if tokens.get(i).unwrap() == token {
                panic!("token already accepted");
            }
        }
        tokens.push_back(token.clone());
        env.storage().instance().set(&DataKey::AcceptedTokens, &tokens);

        if !env.storage().instance().has(&DataKey::TokenTotals(token.clone())) {
            env.storage().instance().set(
                &DataKey::TokenTotals(token.clone()),
                &PoolTokenTotals::default(),
            );
            env.storage().instance().set(&DataKey::ShareToken(token), &share_token);
        }
    }

    pub fn remove_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        bump_instance(&env);
        Self::require_admin(&env, &admin);

        let tokens: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::AcceptedTokens)
            .expect("not initialized");

        let mut new_tokens: Vec<Address> = Vec::new(&env);
        let mut found = false;
        for i in 0..tokens.len() {
            let t = tokens.get(i).unwrap();
            if t == token {
                found = true;
            } else {
                new_tokens.push_back(t);
            }
        }
        if !found { panic!("token not in whitelist"); }

        let tt: PoolTokenTotals = env
            .storage()
            .instance()
            .get(&DataKey::TokenTotals(token.clone()))
            .unwrap_or_default();
        if tt.pool_value != 0 || tt.total_deployed != 0 {
            panic!("token has non-zero pool balances");
        }

        env.storage()
            .instance()
            .set(&DataKey::AcceptedTokens, &new_tokens);
    }

    pub fn deposit(env: Env, investor: Address, token: Address, amount: i128) {
        investor.require_auth();
        bump_instance(&env);
        if amount <= 0 { panic!("amount must be positive"); }
        Self::assert_accepted_token(&env, &token);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&investor, &env.current_contract_address(), &amount);

        let mut tt: PoolTokenTotals = env
            .storage()
            .instance()
            .get(&DataKey::TokenTotals(token.clone()))
            .unwrap_or_default();
            
        let share_token: Address = env.storage().instance().get(&DataKey::ShareToken(token.clone())).unwrap();
        
        let total_shares: i128 = env.invoke_contract(&share_token, &Symbol::new(&env, "total_supply"), Vec::new(&env));
        let shares_to_mint = if total_shares == 0 || tt.pool_value == 0 {
            amount
        } else {
            (amount as i128 * total_shares) / tt.pool_value
        };

        tt.pool_value += amount;
        env.storage().instance().set(&DataKey::TokenTotals(token), &tt);

        let mut mint_args = Vec::new(&env);
        mint_args.push_back(investor.clone().into_val(&env));
        mint_args.push_back(shares_to_mint.into_val(&env));
        let _: () = env.invoke_contract(&share_token, &Symbol::new(&env, "mint"), mint_args);

        env.events()
            .publish((EVT, symbol_short!("deposit")), (investor, amount, shares_to_mint));
    }

    pub fn withdraw(env: Env, investor: Address, token: Address, shares: i128) {
        investor.require_auth();
        bump_instance(&env);
        if shares <= 0 { panic!("shares must be positive"); }
        Self::assert_accepted_token(&env, &token);

        let share_token: Address = env.storage().instance().get(&DataKey::ShareToken(token.clone())).unwrap();
        let mut bal_args = Vec::new(&env);
        bal_args.push_back(investor.clone().into_val(&env));
        let share_balance: i128 = env.invoke_contract(&share_token, &Symbol::new(&env, "balance"), bal_args);
        if share_balance < shares { panic!("insufficient shares"); }
        
        let total_shares: i128 = env.invoke_contract(&share_token, &Symbol::new(&env, "total_supply"), Vec::new(&env));
        let mut tt: PoolTokenTotals = env.storage().instance().get(&DataKey::TokenTotals(token.clone())).unwrap_or_default();

        let amount = (shares as i128 * tt.pool_value) / total_shares;
        
        let available_liquidity = tt.pool_value - tt.total_deployed;
        if available_liquidity < amount { panic!("insufficient available liquidity"); }

        let mut burn_args = Vec::new(&env);
        burn_args.push_back(investor.clone().into_val(&env));
        burn_args.push_back(shares.into_val(&env));
        let _: () = env.invoke_contract(&share_token, &Symbol::new(&env, "burn"), burn_args);

        tt.pool_value -= amount;
        env.storage().instance().set(&DataKey::TokenTotals(token.clone()), &tt);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &investor, &amount);

        env.events()
            .publish((EVT, symbol_short!("withdraw")), (investor, amount, shares));
    }

    pub fn fund_invoice(
        env: Env,
        admin: Address,
        invoice_id: u64,
        principal: i128,
        sme: Address,
        due_date: u64,
        token: Address,
    ) {
        admin.require_auth();
        bump_instance(&env);
        Self::require_admin(&env, &admin);
        Self::assert_accepted_token(&env, &token);

        if principal <= 0 { panic!("principal must be positive"); }
        if env.storage().persistent().has(&DataKey::FundedInvoice(invoice_id)) { panic!("invoice already funded"); }

        let mut tt: PoolTokenTotals = env.storage().instance().get(&DataKey::TokenTotals(token.clone())).unwrap_or_default();
        let available = tt.pool_value - tt.total_deployed;
        if available < principal { panic!("insufficient available liquidity"); }

        let record = FundedInvoice {
            invoice_id,
            sme: sme.clone(),
            token: token.clone(),
            principal,
            funded_at: env.ledger().timestamp(),
            due_date,
            repaid: false,
        };
        env.storage().persistent().set(&DataKey::FundedInvoice(invoice_id), &record);
        set_funded_invoice_ttl(&env, invoice_id, false);

        tt.total_deployed += principal;
        env.storage().instance().set(&DataKey::TokenTotals(token.clone()), &tt);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &sme, &principal);

        let mut stats: PoolStorageStats = env.storage().instance().get(&DataKey::StorageStats).unwrap_or_default();
        stats.total_funded_invoices += 1;
        stats.active_funded_invoices += 1;
        env.storage().instance().set(&DataKey::StorageStats, &stats);

        env.events().publish((EVT, symbol_short!("funded")), (invoice_id, sme.clone(), principal));
    }

    pub fn repay_invoice(env: Env, invoice_id: u64, payer: Address) {
        payer.require_auth();
        bump_instance(&env);

        let config: PoolConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        let mut record: FundedInvoice = env.storage().persistent().get(&DataKey::FundedInvoice(invoice_id)).expect("invoice not found");

        if record.repaid { panic!("already repaid"); }

        let now = env.ledger().timestamp();
        let elapsed_secs = now - record.funded_at;
        let total_interest = calculate_interest(
            record.principal as u128,
            config.yield_bps,
            elapsed_secs,
            config.compound_interest,
        );
        let total_due = record.principal + total_interest as i128;

        let token_client = token::Client::new(&env, &record.token);
        token_client.transfer(&payer, &env.current_contract_address(), &total_due);

        record.repaid = true;
        env.storage().persistent().set(&DataKey::FundedInvoice(invoice_id), &record);
        set_funded_invoice_ttl(&env, invoice_id, true);

        let mut tt: PoolTokenTotals = env.storage().instance().get(&DataKey::TokenTotals(record.token.clone())).unwrap_or_default();
        tt.total_deployed -= record.principal;
        tt.pool_value += total_interest as i128; // Represents distributed yield!
        env.storage().instance().set(&DataKey::TokenTotals(record.token.clone()), &tt);

        let mut stats: PoolStorageStats = env.storage().instance().get(&DataKey::StorageStats).unwrap_or_default();
        stats.active_funded_invoices = stats.active_funded_invoices.saturating_sub(1);
        env.storage().instance().set(&DataKey::StorageStats, &stats);

        env.events().publish((EVT, symbol_short!("repaid")), (invoice_id, record.principal, total_interest as i128));
    }

    pub fn set_yield(env: Env, admin: Address, yield_bps: u32) {
        admin.require_auth();
        bump_instance(&env);
        let mut config: PoolConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        Self::require_admin(&env, &admin);
        if yield_bps > 5_000 { panic!("yield cannot exceed 50%"); }
        config.yield_bps = yield_bps;
        env.storage().instance().set(&DataKey::Config, &config);
    }

    pub fn set_compound_interest(env: Env, admin: Address, compound: bool) {
        admin.require_auth();
        bump_instance(&env);
        Self::require_admin(&env, &admin);
        let mut config: PoolConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        config.compound_interest = compound;
        env.storage().instance().set(&DataKey::Config, &config);
    }

    pub fn get_config(env: Env) -> PoolConfig { env.storage().instance().get(&DataKey::Config).expect("not initialized") }
    pub fn accepted_tokens(env: Env) -> Vec<Address> { env.storage().instance().get(&DataKey::AcceptedTokens).expect("not initialized") }
    pub fn get_token_totals(env: Env, token: Address) -> PoolTokenTotals { env.storage().instance().get(&DataKey::TokenTotals(token)).unwrap_or_default() }
    pub fn get_funded_invoice(env: Env, invoice_id: u64) -> Option<FundedInvoice> { env.storage().persistent().get(&DataKey::FundedInvoice(invoice_id)) }
    pub fn available_liquidity(env: Env, token: Address) -> i128 {
        let tt: PoolTokenTotals = env.storage().instance().get(&DataKey::TokenTotals(token)).unwrap_or_default();
        tt.pool_value - tt.total_deployed
    }
    pub fn get_storage_stats(env: Env) -> PoolStorageStats { env.storage().instance().get(&DataKey::StorageStats).unwrap_or_default() }

    pub fn cleanup_funded_invoice(env: Env, admin: Address, invoice_id: u64) {
        admin.require_auth();
        bump_instance(&env);
        Self::require_admin(&env, &admin);
        let record: FundedInvoice = env.storage().persistent().get(&DataKey::FundedInvoice(invoice_id)).expect("funded invoice not found");
        if !record.repaid { panic!("can only cleanup repaid invoices"); }
        env.storage().persistent().remove(&DataKey::FundedInvoice(invoice_id));

        let mut stats: PoolStorageStats = env.storage().instance().get(&DataKey::StorageStats).unwrap_or_default();
        stats.cleaned_invoices += 1;
        env.storage().instance().set(&DataKey::StorageStats, &stats);
        env.events().publish((EVT, symbol_short!("cleanup")), invoice_id);
    }

    pub fn estimate_repayment(env: Env, invoice_id: u64) -> i128 {
        bump_instance(&env);
        let config: PoolConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        let record: FundedInvoice = env.storage().persistent().get(&DataKey::FundedInvoice(invoice_id)).expect("invoice not funded");
        if record.funded_at == 0 { return record.principal; }
        
        let now = env.ledger().timestamp();
        let elapsed = now - record.funded_at;
        let interest = calculate_interest(
            record.principal as u128,
            config.yield_bps,
            elapsed,
            config.compound_interest,
        );
        record.principal + interest as i128
    }

    fn require_admin(env: &Env, admin: &Address) {
        let config: PoolConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        if admin != &config.admin { panic!("unauthorized"); }
    }

    fn assert_accepted_token(env: &Env, token: &Address) {
        let tokens: Vec<Address> = env.storage().instance().get(&DataKey::AcceptedTokens).expect("not initialized");
        for i in 0..tokens.len() {
            if tokens.get(i).unwrap() == *token { return; }
        }
        panic!("token not accepted");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env, String
    };

    #[contract]
    pub struct DummyShare;
    #[contractimpl]
    impl DummyShare {
        pub fn total_supply(env: Env) -> i128 { env.storage().instance().get(&symbol_short!("tot")).unwrap_or(0) }
        pub fn balance(env: Env, id: Address) -> i128 { env.storage().persistent().get(&id).unwrap_or(0) }
        pub fn mint(env: Env, to: Address, amount: i128) { 
            let t = Self::total_supply(env.clone());
            let b = Self::balance(env.clone(), to.clone());
            env.storage().instance().set(&symbol_short!("tot"), &(t + amount));
            env.storage().persistent().set(&to, &(b + amount));
        }
        pub fn burn(env: Env, from: Address, amount: i128) {
            let t = Self::total_supply(env.clone());
            let b = Self::balance(env.clone(), from.clone());
            env.storage().instance().set(&symbol_short!("tot"), &(t - amount));
            env.storage().persistent().set(&from, &(b - amount));
        }
    }

    fn setup(env: &Env) -> (FundingPoolClient<'_>, Address, Address, Address) {
        env.ledger().with_mut(|l| l.timestamp = 100_000);
        let contract_id = env.register(FundingPool, ());
        let client = FundingPoolClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let token_admin = Address::generate(env);
        let usdc_id = env.register_stellar_asset_contract_v2(token_admin).address();
        let invoice_contract = Address::generate(env);
        
        let share_token = env.register(DummyShare, ());
        client.initialize(&admin, &usdc_id, &share_token, &invoice_contract);
        (client, admin, usdc_id, share_token)
    }

    fn mint(env: &Env, token_id: &Address, to: &Address, amount: i128) {
        soroban_sdk::token::StellarAssetClient::new(env, token_id).mint(to, &amount);
    }

    #[test]
    fn test_vault_deposit_withdraw() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, usdc_id, share_token) = setup(&env);
        let investor1 = Address::generate(&env);
        let investor2 = Address::generate(&env);

        mint(&env, &usdc_id, &investor1, 1000);
        mint(&env, &usdc_id, &investor2, 1000);

        client.deposit(&investor1, &usdc_id, &1000);
        
        let shares1: i128 = env.invoke_contract(&share_token, &Symbol::new(&env, "balance"), soroban_sdk::vec![&env, investor1.clone().into_val(&env)]);
        assert_eq!(shares1, 1000);

        let tt = client.get_token_totals(&usdc_id);
        assert_eq!(tt.pool_value, 1000);

        client.deposit(&investor2, &usdc_id, &500);

        let shares2: i128 = env.invoke_contract(&share_token, &Symbol::new(&env, "balance"), soroban_sdk::vec![&env, investor2.clone().into_val(&env)]);
        assert_eq!(shares2, 500);

        client.withdraw(&investor1, &usdc_id, &1000);
        let bal = soroban_sdk::token::Client::new(&env, &usdc_id).balance(&investor1);
        assert_eq!(bal, 1000);
    }
    
    #[test]
    fn test_yield_accumulation() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, usdc_id, share_token) = setup(&env);
        let investor = Address::generate(&env);
        let sme = Address::generate(&env);

        mint(&env, &usdc_id, &investor, 10000);
        mint(&env, &usdc_id, &sme, 10000);
        
        client.deposit(&investor, &usdc_id, &10000);
        client.fund_invoice(&admin, &1u64, &5000i128, &sme, &(env.ledger().timestamp() + 50000), &usdc_id);
        
        env.ledger().with_mut(|l| l.timestamp += 100_000); // 100k secs
        client.repay_invoice(&1u64, &sme);
        
        // Wait, 5000 principal at 8% APY for 100k secs.
        let tt = client.get_token_totals(&usdc_id);
        assert!(tt.pool_value > 10000);
        
        // When investor withdraws their 10000 shares, they should get > 10000 underlying!
        client.withdraw(&investor, &usdc_id, &10000);
        let bal = soroban_sdk::token::Client::new(&env, &usdc_id).balance(&investor);
        assert_eq!(bal, tt.pool_value); // Investor got everything because they owned 100% shares
    }
}
