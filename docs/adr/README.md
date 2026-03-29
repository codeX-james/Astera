# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Astera project.

## What is an ADR?

An Architecture Decision Record captures an important architectural decision made along with its context and consequences. ADRs help future contributors understand why certain design choices were made.

## ADR Index

| ID | Title | Status |
|----|-------|--------|
| [0001](0001-simple-interest-calculation.md) | Simple Interest Calculation | Accepted |
| [0002](0002-default-yield-rate.md) | Default Yield Rate of 8% APY | Accepted |
| [0003](0003-no-pool-share-tokenization.md) | No Tokenization of Pool Shares | Accepted |
| [0004](0004-storage-architecture.md) | Storage Architecture and Optimization | Accepted |

## ADR Format

Each ADR follows this structure:

- **Status**: Draft, Proposed, Accepted, Deprecated, Superseded
- **Context**: The situation and forces at play
- **Decision**: What we decided to do
- **Rationale**: Why we made this decision
- **Consequences**: The resulting context after the decision

## Creating a New ADR

1. Copy the template from an existing ADR
2. Assign the next sequential number (e.g., 0005)
3. Fill in all sections
4. Submit for review
5. Update this README index

## References

- [Michael Nygard's ADR article](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub organization](https://adr.github.io/)
