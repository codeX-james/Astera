'use client';

import { useState } from 'react';
import { useStore } from '@/lib/store';
import { truncateAddress } from '@/lib/stellar';
import LoadingSpinner from '@/components/LoadingSpinner';

type WalletStep = 'idle' | 'detecting' | 'requesting-access' | 'fetching-address';

const STEP_LABELS: Record<WalletStep, string> = {
  idle: 'Connect Wallet',
  detecting: 'Detecting Freighter…',
  'requesting-access': 'Approve in Freighter…',
  'fetching-address': 'Fetching address…',
};

const MAX_RETRIES = 2;

export default function WalletConnect() {
  const { wallet, setWallet, disconnect } = useStore();
  const [step, setStep] = useState<WalletStep>('idle');
  const [error, setError] = useState<string | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  const loading = step !== 'idle';

  async function connect(attempt = 0) {
    setStep('detecting');
    setError(null);
    try {
      const freighter = await import('@stellar/freighter-api');

      const { isConnected } = await freighter.isConnected();
      if (!isConnected) {
        setError('Freighter not detected. Please install the browser extension and reload.');
        setStep('idle');
        return;
      }

      setStep('requesting-access');
      const { isAllowed } = await freighter.isAllowed();
      if (!isAllowed) {
        await freighter.setAllowed();
      }

      setStep('fetching-address');
      const { address, error: addrError } = await freighter.getAddress();
      if (addrError) {
        setError('Could not retrieve wallet address. Please try again.');
        setStep('idle');
        return;
      }

      setWallet({ address, connected: true, network: 'testnet' });
      setRetryCount(0);
      setStep('idle');
    } catch (e) {
      console.error('[WalletConnect] Connection error:', e);
      if (attempt < MAX_RETRIES) {
        setRetryCount(attempt + 1);
        // Brief delay before auto-retry
        setTimeout(() => connect(attempt + 1), 800);
      } else {
        setError('Failed to connect wallet after multiple attempts. Please try again.');
        setRetryCount(0);
        setStep('idle');
      }
    }
  }

  function handleRetry() {
    setRetryCount(0);
    connect(0);
  }

  if (wallet.connected && wallet.address) {
    return (
      <div className="flex items-center gap-3">
        <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-green-900/30 border border-green-800/50 text-green-400 text-sm">
          <span className="w-1.5 h-1.5 rounded-full bg-green-400" aria-hidden="true" />
          <span>{truncateAddress(wallet.address)}</span>
        </div>
        <button
          onClick={disconnect}
          className="text-sm text-brand-muted hover:text-white transition-colors"
        >
          Disconnect
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-end gap-1.5">
      <button
        onClick={() => connect(0)}
        disabled={loading}
        className="flex items-center gap-2 px-4 py-2 bg-brand-gold text-brand-dark font-semibold rounded-lg hover:bg-brand-amber transition-colors text-sm disabled:opacity-60 disabled:cursor-not-allowed"
        aria-busy={loading}
      >
        {loading && <LoadingSpinner size="sm" />}
        {STEP_LABELS[step]}
      </button>

      {error && (
        <div role="alert" className="flex flex-col items-end gap-1">
          <p className="text-red-400 text-xs max-w-[220px] text-right">{error}</p>
          <button
            onClick={handleRetry}
            className="text-xs text-brand-gold hover:text-brand-amber underline transition-colors"
          >
            Retry
          </button>
        </div>
      )}

      {loading && retryCount > 0 && (
        <p className="text-brand-muted text-xs">
          Retry attempt {retryCount}/{MAX_RETRIES}…
        </p>
      )}
    </div>
  );
}
