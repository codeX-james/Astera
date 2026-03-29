# ADR-0003: No Tokenization of Pool Shares

## Status

Accepted

## Context

Liquidity pool designs typically face a choice between:

1. **Tokenized shares**: Issuing LP tokens representing pool ownership (e.g., Uniswap LP tokens)
2. **Internal accounting**: Tracking positions through contract state without separate tokens

Each approach has implications for composability, complexity, and regulatory considerations.

## Decision

We will not tokenize pool shares. Investor positions are tracked internally through the `InvestorPosition` struct in contract storage.

```rust
pub struct InvestorPosition {
    pub deposited: i128,
    pub available: i128,
    pub deployed: i128,
    pub earned: i128,
    pub deposit_count: u32,
}
```

## Rationale

1. **Regulatory simplicity**: Tokenized pool shares may constitute securities in many jurisdictions. Internal accounting avoids creating a tradeable asset that could trigger additional regulatory requirements.

2. **Reduced attack surface**: LP tokens introduce risks including:
   - Flash loan attacks for governance manipulation
   - Price oracle manipulation
   - Secondary market speculation affecting pool stability

3. **Direct investor relationship**: Without tokenization, investors maintain a direct relationship with the pool. This simplifies:
   - KYC/AML compliance
   - Investor communication
   - Position tracking and reporting

4. **Invoice-specific attribution**: Our co-funding model tracks per-invoice contributions. This granular tracking would be complicated with fungible LP tokens.

5. **Gas efficiency**: Avoiding token minting, burning, and transfer operations reduces transaction costs.

6. **Simplicity**: Fewer moving parts means fewer potential bugs and easier auditing.

## Consequences

### Positive

- Clearer regulatory positioning
- Simpler contract architecture
- Lower transaction costs
- Direct investor tracking
- Reduced composability risks

### Negative

- No secondary market for positions
- Positions are not composable with DeFi protocols
- Cannot use positions as collateral elsewhere
- No easy fractional transfer of positions

## Future Considerations

If tokenization becomes desirable:

- Could implement as a separate opt-in layer
- Would require legal review per jurisdiction
- Should include transfer restrictions and compliance hooks

## References

- Pool contract: `contracts/pool/src/lib.rs`
- `InvestorPosition` struct
- `InvestorTokenKey` for per-token position tracking
