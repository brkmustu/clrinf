<script lang="ts">
  import type { KampanyaVerisi } from '../types';
  import { kuponKoduUygula } from '../hazneler/sepetHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  interface Props {
    kampanyalar: KampanyaVerisi[];
  }

  let { kampanyalar }: Props = $props();

  function kuponSec(k: KampanyaVerisi) {
    kuponKoduUygula(k.kod, typeof k.deger === 'number' && k.deger < 1 ? k.deger : 0.20);
  }
</script>

<section class="mb-10">
  <div class="flex items-center justify-between mb-4">
    <div>
      <div class="flex items-center gap-1.5 text-xs font-semibold text-blue-400">
        <HeroIcon name="bolt" size={14} />
        <span>Rust Kural Çözücü Motoru</span>
      </div>
      <h2 class="text-xl font-bold text-white tracking-tight mt-0.5">Aktif B2B Promosyonlar & Hacim Kuponları</h2>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    {#each kampanyalar as k}
      <div class="rounded-xl bg-slate-900 border border-slate-800 hover:border-slate-700 p-4.5 flex flex-col justify-between transition-colors text-left">
        <div>
          <div class="flex items-center justify-between gap-2 mb-2.5">
            <span class="px-2 py-0.5 text-xs font-bold rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 font-mono">
              {k.kod}
            </span>
            <span class="text-[11px] text-slate-400 font-mono">Min. ${k.minimumSepetTutari} USD</span>
          </div>

          <h3 class="text-xs font-bold text-white">
            {k.baslik}
          </h3>

          <p class="text-[11px] text-slate-400 mt-1.5 line-clamp-2">
            {k.aciklama || 'Endüstriyel parça alımlarında geçerli promosyon indirimi.'}
          </p>
        </div>

        <div class="mt-4 pt-3 border-t border-slate-800 flex items-center justify-between">
          <div class="text-[11px] text-slate-400 font-mono">
            Kullanım: <strong class="text-slate-200">{k.toplamKullanimSayisi}</strong> / {k.maksimumKullanimSayisi || '∞'}
          </div>

          <button 
            type="button"
            onclick={() => kuponSec(k)}
            class="inline-flex items-center gap-1 bg-slate-800 hover:bg-slate-700 text-blue-400 font-semibold text-xs px-3 py-1.5 rounded-lg border border-slate-700 transition-colors cursor-pointer"
          >
            <HeroIcon name="tag" size={13} />
            <span>Kuponu Tanımla</span>
          </button>
        </div>

      </div>
    {/each}
  </div>
</section>
