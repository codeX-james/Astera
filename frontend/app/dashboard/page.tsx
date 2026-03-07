"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useStore } from "@/lib/store";
import InvoiceCard from "@/components/InvoiceCard";
import CreditScore from "@/components/CreditScore";
import { getInvoice, getInvoiceCount } from "@/lib/contracts";
import { formatUSDC } from "@/lib/stellar";
import type { Invoice } from "@/lib/types";

export default function DashboardPage() {
  const { wallet } = useStore();
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!wallet.connected) {
      setLoading(false);
      return;
    }
    loadInvoices();
  }, [wallet.connected, wallet.address]);

  async function loadInvoices() {
    setLoading(true);
    setError(null);
    try {
      const count = await getInvoiceCount();
      const all: Invoice[] = [];
      for (let i = 1; i <= count; i++) {
        const inv = await getInvoice(i);
        if (inv.owner === wallet.address) all.push(inv);
      }
      setInvoices(all);
    } catch (e) {
      setError("Failed to load invoices. Make sure contracts are deployed.");
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  const stats = {
    total: invoices.length,
    pending: invoices.filter((i) => i.status === "Pending").length,
    funded: invoices.filter((i) => i.status === "Funded").length,
    paid: invoices.filter((i) => i.status === "Paid").length,
    defaulted: invoices.filter((i) => i.status === "Defaulted").length,
    totalVolume: invoices.reduce((acc, i) => acc + i.amount, 0n),
  };

  return (
    <div className="min-h-screen pt-24 pb-16 px-6">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold mb-1">SME Dashboard</h1>
            <p className="text-brand-muted">Manage your tokenized invoices</p>
          </div>
          {wallet.connected && (
            <Link
              href="/invoice/new"
              className="px-5 py-2.5 bg-brand-gold text-brand-dark font-semibold rounded-xl hover:bg-brand-amber transition-colors"
            >
              + New Invoice
            </Link>
          )}
        </div>

        {!wallet.connected ? (
          <div className="flex flex-col items-center justify-center py-32 text-center">
            <div className="text-4xl mb-4">◈</div>
            <h2 className="text-xl font-semibold mb-2">Connect your wallet</h2>
            <p className="text-brand-muted">
              Connect Freighter to view and manage your invoices.
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Left column */}
            <div className="lg:col-span-2 space-y-6">
              {/* Quick stats */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
                {[
                  { label: "Total Volume", value: formatUSDC(stats.totalVolume), highlight: true },
                  { label: "Pending", value: stats.pending.toString() },
                  { label: "Funded", value: stats.funded.toString() },
                  { label: "Paid", value: stats.paid.toString() },
                ].map((s) => (
                  <div key={s.label} className="p-4 bg-brand-card border border-brand-border rounded-xl">
                    <p className="text-xs text-brand-muted mb-1">{s.label}</p>
                    <p className={`text-xl font-bold ${s.highlight ? "gradient-text" : ""}`}>
                      {s.value}
                    </p>
                  </div>
                ))}
              </div>

              {/* Invoices */}
              <div>
                <h2 className="text-lg font-semibold mb-4">Your Invoices</h2>
                {loading ? (
                  <div className="space-y-4">
                    {[1, 2, 3].map((n) => (
                      <div key={n} className="h-32 bg-brand-card border border-brand-border rounded-2xl animate-pulse" />
                    ))}
                  </div>
                ) : error ? (
                  <div className="p-4 bg-red-900/20 border border-red-800/50 rounded-xl text-red-400 text-sm">
                    {error}
                  </div>
                ) : invoices.length === 0 ? (
                  <div className="p-12 bg-brand-card border border-brand-border rounded-2xl text-center">
                    <p className="text-brand-muted mb-4">No invoices yet.</p>
                    <Link
                      href="/invoice/new"
                      className="text-brand-gold hover:underline text-sm font-medium"
                    >
                      Create your first invoice →
                    </Link>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {invoices.map((inv) => (
                      <InvoiceCard key={inv.id} invoice={inv} />
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* Right column */}
            <div>
              <CreditScore
                paid={stats.paid}
                funded={stats.funded}
                defaulted={stats.defaulted}
                totalVolume={stats.totalVolume}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
