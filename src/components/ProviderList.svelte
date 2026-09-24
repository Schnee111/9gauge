<script lang="ts">
  import type { CounterEntry } from '../types';
  import { formatCompact, formatCurrency } from '../utils/format';

  interface Props {
    byProvider: Record<string, CounterEntry>;
    totalTokens: number;
    errorProvider?: string;
  }

  let { byProvider, totalTokens, errorProvider }: Props = $props();

  // Convert byProvider map into sorted array (highest volume first)
  const providerEntries = $derived(
    Object.entries(byProvider || {})
      .map(([name, stats]) => {
        const tokens = (stats.promptTokens || 0) + (stats.completionTokens || 0);
        const pct = totalTokens > 0 ? (tokens / totalTokens) * 100 : 0;
        return {
          name,
          stats,
          tokens,
          pct,
          isError: errorProvider === name,
        };
      })
      .sort((a, b) => b.tokens - a.tokens)
  );

  function formatProviderName(raw: string): string {
    if (raw.startsWith('openai-compatible-chat-')) {
      return `Custom (${raw.slice('openai-compatible-chat-'.length, 'openai-compatible-chat-'.length + 6)})`;
    }
    return raw;
  }
</script>

<div class="px-3.5 py-2 border-b border-white/[0.08]">
  <div class="flex items-center justify-between mb-1.5">
    <span class="text-[10px] uppercase tracking-wider text-slate-400 font-medium">
      Upstream Providers ({providerEntries.length})
    </span>
  </div>

  <div class="space-y-1.5 max-h-[140px] overflow-y-auto pr-0.5">
    {#if providerEntries.length === 0}
      <div class="py-3 text-center text-[11px] text-slate-500 font-mono">
        No active upstream providers in this period
      </div>
    {:else}
      {#each providerEntries as item (item.name)}
        <div
          class="flex items-center justify-between p-2 rounded glass-panel border transition-all duration-150 {item.isError ? 'border-rose-500/40 bg-rose-950/20' : 'border-white/[0.06]'}"
        >
          <!-- Left: Provider identity & volume -->
          <div class="min-w-0 pr-2">
            <div class="flex items-center space-x-1.5">
              <span
                class="w-1.5 h-1.5 rounded-full {item.isError ? 'bg-rose-500' : 'bg-emerald-400'}"
              ></span>
              <span
                class="text-[11px] font-medium text-slate-200 truncate capitalize"
                title={item.name}
              >
                {formatProviderName(item.name)}
              </span>
            </div>
            <div class="text-[10px] text-slate-400 font-mono pl-3 tnum">
              {formatCompact(item.tokens)} tok ({item.pct.toFixed(1)}%) · {item.stats.requests} req
            </div>
          </div>

          <!-- Right: Health or Cost Badge -->
          <div class="text-right flex-shrink-0">
            {#if item.isError}
              <span class="px-1.5 py-0.5 rounded text-[9.5px] font-mono font-medium bg-rose-500/20 text-rose-300 border border-rose-500/30">
                Rate Limited
              </span>
            {:else if item.stats.cost && item.stats.cost > 0}
              <div class="text-[11px] font-mono font-medium text-slate-200 tnum">
                {formatCurrency(item.stats.cost)}
              </div>
            {:else}
              <span class="px-1.5 py-0.5 rounded text-[9.5px] font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                Active
              </span>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
