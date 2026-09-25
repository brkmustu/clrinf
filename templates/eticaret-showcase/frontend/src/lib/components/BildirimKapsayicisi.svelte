<script lang="ts">
  import { bildirimler, bildirimSil } from '../hazneler/bildirimHaznesi';
  import HeroIcon from './HeroIcon.svelte';
</script>

<div class="fixed top-4 right-4 z-50 p-2 space-y-2.5 max-w-sm w-full pointer-events-none">
  {#each $bildirimler as b (b.id)}
    <div
      class="shadow-xl border text-xs pointer-events-auto flex items-start gap-3 p-3.5 rounded-xl transition-all duration-200 {b.tip === 'basari' ? 'bg-slate-900 border-emerald-500/40 text-slate-100' : b.tip === 'uyari' ? 'bg-slate-900 border-amber-500/40 text-slate-100' : b.tip === 'hata' ? 'bg-slate-900 border-rose-500/40 text-slate-100' : 'bg-slate-900 border-blue-500/40 text-slate-100'}"
    >
      <div class="shrink-0 mt-0.5">
        {#if b.tip === 'basari'}
          <HeroIcon name="check-circle" size={18} solid class="text-emerald-400" />
        {:else if b.tip === 'uyari'}
          <HeroIcon name="exclamation-triangle" size={18} class="text-amber-400" />
        {:else if b.tip === 'hata'}
          <HeroIcon name="x-mark" size={18} class="text-rose-400" />
        {:else}
          <HeroIcon name="information-circle" size={18} class="text-blue-400" />
        {/if}
      </div>

      <div class="flex-1 space-y-0.5 text-left">
        <div class="font-bold text-xs flex items-center justify-between">
          <span class="{b.tip === 'basari' ? 'text-emerald-400' : b.tip === 'uyari' ? 'text-amber-400' : b.tip === 'hata' ? 'text-rose-400' : 'text-blue-400'}">{b.baslik}</span>
          <span class="text-[10px] font-mono text-slate-400 ml-2">{b.zaman}</span>
        </div>
        <p class="text-[11px] text-slate-300 leading-relaxed font-normal">{b.mesaj}</p>
      </div>

      <button
        type="button"
        class="text-slate-400 hover:text-white p-0.5 rounded transition-colors cursor-pointer"
        onclick={() => bildirimSil(b.id)}
      >
        <HeroIcon name="x-mark" size={14} />
      </button>
    </div>
  {/each}
</div>
