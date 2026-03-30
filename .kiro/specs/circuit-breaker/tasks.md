# Implementation Plan

- [x] 1. Add circuit breaker to InvoiceContract
  - Add `Paused` variant to `DataKey` enum
  - Add `require_not_paused` helper function
  - Add `pause`, `unpause`, and `is_paused` public functions with admin auth
  - Insert `require_not_paused` call at the top of every mutating function: `create_invoice`, `verify_invoice`, `resolve_dispute`, `mark_funded`, `mark_paid`, `mark_defaulted`, `cleanup_invoice`, `set_oracle`, `set_pool`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 4.1_

- [ ]* 1.1 Write unit tests for InvoiceContract circuit breaker
  - Test `pause` sets flag and emits event
  - Test `unpause` clears flag and emits event
  - Test each mutating function panics with "contract is paused" when paused
  - Test view functions succeed while paused
  - Test non-admin `pause`/`unpause` panics with "unauthorized"
  - Test `is_paused` returns false after initialization
  - **Property 1: Pause blocks all mutating operations**
  - **Property 3: Pause then unpause restores full operation**
  - **Property 4: Only admin can toggle the circuit breaker**
  - **Property 5: is_paused reflects current state**
  - **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 4.1**

- [ ] 2. Add circuit breaker to FundingPool
  - Add `Paused` variant to `DataKey` enum
  - Add `require_not_paused` helper function
  - Add `pause`, `unpause`, and `is_paused` public functions with admin auth
  - Insert `require_not_paused` call at the top of every mutating function: `deposit`, `withdraw`, `init_co_funding`, `commit_to_invoice`, `repay_invoice`, `add_token`, `remove_token`, `set_yield`, `cleanup_funded_invoice`
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 4.2_

- [ ]* 2.1 Write unit tests for FundingPool circuit breaker
  - Test `pause` sets flag and emits event
  - Test `unpause` clears flag and emits event
  - Test each mutating function panics with "contract is paused" when paused
  - Test view functions succeed while paused
  - Test non-admin `pause`/`unpause` panics with "unauthorized"
  - Test `is_paused` returns false after initialization
  - **Property 1: Pause blocks all mutating operations**
  - **Property 2: Read-only operations are unaffected by pause**
  - **Property 3: Pause then unpause restores full operation**
  - **Property 4: Only admin can toggle the circuit breaker**
  - **Property 5: is_paused reflects current state**
  - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 4.2**

- [ ] 3. Add circuit breaker to CreditScoreContract
  - Add `Paused` variant to `DataKey` enum
  - Add `require_not_paused` helper function
  - Add `pause`, `unpause`, and `is_paused` public functions with admin auth
  - Insert `require_not_paused` call at the top of every mutating function: `record_payment`, `record_default`, `set_invoice_contract`, `set_pool_contract`
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 4.3_

- [ ]* 3.1 Write unit tests for CreditScoreContract circuit breaker
  - Test `pause` sets flag and emits event
  - Test `unpause` clears flag and emits event
  - Test each mutating function panics with "contract is paused" when paused
  - Test view functions succeed while paused
  - Test non-admin `pause`/`unpause` panics with "unauthorized"
  - Test `is_paused` returns false after initialization
  - **Property 1: Pause blocks all mutating operations**
  - **Property 2: Read-only operations are unaffected by pause**
  - **Property 3: Pause then unpause restores full operation**
  - **Property 4: Only admin can toggle the circuit breaker**
  - **Property 5: is_paused reflects current state**
  - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 4.3**

- [ ] 4. Final Checkpoint - Make sure all tests are passing
  - Ensure all tests pass, ask the user if questions arise.
