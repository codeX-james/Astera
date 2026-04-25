'use client';

import { useEffect, useState } from 'react';
import toast from 'react-hot-toast';
import { useStore } from '@/lib/store';
import { Skeleton } from '@/components/Skeleton';
import {
  getPoolConfig,
  buildSetYieldTx,
  buildSetFactoringFeeTx,
  submitTx,
} from '@/lib/contracts';

export default function AdminYieldPage() {
  const { wallet, poolConfig, setPoolConfig } = useStore();
  const [newYield, setNewYield] = useState('');
  const [newFactoringFee, setNewFactoringFee] = useState('');
  const [loading, setLoading] = useState(false);
  const [txLoading, setTxLoading] = useState(false);

  useEffect(() => {
    async function load() {
      setLoading(true);
      try {
        const config = await getPoolConfig();
        setPoolConfig(config);
        setNewYield((config.yieldBps / 100).toString());
        setNewFactoringFee((config.factoringFeeBps / 100).toString());
      } catch (e) {
        console.error(e);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [setPoolConfig]);

  if (loading) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-10 w-48 rounded-lg" />
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Skeleton className="h-48 rounded-2xl" />
          <Skeleton className="h-48 rounded-2xl" />
        </div>
      </div>
    );
  }

  async function signAndSubmit(xdr: string) {
    const freighter = await import('@stellar/freighter-api');
    const { signedTxXdr, error: signError } = await freighter.signTransaction(xdr, {
      networkPassphrase: 'Test SDF Network ; September 2015',
      address: wallet.address!,
    });

    if (signError) throw new Error(signError.message || 'Signing rejected.');
    await submitTx(signedTxXdr);
  }

  async function handleYieldSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!wallet.address) return;

    const bps = Math.round(parseFloat(newYield) * 100);
    if (isNaN(bps) || bps < 0 || bps > 5000) {
      toast.error('Yield must be between 0% and 50% (5000 bps).');
      return;
    }

    setTxLoading(true);

    try {
      const xdr = await buildSetYieldTx(wallet.address, bps);
      await signAndSubmit(xdr);
      toast.success(`Yield rate updated to ${newYield}% (${bps} bps).`);

      const updatedConfig = await getPoolConfig();
      setPoolConfig(updatedConfig);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Failed to update yield rate.';
      toast.error(msg);
      console.error(e);
    } finally {
      setTxLoading(false);
    }
  }

  async function handleFactoringFeeSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!wallet.address) return;

    const bps = Math.round(parseFloat(newFactoringFee) * 100);
    if (isNaN(bps) || bps < 0 || bps > 10000) {
      toast.error('Factoring fee must be between 0% and 100% (10000 bps).');
      return;
    }

    setTxLoading(true);

    try {
      const xdr = await buildSetFactoringFeeTx(wallet.address, bps);
      await signAndSubmit(xdr);
      toast.success(`Factoring fee updated to ${newFactoringFee}% (${bps} bps).`);

      const updatedConfig = await getPoolConfig();
      setPoolConfig(updatedConfig);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Failed to update factoring fee.';
      toast.error(msg);
      console.error(e);
    } finally {
      setTxLoading(false);
    }
  }

  return (
    <div className="max-w-2xl space-y-8">
      <div>
        <h1 className="text-3xl font-bold mb-2">Pool Fee Management</h1>
        <p className="text-brand-muted text-sm">
          Configure borrower pricing for the Astera liquidity pool.
        </p>
      </div>

      <div className="p-8 bg-brand-card border border-brand-border rounded-2xl shadow-sm">
        <label className="block text-sm font-semibold text-brand-muted mb-6 uppercase tracking-wider">
          Current Configuration
        </label>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-8">
          <div className="p-4 bg-brand-dark rounded-xl border border-brand-border">
            <p className="text-xs text-brand-muted mb-1">Current Yield</p>
            <p className="text-2xl font-bold text-white">
              {loading ? '...' : ((poolConfig?.yieldBps ?? 0) / 100).toFixed(2)}%
            </p>
          </div>
          <div className="p-4 bg-brand-dark rounded-xl border border-brand-border">
            <p className="text-xs text-brand-muted mb-1">Current Factoring Fee</p>
            <p className="text-2xl font-bold text-brand-gold">
              {loading ? '...' : ((poolConfig?.factoringFeeBps ?? 0) / 100).toFixed(2)}%
            </p>
          </div>
        </div>

        <form onSubmit={handleYieldSubmit} className="space-y-6 pt-6 border-t border-brand-border">
          <div>
            <label className="block text-sm font-medium text-white mb-2">New Yield Rate (%)</label>
            <div className="relative">
              <input
                type="number"
                step="0.01"
                min="0"
                max="50"
                value={newYield}
                onChange={(e) => setNewYield(e.target.value)}
                className="w-full bg-brand-dark border border-brand-border rounded-xl px-4 py-3 text-white placeholder-brand-muted focus:outline-none focus:border-brand-gold text-lg"
                placeholder="e.g. 8.5"
                required
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-brand-muted font-bold">
                %
              </span>
            </div>
            <p className="mt-2 text-xs text-brand-muted">
              Example: 8.5% is equivalent to 850 basis points.
            </p>
          </div>

          <button
            type="submit"
            disabled={txLoading || loading}
            className="w-full py-4 bg-brand-gold text-brand-dark font-bold rounded-xl hover:bg-brand-amber transition-all shadow-lg active:scale-[0.98] disabled:opacity-50"
          >
            {txLoading ? 'Updating Rate...' : 'Update Yield Rate'}
          </button>
        </form>

        <form
          onSubmit={handleFactoringFeeSubmit}
          className="space-y-6 pt-6 mt-6 border-t border-brand-border"
        >
          <div>
            <label className="block text-sm font-medium text-white mb-2">
              New Factoring Fee (%)
            </label>
            <div className="relative">
              <input
                type="number"
                step="0.01"
                min="0"
                max="100"
                value={newFactoringFee}
                onChange={(e) => setNewFactoringFee(e.target.value)}
                className="w-full bg-brand-dark border border-brand-border rounded-xl px-4 py-3 text-white placeholder-brand-muted focus:outline-none focus:border-brand-gold text-lg"
                placeholder="e.g. 2.5"
                required
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-brand-muted font-bold">
                %
              </span>
            </div>
            <p className="mt-2 text-xs text-brand-muted">
              This fee is locked when an invoice becomes fully funded and is charged on top of
              borrower interest at repayment.
            </p>
          </div>

          <button
            type="submit"
            disabled={txLoading || loading}
            className="w-full py-4 bg-white text-brand-dark font-bold rounded-xl hover:bg-stone-200 transition-all shadow-lg active:scale-[0.98] disabled:opacity-50"
          >
            {txLoading ? 'Updating Fee...' : 'Update Factoring Fee'}
          </button>
        </form>
      </div>

      <div className="p-6 bg-brand-dark border border-brand-border rounded-2xl text-xs text-brand-muted space-y-2">
        <p className="font-bold text-white mb-1 uppercase tracking-tighter">Safety Controls:</p>
        <p>• The contract enforces a maximum yield of 50.00% (5000 bps).</p>
        <p>• The contract enforces a maximum factoring fee of 100.00% (10000 bps).</p>
        <p>• Yield changes apply to active and new funded invoices at repayment time.</p>
        <p>• Factoring fees are locked when an invoice becomes fully funded.</p>
      </div>
    </div>
  );
}
