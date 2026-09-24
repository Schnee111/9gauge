import type { ThroughputBucket } from '../types';

export function formatNumber(n: number): string {
  return new Intl.NumberFormat('en-US').format(Math.round(n));
}

export function formatCompact(n: number): string {
  if (n >= 1_000_000_000) {
    return (n / 1_000_000_000).toFixed(1) + 'B';
  }
  if (n >= 1_000_000) {
    return (n / 1_000_000).toFixed(1) + 'M';
  }
  if (n >= 1_000) {
    return (n / 1_000).toFixed(1) + 'k';
  }
  return n.toString();
}

export function formatCurrency(amount: number): string {
  if (amount === 0) return '$0.00';
  if (amount < 0.01) return `< $0.01`;
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 3,
  }).format(amount);
}

export function formatTime(isoString: string): string {
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return isoString;
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    const ss = String(d.getSeconds()).padStart(2, '0');
    return `${hh}:${mm}:${ss}`;
  } catch {
    return isoString;
  }
}

export function calculateVelocity(buckets: ThroughputBucket[]): {
  tokPerHour: number;
  reqPerMin: number;
} {
  if (!buckets || buckets.length === 0) {
    return { tokPerHour: 0, reqPerMin: 0 };
  }

  // Use the last 5 minutes (or whatever is available) for smooth instantaneous burn
  const sample = buckets.slice(-5);
  const totalTokens = sample.reduce(
    (acc, b) => acc + (b.promptTokens || 0) + (b.completionTokens || 0),
    0
  );
  const totalRequests = sample.reduce((acc, b) => acc + (b.requests || 0), 0);
  const minutes = Math.max(sample.length, 1);

  const reqPerMin = totalRequests / minutes;
  const tokPerHour = (totalTokens / minutes) * 60;

  return { tokPerHour, reqPerMin };
}
