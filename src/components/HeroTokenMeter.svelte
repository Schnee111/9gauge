<script lang="ts">
  import type { UsageSnapshot } from '../types';
  import {
    formatNumber,
    formatCompact,
    formatCurrency,
    calculateVelocity,
  } from '../utils/format';

  interface Props {
    snapshot: UsageSnapshot;
    periodLabel: string;
  }

  let { snapshot, periodLabel }: Props = $props();

  const totalTokens = $derived(
    snapshot.totalPromptTokens + snapshot.totalCompletionTokens
  );

  const promptPct = $derived(
    totalTokens > 0 ? (snapshot.totalPromptTokens / totalTokens) * 100 : 0
  );
  const completionPct = $derived(
    totalTokens > 0 ? (snapshot.totalCompletionTokens / totalTokens) * 100 : 0
  );
  const cachePct = $derived(
    totalTokens > 0
      ? (snapshot.totalCachedTokens / (totalTokens + snapshot.totalCachedTokens)) * 100
      : 0
  );

  const velocity = $derived(calculateVelocity(snapshot.last10Minutes || []));
</script>

<div class="px-3.5 py-3 border-b border-white/[0.08] bg-white/[0.01]">
  <!-- Headline & Cost -->
  <div class="flex items-baseline justify-between mb-1.5">
    <div>
      <div class="text-[20px] font-semibold text-white font-mono tracking-tight leading-none tnum">
        {formatNumber(totalTokens)}
      </div>
      <div class="text-[10px] uppercase tracking-wider text-slate-400 font-medium mt-0.5">
        Tokens Consumed {periodLabel}
      </div>
    </div>
    <div class="text-right">
      <div class="text-[13px] font-semibold text-slate-200 font-mono tnum">
        {formatCurrency(snapshot.totalCost)}
      </div>
      <div class="text-[9px] uppercase tracking-wider text-slate-500 font-medium">
        Estimated Cost
      </div>
    </div>
  </div>

  <!-- Segmented Progress Bar -->
  <div class="w-full h-1.5 rounded-full overflow-hidden flex bg-white/[0.06] my-2.5 shadow-inner">
    {#if totalTokens > 0}
      <div
        class="h-full bg-blue-400 transition-all duration-300"
        style="width: {promptPct}%;"
        title="Prompt: {formatCompact(snapshot.totalPromptTokens)} ({promptPct.toFixed(1)}%)"
      ></div>
      <div
        class="h-full bg-purple-400 transition-all duration-300"
        style="width: {completionPct}%;"
        title="Completion: {formatCompact(snapshot.totalCompletionTokens)} ({completionPct.toFixed(1)}%)"
      ></div>
    {:else}
      <div class="w-full h-full bg-white/[0.05]"></div>
    {/if}
  </div>

  <!-- Metrics Breakdown Grid -->
  <div class="grid grid-cols-3 gap-1.5 py-1 text-[11px] font-mono">
    <!-- Prompt -->
    <div class="flex items-center space-x-1.5">
      <span class="w-1.5 h-1.5 rounded-full bg-blue-400"></span>
      <span class="text-slate-400">In:</span>
      <span class="font-medium text-slate-200 tnum">{formatCompact(snapshot.totalPromptTokens)}</span>
    </div>

    <!-- Completion -->
    <div class="flex items-center space-x-1.5">
      <span class="w-1.5 h-1.5 rounded-full bg-purple-400"></span>
      <span class="text-slate-400">Out:</span>
      <span class="font-medium text-slate-200 tnum">{formatCompact(snapshot.totalCompletionTokens)}</span>
    </div>

    <!-- Cached -->
    <div class="flex items-center space-x-1.5 justify-end">
      <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
      <span class="text-slate-400">Hit:</span>
      <span class="font-medium text-emerald-400 tnum">
        {formatCompact(snapshot.totalCachedTokens)}
        <span class="text-[9.5px] text-slate-500">({cachePct.toFixed(0)}%)</span>
      </span>
    </div>
  </div>

  <!-- Velocity & Throughput Footer -->
  <div class="flex items-center justify-between text-[10px] font-mono text-slate-400 pt-1.5 mt-1 border-t border-white/[0.04]">
    <div class="flex items-center space-x-1">
      <span class="text-slate-500">Rate:</span>
      <span class="text-slate-300 font-medium tnum">{formatCompact(velocity.tokPerHour)} tok/h</span>
    </div>
    <div class="flex items-center space-x-1">
      <span class="text-slate-500">Throughput:</span>
      <span class="text-slate-300 font-medium tnum">{velocity.reqPerMin.toFixed(1)} req/m</span>
    </div>
  </div>
</div>
