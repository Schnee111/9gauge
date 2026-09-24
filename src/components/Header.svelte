<script lang="ts">
  import type { ConnectionState } from '../types';

  interface Props {
    host: string;
    connection: ConnectionState;
    period: string;
    onPeriodChange: (p: string) => void;
    onOpenSettings?: () => void;
  }

  let { host, connection, period, onPeriodChange, onOpenSettings }: Props = $props();

  let isDropdownOpen = $state(false);

  const periods = [
    { label: 'Today', value: 'today' },
    { label: 'Past 24 Hours', value: '24h' },
    { label: 'Past 7 Days', value: '7d' },
    { label: 'Past 30 Days', value: '30d' },
    { label: 'All Time', value: 'all' },
  ];

  const currentLabel = $derived(
    periods.find((p) => p.value === period)?.label || 'Today'
  );

  function getStatusClasses(status: ConnectionState) {
    switch (status) {
      case 'Healthy':
        return 'bg-emerald-500 shadow-[0_0_8px_rgba(46,158,107,0.8)] animate-breathe';
      case 'Connecting':
        return 'bg-amber-400 shadow-[0_0_6px_rgba(217,154,43,0.6)] animate-pulse';
      case 'Degraded':
        return 'bg-amber-500 shadow-[0_0_6px_rgba(217,154,43,0.8)]';
      case 'Reconnecting':
        return 'bg-rose-500 shadow-[0_0_8px_rgba(212,85,63,0.8)] animate-pulse';
      case 'AuthFailed':
        return 'bg-rose-600 shadow-[0_0_8px_rgba(212,85,63,1)]';
      case 'Disconnected':
      default:
        return 'bg-slate-500';
    }
  }

  function handleSelect(val: string) {
    onPeriodChange(val);
    isDropdownOpen = false;
  }
</script>

<header class="flex items-center justify-between px-3.5 py-2.5 border-b border-white/[0.08] bg-white/[0.02]">
  <!-- Left Breadcrumb -->
  <div class="flex items-center space-x-2">
    <!-- Breathing Status Dot -->
    <span
      class="inline-block w-2 h-2 rounded-full transition-all duration-300 {getStatusClasses(connection)}"
      title="Connection: {connection}"
    ></span>

    <div class="flex items-center space-x-1.5 text-[11px] font-semibold tracking-wider uppercase text-slate-300">
      <span class="text-slate-400 font-mono">✦</span>
      <span class="font-medium text-white tracking-widest">9GAUGE</span>
      <span class="text-slate-600">/</span>
      <span class="text-slate-400 font-normal">TELEMETRY</span>
    </div>
  </div>

  <!-- Right Host & Period Controls -->
  <div class="flex items-center space-x-2 relative">
    <button
      type="button"
      class="flex items-center space-x-1 px-2 py-0.5 rounded text-[11px] font-mono text-slate-300 bg-white/[0.05] hover:bg-white/[0.1] border border-white/[0.1] transition-colors"
      onclick={() => (isDropdownOpen = !isDropdownOpen)}
    >
      <span class="truncate max-w-[110px]" title={host}>{currentLabel}</span>
      <span class="text-slate-400 text-[9px]">▾</span>
    </button>

    {#if isDropdownOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="fixed inset-0 z-40"
        onclick={() => (isDropdownOpen = false)}
      ></div>
      <div
        class="absolute right-0 top-7 z-50 w-36 py-1 rounded-md glass-panel border border-white/[0.12] shadow-2xl text-[11px] font-medium"
      >
        {#each periods as p}
          <button
            type="button"
            class="w-full text-left px-3 py-1.5 hover:bg-white/[0.08] transition-colors flex items-center justify-between {p.value === period ? 'text-emerald-400 font-semibold' : 'text-slate-300'}"
            onclick={() => handleSelect(p.value)}
          >
            <span>{p.label}</span>
            {#if p.value === period}
              <span class="text-[10px]">✓</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}

    {#if onOpenSettings}
      <button
        type="button"
        class="p-1 rounded text-slate-400 hover:text-white hover:bg-white/[0.08] transition-colors"
        title="Settings"
        onclick={onOpenSettings}
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
      </button>
    {/if}
  </div>
</header>
