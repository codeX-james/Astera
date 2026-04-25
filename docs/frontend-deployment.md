# Frontend Deployment Guide

This guide covers how to deploy the Astera frontend to various hosting providers.

## 1. Vercel (Recommended)

The easiest way to deploy the Astera frontend is via Vercel.

### One-Click Deploy
Click the button below to fork the repository and deploy a new instance to Vercel:

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https://github.com/astera-hq/Astera&root-directory=frontend&env=NEXT_PUBLIC_NETWORK,NEXT_PUBLIC_INVOICE_CONTRACT_ID,NEXT_PUBLIC_POOL_CONTRACT_ID,NEXT_PUBLIC_USDC_TOKEN_ID)

### Manual Setup
1. Push your code to GitHub/GitLab/Bitbucket.
2. Connect your repository to Vercel.
3. Configure the following environment variables:
   - `NEXT_PUBLIC_NETWORK`: `testnet` or `mainnet`
   - `NEXT_PUBLIC_INVOICE_CONTRACT_ID`: Your deployed contract ID
   - `NEXT_PUBLIC_POOL_CONTRACT_ID`: Your deployed contract ID
   - `NEXT_PUBLIC_USDC_TOKEN_ID`: The SAC address for USDC
4. Vercel will automatically detect the Next.js framework and build the project.

## 2. Netlify

1. Connect your repository to Netlify.
2. Set the **Base directory** to `frontend`.
3. Set the **Build command** to `npm run build`.
4. Set the **Publish directory** to `.next`.
5. Add the environment variables listed above in the **Site settings > Environment variables** section.

## 3. Self-Hosted (Docker)

You can build and run the frontend using the provided `Dockerfile`.

1. Build the image:
   ```bash
   docker build -t astera-frontend ./frontend
   ```
2. Run the container:
   ```bash
   docker run -p 3000:3000 \
     -e NEXT_PUBLIC_NETWORK=testnet \
     -e NEXT_PUBLIC_INVOICE_CONTRACT_ID=... \
     -e NEXT_PUBLIC_POOL_CONTRACT_ID=... \
     astera-frontend
   ```

## Required Environment Variables

| Variable | Description | Example |
| :--- | :--- | :--- |
| `NEXT_PUBLIC_NETWORK` | Stellar network to use | `testnet` |
| `NEXT_PUBLIC_INVOICE_CONTRACT_ID` | ID of the Invoice contract | `CA...` |
| `NEXT_PUBLIC_POOL_CONTRACT_ID` | ID of the Pool contract | `CB...` |
| `NEXT_PUBLIC_USDC_TOKEN_ID` | ID of the USDC stablecoin contract | `CC...` |
| `NEXT_PUBLIC_CREDIT_SCORE_CONTRACT_ID` | (Optional) ID of the Credit Score contract | `CD...` |
