<script lang="ts">
  import { formatCompact } from '../utils/format';

  interface Props {
    cachedTokens: number;
    totalTokens: number;
    hostUrl: string;
    onQuit?: () => void;
  }

  let { cachedTokens, totalTokens, hostUrl, onQuit }: Props = $props();

  let copied = $state(false);

  const savingsPct = $derived(
    totalTokens + cachedTokens > 0
      ? (cachedTokens / (totalTokens + cachedTokens)) * 100
      : 0
  );

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(hostUrl);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 1500);
    } catch {
      // Fallback
    }
  }

  function handleOpenDashboard() {
    window.open(hostUrl, '_blank');
  }
</script>

<footer class="px-3.5 py-2.5 bg-white/[0.02]">
  <!-- RTK Savings Badge -->
  <div class="flex items-center justify-between py-1 mb-2 px-2 rounded bg-emerald-500/[0.06] border border-emerald-500/20 text-[10.5px] font-mono">
    <div class="flex items-center space-x-1.5 text-emerald-400">
      <span class="text-[9px]">✦</span>
      <span class="font-medium">RTK / Cache Saved:</span>
      <span class="font-semibold tnum">{formatCompact(cachedTokens)} tok</span>
    </div>
    <span class="text-emerald-300 font-semibold tnum">
      (-{savingsPct.toFixed(1)}%)
    </span>
  </div>

  <!-- Action Buttons -->
  <div class="grid grid-cols-3 gap-1.5 text-[11px] font-medium">
    <button
      type="button"
      class="py-1 rounded bg-white/[0.05] hover:bg-white/[0.1] text-slate-300 hover:text-white border border-white/[0.08] transition-colors text-center"
      onclick={handleOpenDashboard}
    >
      Dashboard
    </button>

    <button
      type="button"
      class="py-1 rounded bg-white/[0.05] hover:bg-white/[0.1] border border-white/[0.08] transition-colors text-center font-mono {copied ? 'text-emerald-400 font-semibold border-emerald-500/40' : 'text-slate-300 hover:text-white'}"
      onclick={handleCopy}
    >
      {#if copied}
        ✓ Copied!
      {:else}
        Copy URL
      {/if}
    </button>

    <button
      type="button"
      class="py-1 rounded bg-white/[0.05] hover:bg-rose-500/20 text-slate-400 hover:text-rose-300 border border-white/[0.08] hover:border-rose-500/30 transition-colors text-center"
      onclick={onQuit}
    >
      Quit
    </button>
  </div>
</footer>
