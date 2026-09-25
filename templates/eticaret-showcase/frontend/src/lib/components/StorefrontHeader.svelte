<script lang="ts">
  import { onMount } from 'svelte';
  import { sepetKalemSayisi, sepetiAc } from '../hazneler/sepetHaznesi';
  import { aktifKullanici, tumKullanicilar, kullaniciDegistir, kullanicilariYukle } from '../hazneler/kimlikHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  onMount(() => {
    kullanicilariYukle();
  });

  interface Props {
    aramaMetni: string;
    seciliKategori: string;
    onKategoriSec: (kat: string) => void;
    onSiparislerimTikla: () => void;
    onMimariPanelTikla: () => void;
    toplamUrunSayisi: number;
  }

  let {
    aramaMetni = $bindable(''),
    seciliKategori,
    onKategoriSec,
    onSiparislerimTikla,
    onMimariPanelTikla,
    toplamUrunSayisi = 0
  }: Props = $props();

  let kullaniciMenuAcik = $state(false);

  function handleKullaniciSec(k: any) {
    kullaniciDegistir(k);
    kullaniciMenuAcik = false;
  }
</script>

<!-- Top Enterprise Procurement Bar -->
<div class="bg-slate-950 border-b border-slate-800/80 text-xs text-slate-400 py-1.5 px-4 sm:px-8">
  <div class="max-w-7xl mx-auto flex flex-wrap items-center justify-between gap-2">
    <div class="flex items-center gap-4">
      <span class="inline-flex items-center gap-1.5 text-slate-300">
        <HeroIcon name="shield-check" size={14} class="text-blue-400" />
        <span class="font-medium">ISO 9001:2015 & CE Endüstriyel Sertifikalı Tedarik Platformu</span>
      </span>
      <span class="hidden md:inline text-slate-700">|</span>
      <span class="hidden md:inline text-slate-400">Küresel B2B Sevkiyat & Hızlı Gümrükleme</span>
    </div>
    
    <div class="flex items-center gap-4 text-slate-300">
      <span class="inline-flex items-center gap-1">
        <HeroIcon name="truck" size={14} class="text-slate-400" />
        <span>$500+ Siparişlerde Ücretsiz Lojistik</span>
      </span>
      <span class="text-slate-700">|</span>
      <button 
        type="button" 
        onclick={onMimariPanelTikla}
        class="inline-flex items-center gap-1 text-blue-400 hover:text-blue-300 font-medium transition-colors cursor-pointer"
      >
        <HeroIcon name="cpu-chip" size={14} />
        <span>Sistem Mimarisi & ABAC Telemetri</span>
      </button>
    </div>
  </div>
</div>

