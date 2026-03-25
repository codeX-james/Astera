import type { Invoice } from '@/lib/types';
import { formatUSDC, formatDate, daysUntil } from '@/lib/stellar';
import Link from 'next/link';

interface Props {
  invoice: Invoice;
}

const statusLabel: Record<string, string> = {
  Pending: 'Pending',
  Funded: 'Funded',
  Paid: 'Paid',
  Defaulted: 'Defaulted',
};

export default function InvoiceCard({ invoice }: Props) {
  const days = daysUntil(invoice.dueDate);
  const isOverdue = days < 0;

  return (
    <Link
      href={`/invoice/${invoice.id}`}
      className="block p-5 bg-brand-card border border-brand-border rounded-2xl hover:border-brand-gold/30 transition-colors group"
    >
      <div className="flex items-start justify-between mb-4">
        <div>
          <p className="text-xs text-brand-muted mb-1">Invoice #{invoice.id}</p>
          <h3 className="font-semibold text-lg group-hover:text-brand-gold transition-colors">
            {invoice.debtor}
          </h3>
        </div>
        <span
          className={`text-xs font-medium px-2.5 py-1 rounded-full badge-${invoice.status.toLowerCase()}`}
        >
          {statusLabel[invoice.status]}
        </span>
      </div>

      <div className="text-2xl font-bold mb-4">{formatUSDC(invoice.amount)}</div>

      <div className="flex items-center justify-between text-sm text-brand-muted">
        <div>
          Due <span className="text-white">{formatDate(invoice.dueDate)}</span>
        </div>
        <div
          className={
            isOverdue ? 'text-red-400' : days <= 7 ? 'text-yellow-400' : 'text-brand-muted'
          }
        >
          {isOverdue ? `${Math.abs(days)}d overdue` : `${days}d left`}
        </div>
      </div>

      {invoice.description && (
        <p className="mt-3 text-xs text-brand-muted line-clamp-2 border-t border-brand-border pt-3">
          {invoice.description}
        </p>
      )}
    </Link>
  );
}
