<script lang="ts">
  import type { UrunKalemi } from '../types';
  import { sepet, sepeteEkle, favoriler, favoriDegistir, seciliUrunDetay } from '../hazneler/sepetHaznesi';
  import { stoklar } from '../hazneler/stokHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  interface Props {
    urun: UrunKalemi;
  }

  let { urun }: Props = $props();

  let ekleniyor = $state(false);

  const favorideMi = $derived($favoriler.includes(urun.sku));
  
  // Canlı Stok Haznesi Dinleyicisi
  const anlikMusaitStok = $derived($stoklar[urun.sku]?.musaitStok ?? urun.musaitStok ?? 50);
  const sepetKalemi = $derived($sepet.kalemler.find(k => k.sku === urun.sku));
  const sepettekiMiktar = $derived(sepetKalemi ? sepetKalemi.miktar : 0);
  const stokDoluMu = $derived(anlikMusaitStok > 0 && sepettekiMiktar >= anlikMusaitStok);
  const tukendiMi = $derived(anlikMusaitStok <= 0);

  async function sepeteEkleTikla(e: MouseEvent) {
    e.stopPropagation();
    if (tukendiMi || stokDoluMu) return;

    ekleniyor = true;
    await sepeteEkle(urun, 1);
    setTimeout(() => {
      ekleniyor = false;
    }, 400);
  }

  function favoriTikla(e: MouseEvent) {
    e.stopPropagation();
    favoriDegistir(urun.sku);
  }

  function detayAc() {
    $seciliUrunDetay = urun;
  }
</script>

<div 
  onclick={detayAc}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); detayAc(); } }}
  role="button"
  tabindex="0"
  class="group flex flex-col justify-between rounded-xl bg-slate-900 border border-slate-800 hover:border-slate-700 p-4.5 transition-all duration-200 hover:shadow-lg cursor-pointer relative overflow-hidden text-left"
>
  
  <!-- Badges & Wishlist -->
  <div class="flex items-center justify-between gap-2 mb-3">
    <div class="flex flex-wrap items-center gap-1.5">
      {#if tukendiMi}
        <span class="px-2 py-0.5 text-[10px] font-semibold rounded bg-rose-500/10 text-rose-400 border border-rose-500/20 font-mono">
          Tükendi
        </span>
      {:else if anlikMusaitStok <= 5}
        <span class="px-2 py-0.5 text-[10px] font-semibold rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 font-mono">
          Kritik Stok: {anlikMusaitStok}
        </span>
      {:else}
        <span class="px-2 py-0.5 text-[10px] font-semibold rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-mono">
          Mevcut ({anlikMusaitStok})
        </span>
      {/if}

      {#if urun.etiket === 'CokSatan'}
        <span class="px-2 py-0.5 text-[10px] font-semibold rounded bg-blue-500/10 text-blue-400 border border-blue-500/20">
          Popüler Parça
        </span>
      {/if}
    </div>

    <!-- Favorite Trigger -->
    <button 
      type="button"
      onclick={favoriTikla}
      title={favorideMi ? 'Favorilerden Çıkar' : 'Favorilere Ekle'}
      class="p-1.5 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-rose-400 transition-colors"
    >
      <HeroIcon name="heart" size={14} solid={favorideMi} class={favorideMi ? 'text-rose-500' : 'text-slate-400'} />
    </button>
  </div>

  <!-- Hardware Mockup / CAD Preview Area -->
  <div class="w-full h-36 rounded-lg bg-slate-950 border border-slate-800/80 flex items-center justify-center relative overflow-hidden group-hover:border-slate-700 transition-all p-3">
    
    <div class="w-16 h-16 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center text-slate-400 group-hover:text-blue-400 group-hover:border-blue-500/30 transition-colors">
      <HeroIcon name="cpu-chip" size={32} />
    </div>

    <!-- Quick View Overlay -->
    <div class="absolute inset-0 bg-slate-950/70 backdrop-blur-xs flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
      <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-900 text-slate-200 font-semibold text-xs rounded-md border border-slate-700 shadow">
        <HeroIcon name="eye" size={14} />
        <span>Spesifikasyon İncele</span>
      </span>
    </div>
  </div>

  <!-- Specs & Details -->
  <div class="mt-3.5 space-y-1.5 flex-1">
    <div class="flex items-center justify-between text-[11px]">
      <span class="text-blue-400 font-medium">{urun.kategoriYolu}</span>
      <span class="text-slate-400 font-mono text-[10px]">{urun.sku}</span>
    </div>

    <h3 class="text-xs font-bold text-white group-hover:text-blue-400 transition-colors line-clamp-2">
      {urun.baslik}
    </h3>

    <p class="text-[11px] text-slate-400 line-clamp-2">
      {urun.aciklama || 'Endüstriyel sınıf onaylı bileşen.'}
    </p>

    <!-- Rating & Stock Metric -->
    <div class="flex items-center justify-between pt-1">
      <div class="flex items-center gap-1 text-xs">
        <HeroIcon name="star" size={12} solid class="text-amber-400" />
        <span class="font-semibold text-slate-200">{urun.puan || 5.0}</span>
        <span class="text-slate-400 text-[10px]">({urun.degerlendirmeSayisi || 12})</span>
      </div>

      <span class="text-[10px] text-slate-400 font-mono">
        B2B Fiyatlandırma
      </span>
    </div>
  </div>

  <!-- Price & Add To Cart Button -->
  <div class="mt-3.5 pt-3 border-t border-slate-800 flex items-center justify-between gap-2">
    <div>
      {#if urun.eskiFiyat && urun.eskiFiyat > urun.birimFiyat}
        <div class="text-[10px] text-slate-400 line-through font-mono">
          ${urun.eskiFiyat.toLocaleString('tr-TR')}
        </div>
      {/if}
      <div class="text-base font-bold text-white font-mono">
        ${urun.birimFiyat.toLocaleString('tr-TR')} <span class="text-[10px] font-normal text-slate-400">{urun.paraBirimi}</span>
      </div>
    </div>

    <button 
      type="button"
      onclick={sepeteEkleTikla}
      disabled={tukendiMi || stokDoluMu || ekleniyor}
      class="inline-flex items-center gap-1 px-3 py-2 rounded-lg font-semibold text-xs transition-colors cursor-pointer {tukendiMi ? 'bg-slate-800 text-slate-500 cursor-not-allowed' : stokDoluMu ? 'bg-slate-800 border border-slate-700 text-slate-400 cursor-not-allowed' : ekleniyor ? 'bg-emerald-600 text-white' : sepettekiMiktar > 0 ? 'bg-blue-700 hover:bg-blue-600 text-white shadow-sm' : 'bg-blue-600 hover:bg-blue-500 text-white'}"
    >
      {#if ekleniyor}
        <HeroIcon name="check" size={14} />
        <span>Eklendi</span>
      {:else if tukendiMi}
        <span>Tükendi</span>
      {:else if stokDoluMu}
        <HeroIcon name="shield-check" size={14} class="text-amber-400" />
        <span>Maks. ({anlikMusaitStok})</span>
      {:else if sepettekiMiktar > 0}
        <HeroIcon name="plus" size={14} />
        <span>Sepette ({sepettekiMiktar}) +1</span>
      {:else}
        <HeroIcon name="plus" size={14} />
        <span>Sepete Ekle</span>
      {/if}
    </button>
  </div>

</div>
