#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Symbol,
};

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum InvoiceStatus {
    Pending,
    Funded,
    Paid,
    Defaulted,
}

#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    pub id: u64,
    pub owner: Address,
    pub debtor: String,
    pub amount: i128,
    pub due_date: u64,
    pub description: String,
    pub status: InvoiceStatus,
    pub created_at: u64,
    pub funded_at: u64,
    pub paid_at: u64,
    pub pool_contract: Address,
}

#[contracttype]
pub enum DataKey {
    Invoice(u64),
    InvoiceCount,
    Admin,
    Pool,
    Initialized,
}

const EVT: Symbol = symbol_short!("INVOICE");

#[contract]
pub struct InvoiceContract;

#[contractimpl]
impl InvoiceContract {
    pub fn initialize(env: Env, admin: Address, pool: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Pool, &pool);
        env.storage().instance().set(&DataKey::InvoiceCount, &0u64);
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    /// SME creates a new invoice token on-chain
    pub fn create_invoice(
        env: Env,
        owner: Address,
        debtor: String,
        amount: i128,
        due_date: u64,
        description: String,
    ) -> u64 {
        owner.require_auth();
        if amount <= 0 {
            panic!("amount must be positive");
        }
        if due_date <= env.ledger().timestamp() {
            panic!("due date must be in the future");
        }

        let count: u64 = env.storage().instance().get(&DataKey::InvoiceCount).unwrap_or(0);
        let id = count + 1;

        // Placeholder address — real pool address set on fund_invoice
        let placeholder: Address = env.storage().instance().get(&DataKey::Admin).unwrap();

        let invoice = Invoice {
            id,
            owner: owner.clone(),
            debtor,
            amount,
            due_date,
            description,
            status: InvoiceStatus::Pending,
            created_at: env.ledger().timestamp(),
            funded_at: 0,
            paid_at: 0,
            pool_contract: placeholder,
        };

        env.storage().persistent().set(&DataKey::Invoice(id), &invoice);
        env.storage().instance().set(&DataKey::InvoiceCount, &id);
        env.events().publish((EVT, symbol_short!("created")), (id, owner, amount));

        id
    }

    /// Called by the pool contract when it funds an invoice
    pub fn mark_funded(env: Env, id: u64, pool: Address) {
        pool.require_auth();

        let authorized_pool: Address = env.storage().instance().get(&DataKey::Pool).expect("not initialized");
        if pool != authorized_pool {
            panic!("unauthorized pool");
        }

        let mut invoice: Invoice = env.storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .expect("invoice not found");

        if invoice.status != InvoiceStatus::Pending {
            panic!("invoice is not pending");
        }

        invoice.status = InvoiceStatus::Funded;
        invoice.funded_at = env.ledger().timestamp();
        invoice.pool_contract = pool;

        env.storage().persistent().set(&DataKey::Invoice(id), &invoice);
        env.events().publish((EVT, symbol_short!("funded")), id);
    }

    /// Called by the pool when repayment is confirmed
    pub fn mark_paid(env: Env, id: u64, caller: Address) {
        caller.require_auth();

        let pool: Address = env.storage().instance().get(&DataKey::Pool).expect("not initialized");
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");

        let mut invoice: Invoice = env.storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .expect("invoice not found");

        if caller != invoice.owner && caller != pool && caller != admin {
            panic!("unauthorized");
        }
        if invoice.status != InvoiceStatus::Funded {
            panic!("invoice is not funded");
        }

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::Invoice(id), &invoice);
        env.events().publish((EVT, symbol_short!("paid")), id);
    }

    /// Mark invoice as defaulted (missed due date, no repayment)
    pub fn mark_defaulted(env: Env, id: u64, pool: Address) {
        pool.require_auth();

        let authorized_pool: Address = env.storage().instance().get(&DataKey::Pool).expect("not initialized");
        if pool != authorized_pool {
            panic!("unauthorized pool");
        }

        let mut invoice: Invoice = env.storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .expect("invoice not found");

        if invoice.status != InvoiceStatus::Funded {
            panic!("invoice is not funded");
        }

        invoice.status = InvoiceStatus::Defaulted;
        env.storage().persistent().set(&DataKey::Invoice(id), &invoice);
        env.events().publish((EVT, symbol_short!("default")), id);
    }

    pub fn get_invoice(env: Env, id: u64) -> Invoice {
        env.storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .expect("invoice not found")
    }

    pub fn get_invoice_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::InvoiceCount).unwrap_or(0)
    }

    /// Update authorized pool address (admin only)
    pub fn set_pool(env: Env, admin: Address, pool: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        if admin != stored_admin {
            panic!("unauthorized");
        }
        env.storage().instance().set(&DataKey::Pool, &pool);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_create_and_fund_invoice() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, InvoiceContract);
        let client = InvoiceContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let pool = Address::generate(&env);
        let sme = Address::generate(&env);

        client.initialize(&admin, &pool);

        let id = client.create_invoice(
            &sme,
            &String::from_str(&env, "ACME Corp"),
            &1_000_000_000i128, // 1000 USDC (7 decimals)
            &(env.ledger().timestamp() + 2_592_000), // 30 days
            &String::from_str(&env, "Invoice #001 - Goods delivery"),
        );

        assert_eq!(id, 1);

        let invoice = client.get_invoice(&id);
        assert_eq!(invoice.status, InvoiceStatus::Pending);

        client.mark_funded(&id, &pool);
        let invoice = client.get_invoice(&id);
        assert_eq!(invoice.status, InvoiceStatus::Funded);

        client.mark_paid(&id, &sme);
        let invoice = client.get_invoice(&id);
        assert_eq!(invoice.status, InvoiceStatus::Paid);
    }
}
