# Requirements Document

## Introduction

This feature adds a circuit breaker pattern to all three Soroban smart contracts (Invoice, Pool, CreditScore). The circuit breaker allows an admin to pause all state-changing operations across the system when a vulnerability or exploit is detected, preventing further damage while a fix is prepared. Read-only (view) operations remain available during a pause so that users can still inspect their positions and data.

## Glossary

- **Circuit Breaker**: A safety mechanism that halts state-changing contract operations when activated by an admin.
- **Paused State**: The condition in which a contract's circuit breaker is active and all mutating operations are blocked.
- **Admin**: The privileged address stored in each contract that is authorized to activate and deactivate the circuit breaker.
- **State-Changing Operation**: Any contract function that writes to storage or transfers tokens (e.g., `create_invoice`, `deposit`, `withdraw`, `record_payment`).
- **Read-Only Operation**: Any contract function that only reads from storage and returns data without modifying state (e.g., `get_invoice`, `get_position`, `get_credit_score`).
- **InvoiceContract**: The Soroban smart contract managing invoice lifecycle.
- **FundingPool**: The Soroban smart contract managing investor deposits, co-funding, and repayments.
- **CreditScoreContract**: The Soroban smart contract managing SME credit scores and payment history.

## Requirements

### Requirement 1

**User Story:** As an admin, I want to pause all state-changing operations on the InvoiceContract, so that I can halt activity when a vulnerability is detected.

#### Acceptance Criteria

1. WHEN an admin calls `pause` on the InvoiceContract, THE InvoiceContract SHALL store a paused flag set to true and emit a `paused` event.
2. WHEN an admin calls `unpause` on the InvoiceContract, THE InvoiceContract SHALL store a paused flag set to false and emit an `unpaused` event.
3. WHILE the InvoiceContract is paused, THE InvoiceContract SHALL reject calls to `create_invoice`, `verify_invoice`, `resolve_dispute`, `mark_funded`, `mark_paid`, `mark_defaulted`, and `cleanup_invoice` with the message "contract is paused".
4. WHILE the InvoiceContract is paused, THE InvoiceContract SHALL allow calls to `get_invoice`, `get_metadata`, `get_invoice_count`, and `get_storage_stats` to succeed.
5. IF a non-admin address calls `pause` or `unpause` on the InvoiceContract, THEN THE InvoiceContract SHALL panic with "unauthorized".

### Requirement 2

**User Story:** As an admin, I want to pause all state-changing operations on the FundingPool, so that I can protect investor funds when an exploit is detected.

#### Acceptance Criteria

1. WHEN an admin calls `pause` on the FundingPool, THE FundingPool SHALL store a paused flag set to true and emit a `paused` event.
2. WHEN an admin calls `unpause` on the FundingPool, THE FundingPool SHALL store a paused flag set to false and emit an `unpaused` event.
3. WHILE the FundingPool is paused, THE FundingPool SHALL reject calls to `deposit`, `withdraw`, `init_co_funding`, `commit_to_invoice`, `repay_invoice`, `add_token`, `remove_token`, `set_yield`, and `cleanup_funded_invoice` with the message "contract is paused".
4. WHILE the FundingPool is paused, THE FundingPool SHALL allow calls to `get_config`, `accepted_tokens`, `get_token_totals`, `get_position`, `get_funded_invoice`, `get_co_fund_share`, `available_liquidity`, `get_storage_stats`, and `estimate_repayment` to succeed.
5. IF a non-admin address calls `pause` or `unpause` on the FundingPool, THEN THE FundingPool SHALL panic with "unauthorized".

### Requirement 3

**User Story:** As an admin, I want to pause all state-changing operations on the CreditScoreContract, so that I can prevent score manipulation during an incident.

#### Acceptance Criteria

1. WHEN an admin calls `pause` on the CreditScoreContract, THE CreditScoreContract SHALL store a paused flag set to true and emit a `paused` event.
2. WHEN an admin calls `unpause` on the CreditScoreContract, THE CreditScoreContract SHALL store a paused flag set to false and emit an `unpaused` event.
3. WHILE the CreditScoreContract is paused, THE CreditScoreContract SHALL reject calls to `record_payment`, `record_default`, `set_invoice_contract`, and `set_pool_contract` with the message "contract is paused".
4. WHILE the CreditScoreContract is paused, THE CreditScoreContract SHALL allow calls to `get_credit_score`, `get_payment_history`, `get_payment_record`, `get_payment_history_length`, `get_score_band`, `is_invoice_processed`, and `get_config` to succeed.
5. IF a non-admin address calls `pause` or `unpause` on the CreditScoreContract, THEN THE CreditScoreContract SHALL panic with "unauthorized".

### Requirement 4

**User Story:** As an admin, I want to query the paused state of each contract, so that I can confirm the circuit breaker status at any time.

#### Acceptance Criteria

1. THE InvoiceContract SHALL expose an `is_paused` function that returns `true` when paused and `false` when not paused.
2. THE FundingPool SHALL expose an `is_paused` function that returns `true` when paused and `false` when not paused.
3. THE CreditScoreContract SHALL expose an `is_paused` function that returns `true` when paused and `false` when not paused.
4. WHEN a contract is newly initialized, THE contract SHALL default the paused flag to `false`.
