# ADR-0004: Storage Architecture and Optimization

## Status

Accepted

## Context

Soroban contracts on Stellar have two primary storage types:

1. **Instance storage**: Tied to contract instance lifetime, lower rent costs
2. **Persistent storage**: Survives beyond instance, higher rent costs, supports TTL

Efficient storage usage directly impacts:

- Contract operational costs
- Data availability guarantees
- Contract complexity

## Decision

We adopt a hybrid storage strategy:

### Instance Storage (contract lifetime data)

- Contract configuration (`PoolConfig`)
- Accepted token whitelist (`AcceptedTokens`)
- Token totals (`TokenTotals`)
- Initialization flags
- Admin address

### Persistent Storage (long-term records)

- Invoice records with TTL management
- Investor positions
- Co-funding records
- Funded invoice records

### TTL Management

- Active invoices: Extended TTL based on due date
- Completed invoices (paid/defaulted): Reduced TTL with cleanup option
- Investor positions: Extended while balance > 0

## Rationale

1. **Cost optimization**: Instance storage is cheaper for frequently accessed data that doesn't need to survive contract upgrades.

2. **Data lifecycle alignment**: Invoice data naturally has a lifecycle (created → funded → paid/defaulted). TTL management aligns storage costs with data utility.

3. **Cleanup incentives**: Allowing cleanup of completed invoices reduces long-term storage costs and provides gas refunds.

4. **Position preservation**: Investor positions use persistent storage to survive potential contract upgrades.

5. **Predictable costs**: Clear rules for what goes where makes cost estimation straightforward.

## Storage Key Design

```rust
pub enum DataKey {
    // Instance storage keys
    Config,
    AcceptedTokens,
    TokenTotals(Address),
    Initialized,
    
    // Persistent storage keys
    Invoice(u64),
    InvestorPosition(InvestorTokenKey),
    FundedInvoice(u64),
    CoFunders(u64),
    CoFundShare(CoFundKey),
}
```

## Consequences

### Positive

- Reduced storage rent costs
- Clear data lifecycle management
- Cleanup mechanism reduces long-term bloat
- Predictable cost model

### Negative

- Added complexity in choosing storage type
- TTL management requires careful implementation
- Cleanup operations consume gas

## References

- Invoice contract: `contracts/invoice/src/lib.rs`
- Pool contract: `contracts/pool/src/lib.rs`
- Soroban storage documentation
