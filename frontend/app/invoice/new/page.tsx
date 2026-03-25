'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useStore } from '@/lib/store';
import { buildCreateInvoiceTx, submitTx } from '@/lib/contracts';
import { toStroops } from '@/lib/stellar';

export default function NewInvoicePage() {
  const { wallet } = useStore();
  const router = useRouter();

  const [form, setForm] = useState({
    debtor: '',
    amount: '',
    dueDate: '',
    description: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function handleChange(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) {
    setForm((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!wallet.address) return;

    setLoading(true);
    setError(null);

    try {
      const dueTimestamp = Math.floor(new Date(form.dueDate).getTime() / 1000);
      const amountStroops = toStroops(parseFloat(form.amount));

      const xdr = await buildCreateInvoiceTx({
        owner: wallet.address,
        debtor: form.debtor,
        amount: amountStroops,
        dueDate: dueTimestamp,
        description: form.description,
      });

      const freighter = await import('@stellar/freighter-api');
      const { signedTxXdr, error: signError } = await freighter.signTransaction(xdr, {
        networkPassphrase: 'Test SDF Network ; September 2015',
        address: wallet.address,
      });
      if (signError) throw new Error(signError.message);

      await submitTx(signedTxXdr);
      router.push('/dashboard');
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Transaction failed.';
      setError(msg);
    } finally {
      setLoading(false);
    }
  }

  const minDate = new Date(Date.now() + 86_400_000).toISOString().split('T')[0];

  return (
    <div className="min-h-screen pt-24 pb-16 px-6">
      <div className="max-w-xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-1">Tokenize Invoice</h1>
          <p className="text-brand-muted">
            Mint your unpaid invoice as a Soroban RWA token to access instant liquidity.
          </p>
        </div>

        {!wallet.connected ? (
          <div className="p-12 bg-brand-card border border-brand-border rounded-2xl text-center">
            <p className="text-brand-muted">Connect your wallet first.</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-5">
            <div className="p-6 bg-brand-card border border-brand-border rounded-2xl space-y-5">
              <Field
                label="Debtor (who owes you)"
                name="debtor"
                placeholder="ACME Corporation Ltd."
                value={form.debtor}
                onChange={handleChange}
                required
              />

              <div>
                <label className="block text-sm text-brand-muted mb-2">Invoice Amount (USDC)</label>
                <div className="relative">
                  <input
                    type="number"
                    name="amount"
                    min="10"
                    step="0.01"
                    placeholder="0.00"
                    value={form.amount}
                    onChange={handleChange}
                    required
                    className="w-full bg-brand-dark border border-brand-border rounded-xl px-4 py-3 text-white placeholder-brand-muted focus:outline-none focus:border-brand-gold text-lg"
                  />
                  <span className="absolute right-4 top-1/2 -translate-y-1/2 text-brand-muted text-sm font-medium">
                    USDC
                  </span>
                </div>
              </div>

              <div>
                <label className="block text-sm text-brand-muted mb-2">Due Date</label>
                <input
                  type="date"
                  name="dueDate"
                  min={minDate}
                  value={form.dueDate}
                  onChange={handleChange}
                  required
                  className="w-full bg-brand-dark border border-brand-border rounded-xl px-4 py-3 text-white focus:outline-none focus:border-brand-gold"
                />
              </div>

              <div>
                <label className="block text-sm text-brand-muted mb-2">
                  Description <span className="text-brand-muted/60">(optional)</span>
                </label>
                <textarea
                  name="description"
                  rows={3}
                  placeholder="Invoice #001 - Goods delivery, 500 units..."
                  value={form.description}
                  onChange={handleChange}
                  className="w-full bg-brand-dark border border-brand-border rounded-xl px-4 py-3 text-white placeholder-brand-muted focus:outline-none focus:border-brand-gold resize-none"
                />
              </div>
            </div>

            {/* Summary */}
            {form.amount && form.dueDate && (
              <div className="p-4 bg-brand-gold/10 border border-brand-gold/20 rounded-xl text-sm space-y-2">
                <p className="text-brand-gold font-medium">Invoice Summary</p>
                <div className="flex justify-between text-brand-muted">
                  <span>Invoice amount</span>
                  <span className="text-white">
                    ${parseFloat(form.amount || '0').toLocaleString()} USDC
                  </span>
                </div>
                <div className="flex justify-between text-brand-muted">
                  <span>Due date</span>
                  <span className="text-white">
                    {new Date(form.dueDate).toLocaleDateString('en-US', {
                      year: 'numeric',
                      month: 'short',
                      day: 'numeric',
                    })}
                  </span>
                </div>
                <div className="flex justify-between text-brand-muted">
                  <span>Estimated repayment (8% APY)</span>
                  <span className="text-white">
                    ${(parseFloat(form.amount || '0') * 1.08).toFixed(2)} USDC
                  </span>
                </div>
              </div>
            )}

            {error && (
              <div className="p-3 bg-red-900/20 border border-red-800/50 rounded-xl text-red-400 text-sm">
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={loading}
              className="w-full py-3.5 bg-brand-gold text-brand-dark font-semibold rounded-xl hover:bg-brand-amber transition-colors disabled:opacity-60 text-lg"
            >
              {loading ? 'Minting on Stellar...' : 'Mint Invoice Token'}
            </button>

            <p className="text-xs text-brand-muted text-center">
              Your invoice will be tokenized on Stellar Testnet. Gas fees are under $0.01.
            </p>
          </form>
        )}
      </div>
    </div>
  );
}

function Field({
  label,
  name,
  placeholder,
  value,
  onChange,
  required,
}: {
  label: string;
  name: string;
  placeholder: string;
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  required?: boolean;
}) {
  return (
    <div>
      <label className="block text-sm text-brand-muted mb-2">{label}</label>
      <input
        type="text"
        name={name}
        placeholder={placeholder}
        value={value}
        onChange={onChange}
        required={required}
        className="w-full bg-brand-dark border border-brand-border rounded-xl px-4 py-3 text-white placeholder-brand-muted focus:outline-none focus:border-brand-gold"
      />
    </div>
  );
}
