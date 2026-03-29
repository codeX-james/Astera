# ADR-0002: Default Yield Rate of 8% APY

## Status

Accepted

## Context

The funding pool requires a default annual percentage yield (APY) that balances investor returns with borrower affordability. This rate directly impacts:

- Investor participation and pool liquidity
- SME borrowing costs
- Platform competitiveness
- Risk coverage for potential defaults

## Decision

We set the default yield rate at 8% APY (800 basis points).

The rate is stored in basis points for precision:

```rust
const DEFAULT_YIELD_BPS: u32 = 800;
const BPS_DENOM: u32 = 10_000;
```

## Rationale

1. **Market alignment**: Traditional invoice factoring rates range from 1-5% per month (12-60% APY). Our 8% annual rate positions Astera competitively below these rates while maintaining investor appeal.

2. **Risk-adjusted returns**: The rate provides adequate compensation for:
   - Default risk on invoice financing
   - Liquidity lock-up during funding periods
   - Platform operational overhead

3. **SME accessibility**: Lower rates than traditional factoring make the platform accessible to SMEs who might otherwise be priced out of invoice financing.

4. **Basis points precision**: Using basis points (1/100th of a percent) allows fine-grained rate adjustments without floating-point arithmetic, which is problematic in smart contracts.

5. **Admin adjustability**: The rate can be modified by admin up to 50% (5000 bps) to respond to market conditions while preventing predatory rates.

## Consequences

### Positive

- Competitive positioning against traditional factoring
- Clear investor expectations
- Precise rate calculations without floating-point errors
- Flexible adjustment mechanism for market response

### Negative

- May require adjustment based on actual default rates
- Fixed rate does not account for individual invoice risk profiles
- May be too low during high-risk market conditions

## Future Considerations

- Risk-based pricing per invoice based on debtor creditworthiness
- Dynamic rate adjustment based on pool utilization
- Tiered rates for different invoice durations

## References

- Pool contract: `contracts/pool/src/lib.rs`
- `DEFAULT_YIELD_BPS` constant
- `set_yield` function for admin adjustments
