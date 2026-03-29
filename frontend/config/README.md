# Environment Configuration

This directory contains environment configuration templates for different deployment environments.

## Files

- `env.development` - Local development configuration (testnet)
- `env.staging` - Staging environment configuration (testnet)
- `env.production` - Production environment configuration (mainnet)

## Usage

1. Copy the appropriate file to `.env.local` in the frontend root:
   ```bash
   cp frontend/config/env.development frontend/.env.local
   ```

2. Fill in the contract IDs for your deployment

3. The application validates environment variables at startup

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `NEXT_PUBLIC_NETWORK` | Yes | Network to connect to: `testnet`, `mainnet`, or `standalone` |
| `NEXT_PUBLIC_INVOICE_CONTRACT_ID` | Yes | Deployed invoice contract ID |
| `NEXT_PUBLIC_POOL_CONTRACT_ID` | Yes | Deployed pool contract ID |
| `NEXT_PUBLIC_USDC_TOKEN_ID` | Yes | USDC token contract ID |
| `NEXT_PUBLIC_EURC_TOKEN_ID` | No | Optional EURC token contract ID |
| `NEXT_PUBLIC_HORIZON_URL` | No | Custom Horizon API URL |
| `NEXT_PUBLIC_SOROBAN_RPC_URL` | No | Custom Soroban RPC URL |

## Validation

The `lib/env.ts` module provides validation utilities:

```typescript
import { validateEnv, assertEnvValid } from '@/lib/env';

// Check if environment is valid
const { valid, errors } = validateEnv();

// Throw error if invalid (useful at app startup)
assertEnvValid();
```

## Security Notes

- Never commit `.env.local` files containing real contract IDs
- Sensitive values should only be set in deployment environment
- All `NEXT_PUBLIC_` variables are exposed to the browser