<!-- Main Corporate Header -->
<header class="sticky top-0 z-40 bg-slate-900/95 backdrop-blur-md border-b border-slate-800 shadow-sm">
  <div class="max-w-7xl mx-auto px-4 sm:px-8 py-3.5 flex items-center justify-between gap-4">
    
    <!-- Corporate Brand Logo -->
    <div class="flex items-center gap-3 shrink-0">
      <div class="w-10 h-10 rounded-lg bg-blue-600 flex items-center justify-center text-white shadow-md shadow-blue-600/20">
        <HeroIcon name="cube" size={24} solid />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-xl font-bold tracking-tight text-white font-mono">CLRINF</span>
          <span class="px-2 py-0.5 text-[10px] font-semibold tracking-wider uppercase bg-blue-500/10 text-blue-400 border border-blue-500/30 rounded">
            Enterprise
          </span>
        </div>
        <p class="text-[11px] text-slate-400 font-medium">Endüstriyel Sensör & IoT Tedarik Merkezi</p>
      </div>
    </div>

    <!-- Corporate Search Input -->
    <div class="flex-1 max-w-xl hidden md:block">
      <div class="relative">
        <div class="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none text-slate-400">
          <HeroIcon name="magnifying-glass" size={18} />
        </div>
        <input
          type="text"
          bind:value={aramaMetni}
          placeholder="Ürün adı, SKU parça no veya kategori ara (örn: SKU-100, Optik)..."
          class="w-full bg-slate-950/80 border border-slate-700 text-slate-100 placeholder-slate-500 text-sm rounded-lg pl-10 pr-10 py-2 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all font-sans"
        />
        {#if aramaMetni}
          <button 
            type="button" 
            onclick={() => aramaMetni = ''}
            class="absolute inset-y-0 right-0 pr-3 flex items-center text-slate-400 hover:text-slate-200"
          >
            <HeroIcon name="x-mark" size={16} />
          </button>
        {/if}
      </div>
    </div>

    <!-- Actions & User Profile -->
    <div class="flex items-center gap-3">
      
      <!-- Order History Action -->
      <button
        type="button"
        onclick={onSiparislerimTikla}
        class="hidden sm:inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium text-slate-300 bg-slate-800/80 hover:bg-slate-800 border border-slate-700 rounded-lg transition-all cursor-pointer"
      >
        <HeroIcon name="document-text" size={16} class="text-slate-400" />
        <span>Siparişlerim</span>
      </button>

      <!-- Active User & Persona Switcher -->
      <div class="relative">
        <button
          type="button"
          onclick={() => kullaniciMenuAcik = !kullaniciMenuAcik}
          class="flex items-center gap-2.5 px-3 py-1.5 bg-slate-800/80 hover:bg-slate-800 border border-slate-700 rounded-lg transition-all text-left cursor-pointer"
        >
          <div class="w-7 h-7 rounded-full bg-slate-700 border border-slate-600 flex items-center justify-center text-blue-400 font-semibold text-xs">
            <HeroIcon name="user" size={14} />
          </div>
          <div class="hidden lg:block leading-tight">
            <div class="text-xs font-semibold text-slate-200">{$aktifKullanici.ad}</div>
            <div class="text-[10px] text-slate-400 flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 inline-block"></span>
              {$aktifKullanici.rol}
            </div>
          </div>
          <HeroIcon name="chevron-down" size={14} class="text-slate-400" />
        </button>

        {#if kullaniciMenuAcik}
          <div class="absolute right-0 mt-2 w-64 bg-slate-900 border border-slate-700 rounded-lg shadow-xl py-2 z-50 animate-in fade-in duration-100">
            <div class="px-3.5 py-2 border-b border-slate-800">
              <p class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Aktif B2B Oturumu</p>
              <p class="text-sm font-bold text-white mt-0.5">{$aktifKullanici.departman || 'Sistem & Operasyon'}</p>
              <p class="text-xs text-blue-400">Yetki: {$aktifKullanici.rol}</p>
            </div>

            <div class="py-1">
              <p class="px-3.5 py-1.5 text-[10px] font-semibold text-slate-500 uppercase tracking-wider">Kullanıcı / Rol Değiştir</p>
              {#each $tumKullanicilar as k}
                <button
                  type="button"
                  onclick={() => handleKullaniciSec(k)}
                  class="w-full text-left px-3.5 py-2 text-xs flex items-center justify-between hover:bg-slate-800 transition-colors {$aktifKullanici.id === k.id ? 'bg-blue-600/10 text-blue-400 font-medium' : 'text-slate-300'}"
                >
                  <div>
                    <div class="font-medium text-slate-200">{k.ad}</div>
                    <div class="text-[10px] text-slate-400">{k.departman} • {k.rol}</div>
                  </div>
                  {#if $aktifKullanici.id === k.id}
                    <HeroIcon name="check" size={14} class="text-blue-400" />
                  {/if}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Procurement Cart Trigger Button -->
      <button
        type="button"
        onclick={sepetiAc}
        class="relative inline-flex items-center gap-2 px-3.5 py-2 bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold rounded-lg shadow-sm shadow-blue-600/20 transition-all cursor-pointer"
        aria-label="Tedarik Sepeti"
      >
        <HeroIcon name="shopping-bag" size={18} />
        <span class="hidden sm:inline">Sepet</span>
        {#if $sepetKalemSayisi > 0}
          <span class="px-1.5 py-0.5 text-[10px] font-bold bg-white text-blue-700 rounded-full">
            {$sepetKalemSayisi}
          </span>
        {/if}
      </button>

    </div>
  </div>
</header>
