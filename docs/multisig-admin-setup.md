# Multi-sig Admin Setup Guide

This document outlines the steps to set up a multi-signature administration account for the Astera protocol on Stellar.

## Setup Steps

1. **Create Stellar Admin Account**
   - Generate a new account that will serve as the primary administrator for the contracts.
   - Fund the account with enough XLM for transaction fees and reserves.

2. **Add Multiple Signers**
   - Add the public keys of all authorized administrators as signers to the admin account.
   - For a 2-of-3 setup, add 3 signers (including the primary one or 3 additional ones).

3. **Set Thresholds**
   - Set the master weight and signer weights.
   - Configure the thresholds (Low, Medium, High).
   - For a **2-of-3** setup:
     - Each signer weight: 1
     - Master weight: 1
     - Thresholds: 2
   - For a **3-of-5** setup:
     - Each signer weight: 1
     - Master weight: 1
     - Thresholds: 3

## Testnet Validation

Before applying this to mainnet, validate the setup on testnet:

1. **Execute Admin Operations**
   - Try to perform an administrative action (e.g., updating pool config or whitelisting a token) with a single signature. It should fail.
2. **Verify Signature Requirements**
   - Collect the required number of signatures (e.g., 2 or 3) using Stellar Laboratory or a custom script.
   - Submit the multi-signed transaction and verify it succeeds.

## Security Notes

- **Use Hardware Wallets**: All signers should use hardware wallets (e.g., Ledger) for their private keys.
- **Distribute Keys Safely**: Ensure that the signers are geographically distributed and that no single individual has access to enough keys to reach the threshold.
- **Backup Recovery**: Maintain secure, offline backups of all recovery phrases.
- **Regular Audits**: Periodically review the list of signers and remove any that are no longer authorized.

> [!NOTE]
> Arabic (RTL) Support: When translating this documentation or the frontend for RTL languages like Arabic, ensure that the layout is mirrored and appropriate fonts are used to maintain readability.
