'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { StatCardSkeleton, ChartSkeleton } from '@/components/Skeleton';
import { getPoolConfig, getAcceptedTokens, getPoolTokenTotals } from '@/lib/contracts';
import { formatUSDC, stablecoinLabel } from '@/lib/stellar';
import type { PoolConfig, PoolTokenTotals } from '@/lib/types';

const POOL_CONFIGURED = Boolean(process.env.NEXT_PUBLIC_POOL_CONTRACT_ID);

interface TokenData {
  token: string;
  totals: PoolTokenTotals;
}

function StatCard({
  label,
  value,
  sub,
  highlight,
}: {
  label: string;
  value: string;
  sub?: string;
  highlight?: boolean;
}) {
  return (
    <div className="p-5 bg-brand-card border border-brand-border rounded-2xl">
      <p className="text-xs text-brand-muted mb-1">{label}</p>
      <p className={`text-2xl font-bold truncate ${highlight ? 'text-brand-gold' : 'text-white'}`}>
        {value}
      </p>
      {sub && <p className="text-xs text-brand-muted mt-1">{sub}</p>}
    </div>
  );
}

function UtilizationBar({ deployed, total }: { deployed: bigint; total: bigint }) {
  const pct = total > 0n ? Number((deployed * 100n) / total) : 0;
  const barColor =
    pct > 90 ? 'bg-red-500' : pct > 70 ? 'bg-yellow-400' : 'bg-brand-gold';
  const textColor = pct > 90 ? 'text-red-400' : pct > 70 ? 'text-yellow-400' : 'text-white';
  return (
    <div className="space-y-1.5">
      <div className="flex justify-between text-xs text-brand-muted">
        <span>Utilization</span>
        <span className={textColor}>{pct.toFixed(1)}%</span>
      </div>
      <div className="h-2.5 bg-brand-dark rounded-full overflow-hidden border border-brand-border">
        <div
          className={`h-full rounded-full transition-all duration-500 ${barColor}`}
          style={{ width: `${Math.min(pct, 100)}%` }}
        />
      </div>
    </div>
  );
}

