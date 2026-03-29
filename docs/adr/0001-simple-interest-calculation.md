# ADR-0001: Simple Interest Calculation

## Status

Accepted

## Context

Invoice factoring platforms need to calculate interest on funded invoices. The two primary approaches are:

1. **Simple interest**: Interest calculated only on the principal amount
2. **Compound interest**: Interest calculated on principal plus accumulated interest

Invoice factoring typically involves short-term financing (30-90 days), making the choice of interest calculation method significant for both platform economics and user experience.

## Decision

We will use simple interest calculation for all invoice financing operations.

The formula used is:

```
interest = principal × annual_rate × (elapsed_seconds / seconds_per_year)
```

## Rationale

1. **Short duration**: Invoice financing typically spans 30-90 days. Over such short periods, the difference between simple and compound interest is minimal (less than 0.5% for typical rates).

2. **Transparency**: Simple interest is easier for SMEs and investors to understand and verify. The calculation is straightforward and predictable.

3. **Gas efficiency**: Simple interest requires fewer computational operations, reducing transaction costs on the Stellar network.

4. **Industry standard**: Traditional invoice factoring uses simple interest. This aligns with existing financial practices and regulatory expectations.

5. **Auditability**: Simple interest calculations are easier to audit and verify, reducing dispute potential.

## Consequences

### Positive

- Lower computational costs per transaction
- Clear and predictable returns for investors
- Easier regulatory compliance
- Simpler integration with accounting systems

### Negative

- Slightly lower returns compared to compound interest for longer-term invoices
- May need adjustment if the platform expands to longer financing terms

## References

- Pool contract: `contracts/pool/src/lib.rs` - `repay_invoice` function
- Interest calculation uses `SECS_PER_YEAR` constant (31,536,000 seconds)
