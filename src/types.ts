export interface CounterEntry {
  requests: number;
  promptTokens: number;
  completionTokens: number;
  cachedTokens: number;
  cost: number;
  rawModel?: string;
  provider?: string;
  accountName?: string;
  [key: string]: unknown;
}

export interface RecentRequest {
  timestamp: string;
  model: string;
  provider: string;
  promptTokens: number;
  completionTokens: number;
  cachedTokens: number;
  status: string;
}

export interface ActiveRequest {
  model: string;
  provider: string;
  account: string;
  count: number;
}

export interface ThroughputBucket {
  requests: number;
  promptTokens: number;
  completionTokens: number;
  cost: number;
}

export interface UsageSnapshot {
  totalRequests: number;
  totalPromptTokens: number;
  totalCompletionTokens: number;
  totalCachedTokens: number;
  totalCost: number;
  byProvider: Record<string, CounterEntry>;
  byModel: Record<string, CounterEntry>;
  byAccount: Record<string, CounterEntry>;
  last10Minutes: ThroughputBucket[];
  activeRequests: ActiveRequest[];
  recentRequests: RecentRequest[];
  errorProvider: string;
}

export type ConnectionState =
  | 'Connecting'
  | 'Healthy'
  | 'Degraded'
  | 'Disconnected'
  | 'Reconnecting'
  | 'AuthFailed';

export interface AppState {
  snapshot: UsageSnapshot;
  connection: ConnectionState;
  host: string;
}