export default function AnalyticsPage() {
  const [config, setConfig] = useState<PoolConfig | null>(null);
  const [tokens, setTokens] = useState<TokenData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!POOL_CONFIGURED) {
      setLoading(false);
      return;
    }
    load();
  }, []);

  async function load() {
    try {
      const [cfg, acceptedTokens] = await Promise.all([getPoolConfig(), getAcceptedTokens()]);
      setConfig(cfg);
      const tokenData = await Promise.all(
        acceptedTokens.map(async (token) => ({
          token,
          totals: await getPoolTokenTotals(token),
        })),
      );
      setTokens(tokenData);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load analytics data.');
    } finally {
      setLoading(false);
    }
  }

  // Aggregate across all tokens (assumes 1:1 USD peg for stablecoins)
  const agg = tokens.reduce(
    (acc, { totals }) => ({
      poolValue: acc.poolValue + totals.totalDeposited,
      deployed: acc.deployed + totals.totalDeployed,
      paidOut: acc.paidOut + totals.totalPaidOut,
      feeRevenue: acc.feeRevenue + totals.totalFeeRevenue,
    }),
    { poolValue: 0n, deployed: 0n, paidOut: 0n, feeRevenue: 0n },
  );

  const available = agg.poolValue - agg.deployed;
  const apy = config ? (config.yieldBps / 100).toFixed(2) : '–';
  const factoringFee = config ? (config.factoringFeeBps / 100).toFixed(2) : '–';

  return (
    <div className="min-h-screen pt-24 pb-16 px-6">
      <div className="max-w-5xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-1">Pool Analytics</h1>
          <p className="text-brand-muted">
            Real-time performance metrics for the Astera liquidity pool.
          </p>
        </div>

        {!POOL_CONFIGURED && (
          <div className="p-6 bg-brand-card border border-brand-border rounded-2xl text-brand-muted text-sm">
            Pool contracts are not yet deployed. Configure{' '}
            <code className="text-brand-gold text-xs">NEXT_PUBLIC_POOL_CONTRACT_ID</code> to see
            live data.
          </div>
        )}

        {POOL_CONFIGURED && loading && (
          <div className="space-y-6">
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
              <StatCardSkeleton />
              <StatCardSkeleton />
              <StatCardSkeleton />
              <StatCardSkeleton />
            </div>
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <ChartSkeleton />
              <ChartSkeleton />
            </div>
          </div>
        )}

        {POOL_CONFIGURED && error && (
          <div className="p-4 bg-red-900/20 border border-red-800/50 rounded-2xl text-red-400 text-sm">
            {error}
          </div>
        )}

        {POOL_CONFIGURED && !loading && !error && (
          <div className="space-y-6">
            {/* Key metrics */}
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
              <StatCard
                label="Total Pool Value"
                value={formatUSDC(agg.poolValue)}
                sub="Net asset value"
                highlight
              />
              <StatCard
                label="Deployed Capital"
                value={formatUSDC(agg.deployed)}
                sub="Funding active invoices"
              />
              <StatCard
                label="Available Liquidity"
                value={formatUSDC(available)}
                sub="Ready to deploy"
              />
              <StatCard
                label="All-time Repaid"
                value={formatUSDC(agg.paidOut)}
                sub="Cumulative repayments"
              />
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Yield configuration */}
              <div className="p-6 bg-brand-card border border-brand-border rounded-2xl">
                <h2 className="text-lg font-semibold mb-4">Yield &amp; Fee Configuration</h2>
                <div className="space-y-3">
                  {[
                    { label: 'Target APY', value: `${apy}%`, highlight: true },
                    { label: 'Factoring Fee', value: `${factoringFee}%` },
                    {
                      label: 'Interest Mode',
                      value: config?.compoundInterest ? 'Compound' : 'Simple',
                    },
                    {
                      label: 'All-time Fee Revenue',
                      value: formatUSDC(agg.feeRevenue),
                      highlight: true,
                    },
                  ].map((r) => (
                    <div key={r.label} className="flex justify-between items-center text-sm">
                      <span className="text-brand-muted">{r.label}</span>
                      <span
                        className={`font-semibold ${r.highlight ? 'text-brand-gold' : 'text-white'}`}
                      >
                        {r.value}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Capital allocation per token */}
              <div className="p-6 bg-brand-card border border-brand-border rounded-2xl">
                <h2 className="text-lg font-semibold mb-4">Capital Allocation</h2>
                {tokens.length === 0 ? (
                  <p className="text-brand-muted text-sm">No tokens configured.</p>
                ) : (
                  <div className="space-y-6">
                    {tokens.map(({ token, totals }) => {
                      const avail = totals.totalDeposited - totals.totalDeployed;
                      return (
                        <div key={token} className="space-y-3">
                          <p className="text-sm font-medium">{stablecoinLabel(token)}</p>
                          <UtilizationBar
                            deployed={totals.totalDeployed}
                            total={totals.totalDeposited}
                          />
                          <div className="grid grid-cols-3 gap-2 text-xs">
                            <div className="text-center p-2 bg-brand-dark rounded-lg">
                              <p className="text-brand-muted mb-0.5">Pool NAV</p>
                              <p className="text-white font-medium">
                                {formatUSDC(totals.totalDeposited)}
                              </p>
                            </div>
                            <div className="text-center p-2 bg-brand-dark rounded-lg">
                              <p className="text-brand-muted mb-0.5">Deployed</p>
                              <p className="text-brand-gold font-medium">
                                {formatUSDC(totals.totalDeployed)}
                              </p>
                            </div>
                            <div className="text-center p-2 bg-brand-dark rounded-lg">
                              <p className="text-brand-muted mb-0.5">Available</p>
                              <p className="text-white font-medium">{formatUSDC(avail)}</p>
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>

            {/* Multi-token breakdown table */}
            {tokens.length > 1 && (
              <div className="p-6 bg-brand-card border border-brand-border rounded-2xl">
                <h2 className="text-lg font-semibold mb-4">Token Breakdown</h2>
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="text-brand-muted border-b border-brand-border">
                        <th className="text-left py-2 pr-4 font-medium">Token</th>
                        <th className="text-right py-2 px-2 font-medium">Pool Value</th>
                        <th className="text-right py-2 px-2 font-medium">Deployed</th>
                        <th className="text-right py-2 px-2 font-medium">Available</th>
                        <th className="text-right py-2 px-2 font-medium">Total Repaid</th>
                        <th className="text-right py-2 pl-2 font-medium">Fee Revenue</th>
                      </tr>
                    </thead>
                    <tbody>
                      {tokens.map(({ token, totals }) => (
                        <tr
                          key={token}
                          className="border-b border-brand-border/50 last:border-0 hover:bg-brand-dark/40 transition-colors"
                        >
                          <td className="py-3 pr-4 font-medium">{stablecoinLabel(token)}</td>
                          <td className="py-3 px-2 text-right text-brand-gold font-medium">
                            {formatUSDC(totals.totalDeposited)}
                          </td>
                          <td className="py-3 px-2 text-right">
                            {formatUSDC(totals.totalDeployed)}
                          </td>
                          <td className="py-3 px-2 text-right">
                            {formatUSDC(totals.totalDeposited - totals.totalDeployed)}
                          </td>
                          <td className="py-3 px-2 text-right">
                            {formatUSDC(totals.totalPaidOut)}
                          </td>
                          <td className="py-3 pl-2 text-right">
                            {formatUSDC(totals.totalFeeRevenue)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* Default rate notice */}
            <div className="p-4 bg-brand-dark border border-brand-border rounded-2xl text-xs text-brand-muted">
              <p>
                Historical yield rates and default rate tracking require an off-chain indexer. The
                metrics above reflect the current on-chain state. Pool NAV grows as invoices are
                repaid with interest.
              </p>
            </div>

            <div className="flex items-center gap-4">
              <Link
                href="/invest"
                className="px-5 py-2.5 bg-brand-gold text-brand-dark font-semibold rounded-xl hover:bg-brand-amber transition-colors text-sm"
              >
                Invest Now
              </Link>
              <Link
                href="/portfolio"
                className="px-5 py-2.5 border border-brand-border text-white rounded-xl hover:border-brand-gold/50 transition-colors text-sm"
              >
                View Portfolio
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
