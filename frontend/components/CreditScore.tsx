interface Props {
  paid: number;
  funded: number;
  defaulted: number;
  totalVolume: bigint;
}

export default function CreditScore({ paid, funded, defaulted, totalVolume }: Props) {
  const total = paid + funded + defaulted;

  // Simple score: 300–850 based on repayment rate and volume
  const repaymentRate = total > 0 ? paid / total : 0;
  const volumeBonus = Math.min(Number(totalVolume) / 1e10, 50); // up to 50 pts
  const score = Math.round(300 + repaymentRate * 500 + volumeBonus);

  const scoreColor =
    score >= 750 ? 'text-green-400' : score >= 600 ? 'text-yellow-400' : 'text-red-400';
  const scoreLabel =
    score >= 750 ? 'Excellent' : score >= 650 ? 'Good' : score >= 550 ? 'Fair' : 'Building';

  const arc = Math.round(((score - 300) / 550) * 180); // 0–180 degrees

  return (
    <div className="p-6 bg-brand-card border border-brand-border rounded-2xl">
      <h2 className="text-lg font-semibold mb-6">On-Chain Credit Score</h2>

      {/* Score display */}
      <div className="text-center mb-6">
        <div className={`text-5xl font-bold mb-1 ${scoreColor}`}>{score}</div>
        <div className="text-brand-muted text-sm">{scoreLabel}</div>
        <div className="text-xs text-brand-muted/60 mt-1">Based on {total} invoice(s)</div>
      </div>

      {/* Breakdown */}
      <div className="space-y-3">
        <ScoreRow label="Paid on time" count={paid} color="bg-green-500" total={total} />
        <ScoreRow label="Currently funded" count={funded} color="bg-blue-500" total={total} />
        <ScoreRow label="Defaulted" count={defaulted} color="bg-red-500" total={total} />
      </div>

      {total === 0 && (
        <p className="text-center text-brand-muted text-sm mt-4">
          Create and repay invoices to build your score.
        </p>
      )}
    </div>
  );
}

function ScoreRow({
  label,
  count,
  color,
  total,
}: {
  label: string;
  count: number;
  color: string;
  total: number;
}) {
  const pct = total > 0 ? (count / total) * 100 : 0;
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-brand-muted">{label}</span>
        <span className="font-medium">{count}</span>
      </div>
      <div className="h-1.5 bg-brand-border rounded-full overflow-hidden">
        <div
          className={`h-full ${color} rounded-full transition-all`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
