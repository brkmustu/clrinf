<script lang="ts">
  import type { KategoriItem } from '../types';
  import { aktifSayfa } from '../hazneler/sepetHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  interface Props {
    seciliKategori: string;
    kategoriler?: KategoriItem[];
    onKategoriSec: (kategori: string) => void;
  }

  let { seciliKategori, kategoriler = [], onKategoriSec }: Props = $props();

  function kategoriTikla(ad: string) {
    if (seciliKategori === ad) {
      onKategoriSec('');
    } else {
      onKategoriSec(ad);
    }
    $aktifSayfa = 'katalog';
  }

  function getKategoriIcon(ad: string): string {
    const lower = ad.toLowerCase();
    if (lower.includes('optik') || lower.includes('lazer')) return 'eye';
    if (lower.includes('iot') || lower.includes('gateway')) return 'cpu-chip';
    if (lower.includes('sıcaklık') || lower.includes('sicaklik')) return 'bolt';
    if (lower.includes('basınç') || lower.includes('basinc')) return 'wrench-screwdriver';
    if (lower.includes('robot') || lower.includes('otomasyon')) return 'cog-6-tooth';
    return 'cube';
  }
</script>

{#if kategoriler.length > 0}
  <section class="mb-10">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h2 class="text-xl font-bold text-white tracking-tight">Ürün Grupları & Kategoriler</h2>
        <p class="text-xs text-slate-400">Endüstriyel tesisler ve laboratuvarlar için sertifikalı parça kategorileri</p>
      </div>
      <button 
        type="button"
        onclick={() => { onKategoriSec(''); $aktifSayfa = 'katalog'; }}
        class="text-xs font-semibold text-blue-400 hover:text-blue-300 transition-colors inline-flex items-center gap-1 cursor-pointer"
      >
        <span>Tümünü Göster ({kategoriler.length})</span>
        <HeroIcon name="chevron-right" size={14} />
      </button>
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3.5">
      {#each kategoriler as kat}
        <button 
          type="button"
          onclick={() => kategoriTikla(kat.ad)}
          class="group p-4 rounded-xl bg-slate-900 border {seciliKategori === kat.ad ? 'border-blue-500 bg-slate-800/80 shadow-md ring-1 ring-blue-500/20' : 'border-slate-800 hover:border-slate-700 hover:bg-slate-850'} transition-all text-left cursor-pointer"
        >
          <div class="w-10 h-10 rounded-lg bg-slate-800 group-hover:bg-blue-600/20 text-slate-300 group-hover:text-blue-400 flex items-center justify-center mb-3 transition-colors">
            <HeroIcon name={getKategoriIcon(kat.ad)} size={20} />
          </div>

          <h3 class="text-xs font-bold text-white group-hover:text-blue-400 transition-colors truncate">{kat.ad}</h3>
          <p class="text-[11px] text-slate-400 mt-1 line-clamp-2">{kat.aciklama || 'Endüstriyel parça grubu.'}</p>

          <div class="mt-3 flex items-center justify-between pt-2 border-t border-slate-800 text-[11px]">
            <span class="text-slate-400 font-medium">Katalog</span>
            <span class="text-blue-400 font-semibold group-hover:translate-x-0.5 transition-transform">→</span>
          </div>
        </button>
      {/each}
    </div>
  </section>
{/if}
