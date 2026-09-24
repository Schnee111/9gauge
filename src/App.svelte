<script lang="ts">
  import { onMount } from 'svelte';
  import type { AppState, UsageSnapshot, ConnectionState } from './types';
  import Header from './components/Header.svelte';
  import HeroTokenMeter from './components/HeroTokenMeter.svelte';
  import ProviderList from './components/ProviderList.svelte';
  import MicroFeed from './components/MicroFeed.svelte';
  import Footer from './components/Footer.svelte';

  const DEFAULT_SNAPSHOT: UsageSnapshot = {
    totalRequests: 0,
    totalPromptTokens: 0,
    totalCompletionTokens: 0,
    totalCachedTokens: 0,
    totalCost: 0,
    byProvider: {},
    byModel: {},
    byAccount: {},
    last10Minutes: [],
    activeRequests: [],
    recentRequests: [],
    errorProvider: '',
  };

  let appState: AppState = $state({
    snapshot: DEFAULT_SNAPSHOT,
    connection: 'Connecting',
    host: 'localhost:20128',
  });

  let period = $state('today');

  const totalTokens = $derived(
    appState.snapshot.totalPromptTokens + appState.snapshot.totalCompletionTokens
  );

  const periodLabels: Record<string, string> = {
    today: 'Today',
    '24h': 'Past 24h',
    '7d': 'Past 7d',
    '30d': 'Past 30d',
    all: 'All Time',
  };

  const periodLabel = $derived(periodLabels[period] || 'Today');

  onMount(() => {
    // Check if running inside Tauri
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    if (isTauri) {
      // In Tauri runtime: listen for IPC telemetry state broadcast
      import('@tauri-apps/api/event').then(({ listen }) => {
        listen<AppState>('telemetry:state', (event) => {
          appState = event.payload;
        });
      }).catch((e) => {
        console.warn('Tauri event listen failed:', e);
      });
    } else {
      // Standalone browser preview: load mock / preview data
      appState = {
        snapshot: {
          totalRequests: 144898,
          totalPromptTokens: 16813439549,
          totalCompletionTokens: 80131034,
          totalCachedTokens: 8925056095,
          totalCost: 17.42,
          byProvider: {
            antigravity: {
              requests: 1801,
              promptTokens: 230418450,
              completionTokens: 906289,
              cachedTokens: 199137681,
              cost: 9.51,
            },
            'openai-compatible-chat-d27df55a-127c-46a7-a87f-eeed145da7bb': {
              requests: 4036,
              promptTokens: 536813516,
              completionTokens: 2989144,
              cachedTokens: 0,
              cost: 7.91,
            },
          },
          byModel: {},
          byAccount: {},
          last10Minutes: [
            { requests: 45, promptTokens: 4809576, completionTokens: 18363, cost: 0.12 },
            { requests: 28, promptTokens: 4350225, completionTokens: 7933, cost: 0.08 },
          ],
          activeRequests: [],
          recentRequests: [
            {
              timestamp: new Date().toISOString(),
              model: 'gemini-3.8-flash-high',
              provider: 'antigravity',
              promptTokens: 58227,
              completionTokens: 96,
              cachedTokens: 55680,
              status: '200 OK',
            },
            {
              timestamp: new Date(Date.now() - 45000).toISOString(),
              model: 'gfmodel',
              provider: 'qoder',
              promptTokens: 12450,
              completionTokens: 320,
              cachedTokens: 8900,
              status: '200 OK',
            },
          ],
          errorProvider: '',
        },
        connection: 'Healthy',
        host: 'localhost:20128',
      };
    }
  });

  function handlePeriodChange(newPeriod: string) {
    period = newPeriod;
    // Notify backend if inside Tauri
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('set_period', { period: newPeriod });
      });
    }
  }

  function handleQuit() {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/core').then(({ invoke }) => {
        invoke('quit_app');
      });
    }
  }
</script>

<main class="w-[360px] max-h-[480px] flex flex-col rounded-xl overflow-hidden glass-panel border border-white/[0.14] text-slate-100 shadow-2xl bg-[#0d0f14]/85">
  <Header
    host={appState.host}
    connection={appState.connection}
    {period}
    onPeriodChange={handlePeriodChange}
  />

  <div class="overflow-y-auto flex-1">
    <HeroTokenMeter
      snapshot={appState.snapshot}
      {periodLabel}
    />

    <ProviderList
      byProvider={appState.snapshot.byProvider}
      {totalTokens}
      errorProvider={appState.snapshot.errorProvider}
    />

    <MicroFeed
      requests={appState.snapshot.recentRequests}
    />
  </div>

  <Footer
    cachedTokens={appState.snapshot.totalCachedTokens}
    {totalTokens}
    hostUrl={`http://${appState.host}`}
    onQuit={handleQuit}
  />
</main>
