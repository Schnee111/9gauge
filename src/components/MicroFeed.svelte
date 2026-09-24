<script lang="ts">
  import type { RecentRequest } from '../types';
  import { formatCompact, formatTime } from '../utils/format';

  interface Props {
    requests: RecentRequest[];
  }

  let { requests }: Props = $props();

  // Show last 3-4 requests
  const visibleRequests = $derived((requests || []).slice(0, 4));

  function getStatusBadge(status: string) {
    if (!status || status.toLowerCase() === 'ok' || status === '200') {
      return { text: '200 OK', color: 'text-emerald-400' };
    }
    if (status.includes('429')) {
      return { text: '429 LIM', color: 'text-amber-400' };
    }
    return { text: status, color: 'text-rose-400' };
  }
</script>

<div class="px-3.5 py-2 border-b border-white/[0.08]">
  <div class="flex items-center justify-between mb-1.5">
    <span class="text-[10px] uppercase tracking-wider text-slate-400 font-medium">
      Activity Feed
    </span>
    <span class="text-[9.5px] font-mono text-slate-500">Live</span>
  </div>

  <div class="space-y-1">
    {#if visibleRequests.length === 0}
      <div class="py-2 text-center text-[10.5px] font-mono text-slate-500">
        Waiting for incoming requests...
      </div>
    {:else}
      {#each visibleRequests as req, idx (req.timestamp + idx)}
        {@const badge = getStatusBadge(req.status)}
        {@const total = (req.promptTokens || 0) + (req.completionTokens || 0)}
        <div class="flex items-center justify-between text-[10.5px] font-mono py-0.5">
          <div class="flex items-center space-x-1.5 min-w-0 pr-1">
            <span class="text-slate-500 tnum">{formatTime(req.timestamp)}</span>
            <span class="text-slate-600">·</span>
            <span class="text-slate-200 truncate max-w-[130px]" title={req.model}>
              {req.model}
            </span>
          </div>

          <div class="flex items-center space-x-2 flex-shrink-0">
            <span class="text-slate-400 tnum">{formatCompact(total)} tok</span>
            <span class="text-[9.5px] font-medium {badge.color}">{badge.text}</span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
