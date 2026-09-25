<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    sepet, 
    sepetAcik, 
    miktarGuncelle, 
    sepettenCikar, 
    kuponKoduUygula, 
    kuponuKaldir, 
    odemeModalAcik, 
    motorHesaplamaSuresi, 
    aktifKampanyaListesi, 
    yukleAktifKampanyalar 
  } from '../hazneler/sepetHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import { stoklar } from '../hazneler/stokHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  let kuponGirdisi = $state('');
  let kuponKontrolEdiliyor = $state(false);

  const ucretsizKargoLimiti = 500;
  const kargoKalanTutar = $derived(Math.max(0, ucretsizKargoLimiti - $sepet.araToplam));
  const kargoYuzdesi = $derived(Math.min(100, ($sepet.araToplam / ucretsizKargoLimiti) * 100));

  onMount(() => {
    yukleAktifKampanyalar();
  });

  async function kuponSecTikla(kod: string) {
    if ($sepet.kuponKodu === kod) {
      await kuponuKaldir();
    } else {
      await kuponKoduUygula(kod);
    }
  }

  async function manuelKuponUygula() {
    const kod = kuponGirdisi.trim().toUpperCase();
    if (!kod) return;

    kuponKontrolEdiliyor = true;
    await kuponKoduUygula(kod);
    kuponGirdisi = '';
    kuponKontrolEdiliyor = false;
  }

  function odemeyeGec() {
    if ($sepet.kalemler.length === 0) {
      bildirimEkle({ tur: 'uyari', baslik: 'Sepet Boş', mesaj: 'Lütfen önce sepete parça ekleyin.', sure: 2500 });
      return;
    }
    $sepetAcik = false;
    $odemeModalAcik = true;
  }
</script>

{#if $sepetAcik}
  <!-- Backdrop -->
  <div 
    onclick={() => $sepetAcik = false}
    onkeydown={(e) => { if (e.key === 'Escape') $sepetAcik = false; }}
    role="button"
    tabindex="0"
    class="fixed inset-0 bg-slate-950/80 backdrop-blur-xs z-50 transition-opacity"
  >
  </div>

  <!-- Slide-Over Procurement Drawer -->
  <div class="fixed inset-y-0 right-0 max-w-full flex pl-10 z-50">
    <div class="w-screen max-w-md bg-slate-900 border-l border-slate-800 shadow-2xl flex flex-col justify-between">
      
      <!-- Drawer Header -->
      <div class="p-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-blue-600/20 border border-blue-500/30 flex items-center justify-center text-blue-400">
            <HeroIcon name="shopping-bag" size={20} />
          </div>
          <div>
            <h2 class="text-sm font-bold text-white">Tedarik Sepeti</h2>
            <p class="text-[11px] text-slate-400">{$sepet.kalemler.length} Parça Kalemi • Rust Kampanya Motoru Aktif</p>
          </div>
        </div>

        <button 
          type="button"
          onclick={() => $sepetAcik = false}
          class="p-1.5 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white transition-colors cursor-pointer"
        >
          <HeroIcon name="x-mark" size={18} />
        </button>
      </div>

      <!-- Shipping / Promotion Progress Ribbon -->
      <div class="px-4 py-2.5 bg-slate-950 border-b border-slate-800 text-xs">
        {#if $sepet.araToplam >= 500}
          <div class="flex items-center justify-between gap-2 p-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
            <div class="flex items-center gap-1.5 text-xs font-semibold">
              <HeroIcon name="check-circle" size={16} solid />
              <span>Ücretsiz Lojistik & %20 Flaş İndirim Koşulu Sağlandı</span>
            </div>
            {#if !$sepet.kuponKodu}
              <button 
                type="button"
                onclick={() => kuponSecTikla('INDIRIM20')}
                class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-[10px] rounded transition-colors cursor-pointer"
              >
                Tanımla
              </button>
            {/if}
          </div>
        {:else}
          <div class="space-y-1.5">
            <div class="flex justify-between text-slate-300 text-[11px]">
              <span>Ücretsiz Lojistik için Kalan Tutar:</span>
              <span class="text-blue-400 font-mono font-bold">${kargoKalanTutar.toLocaleString('tr-TR')} USD</span>
            </div>
            <div class="w-full h-1.5 bg-slate-800 rounded-full overflow-hidden">
              <div class="h-full bg-blue-500 transition-all duration-300" style="width: {kargoYuzdesi}%"></div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Cart Item Rows -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3 divide-y divide-slate-800">
        {#if $sepet.kalemler.length === 0}
          <div class="text-center py-16 space-y-3">
            <div class="w-12 h-12 rounded-full bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
              <HeroIcon name="shopping-cart" size={24} />
            </div>
            <h3 class="text-xs font-bold text-white">Sepetinizde Henüz Parça Bulunmuyor</h3>
            <p class="text-[11px] text-slate-400 max-w-xs mx-auto">Katalogdan endüstriyel sensör veya donanım kalemlerini seçerek başlayın.</p>
            <button 
              type="button"
              onclick={() => $sepetAcik = false}
              class="px-4 py-2 bg-blue-600 text-white font-semibold text-xs rounded-lg hover:bg-blue-500 transition-colors cursor-pointer"
            >
              Kataloğa Git
            </button>
          </div>
        {:else}
          {#each $sepet.kalemler as item}
            <div class="pt-3 first:pt-0 flex gap-3 items-center">
              
              <!-- Hardware Icon -->
              <div class="w-12 h-12 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-center text-slate-400 shrink-0">
                <HeroIcon name="cpu-chip" size={22} />
              </div>

              <!-- Item Info -->
              <div class="flex-1 min-w-0 text-left">
                <div class="text-[10px] text-blue-400 font-mono">{item.sku}</div>
                <h4 class="text-xs font-semibold text-white truncate">{item.baslik}</h4>
                <div class="text-xs font-bold text-slate-200 mt-0.5 font-mono">
                  ${item.birimFiyat.toLocaleString('tr-TR')} <span class="text-[10px] font-normal text-slate-400">/ adet</span>
                </div>

                <!-- Quantity Modifier -->
                <div class="flex items-center gap-2 mt-1.5">
                  <div class="flex items-center bg-slate-950 border border-slate-800 rounded p-0.5">
                    <button 
                      type="button"
                      onclick={() => miktarGuncelle(item.sku, item.miktar - 1)}
                      class="w-5 h-5 flex items-center justify-center text-xs text-slate-400 hover:text-white rounded cursor-pointer"
                    >
                      -
                    </button>
                    <span class="w-6 text-center text-xs font-mono font-bold text-white">{item.miktar}</span>
                    <button 
                      type="button"
                      disabled={item.miktar >= ($stoklar[item.sku]?.musaitStok ?? 999)}
                      onclick={() => miktarGuncelle(item.sku, item.miktar + 1)}
                      title={item.miktar >= ($stoklar[item.sku]?.musaitStok ?? 999) ? 'Maksimum depodaki stok miktarına ulaşıldı' : 'Adet Arttır'}
                      class="w-5 h-5 flex items-center justify-center text-xs rounded cursor-pointer {item.miktar >= ($stoklar[item.sku]?.musaitStok ?? 999) ? 'text-slate-600 cursor-not-allowed' : 'text-slate-400 hover:text-white'}"
                    >
                      +
                    </button>
                  </div>

                  <button 
                    type="button"
                    onclick={() => sepettenCikar(item.sku)}
                    class="text-[10px] text-rose-400 hover:text-rose-300 transition-colors ml-1 cursor-pointer"
                  >
                    Kaldır
                  </button>
                </div>
              </div>

              <!-- Line Total -->
              <div class="text-right">
                <div class="text-xs font-bold text-white font-mono">
                  ${item.toplamTutar.toLocaleString('tr-TR')}
                </div>
              </div>

            </div>
          {/each}

          <!-- Dynamic Promotional Rules (Live Rust Constraint Solver) -->
          <div class="pt-3 space-y-2">
            <div class="flex items-center justify-between text-[10px]">
              <span class="text-slate-400 font-semibold uppercase tracking-wider">Tanımlı Kampanyalar:</span>
              <span class="text-blue-400 font-mono font-bold">Rust Solver: {$motorHesaplamaSuresi} µs</span>
            </div>

            <div class="grid grid-cols-2 gap-1.5">
              {#each $aktifKampanyaListesi as k}
                {@const uygun = $sepet.araToplam >= (k.minimumSepetTutari || 0)}
                {@const secili = $sepet.kuponKodu === k.kod}
                <button 
                  type="button"
                  onclick={() => uygun && kuponSecTikla(k.kod)}
                  disabled={!uygun}
                  class="p-2 rounded-lg text-left border transition-all cursor-pointer {secili ? 'bg-blue-600/15 border-blue-500 text-blue-300' : uygun ? 'bg-slate-950 hover:bg-slate-800 border-slate-700 text-slate-300' : 'bg-slate-950/40 border-slate-800/60 text-slate-600 cursor-not-allowed'}"
                >
                  <div class="flex items-center justify-between">
                    <span class="font-mono font-bold text-[11px] {secili ? 'text-blue-400' : 'text-slate-300'}">{k.kod}</span>
                    <span class="text-[9px] px-1 py-0.2 rounded font-medium {secili ? 'bg-blue-600/30 text-blue-300' : 'bg-slate-800 text-slate-400'}">
                      {secili ? 'Aktif' : `Min $${k.minimumSepetTutari}`}
                    </span>
                  </div>
                  <div class="text-[10px] font-medium mt-0.5 truncate">{k.baslik}</div>
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Drawer Footer & Quotation Totals -->
      {#if $sepet.kalemler.length > 0}
        <div class="p-4 bg-slate-950 border-t border-slate-800 space-y-3">
          
          <!-- Manual Coupon Box -->
          {#if !$sepet.kuponKodu}
            <div class="flex gap-2">
              <input 
                type="text" 
                placeholder="Promosyon / Kupon Kodu (örn: INDIRIM20)"
                bind:value={kuponGirdisi}
                class="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white uppercase placeholder-slate-500 focus:outline-none focus:border-blue-500 font-mono"
              />
              <button 
                type="button"
                onclick={manuelKuponUygula}
                disabled={kuponKontrolEdiliyor || !kuponGirdisi.trim()}
                class="bg-slate-800 hover:bg-slate-700 border border-slate-700 disabled:opacity-50 text-white font-semibold text-xs px-3 py-1.5 rounded-lg transition-colors cursor-pointer"
              >
                {kuponKontrolEdiliyor ? '...' : 'Uygula'}
              </button>
            </div>
          {:else}
            <div class="flex items-center justify-between p-2 rounded-lg bg-blue-500/10 border border-blue-500/20 text-xs">
              <div class="flex items-center gap-1.5 text-blue-400 font-semibold">
                <HeroIcon name="tag" size={14} />
                <span>{$sepet.kuponKodu} (-${$sepet.indirimTutari.toLocaleString('tr-TR')} İndirim)</span>
              </div>
              <button 
                type="button"
                onclick={kuponuKaldir}
                class="text-slate-400 hover:text-rose-400 font-medium text-xs cursor-pointer"
              >
                Kaldır
              </button>
            </div>
          {/if}

          <!-- Cost Breakdown -->
          <div class="space-y-1.5 text-xs">
            <div class="flex justify-between text-slate-400">
              <span>Ara Toplam (Net)</span>
              <span class="text-slate-200 font-mono font-medium">${$sepet.araToplam.toLocaleString('tr-TR')}</span>
            </div>

            {#if $sepet.indirimTutari > 0}
              <div class="flex justify-between text-emerald-400 font-medium">
                <span>Kampanya İndirimi ({$sepet.kuponKodu})</span>
                <span class="font-mono">-${$sepet.indirimTutari.toLocaleString('tr-TR')}</span>
              </div>
            {/if}

            <div class="flex justify-between text-slate-400">
              <span>KDV (%18)</span>
              <span class="text-slate-200 font-mono font-medium">${$sepet.kdvTutari.toLocaleString('tr-TR')}</span>
            </div>

            <div class="flex justify-between text-slate-400">
              <span>Lojistik & Taşıma</span>
              <span class="{$sepet.kargoUcreti === 0 ? 'text-emerald-400 font-semibold' : 'text-slate-200 font-mono'}">
                {$sepet.kargoUcreti === 0 ? 'ÜCRETSİZ' : `$${$sepet.kargoUcreti}`}
              </span>
            </div>

            <div class="pt-2 border-t border-slate-800 flex justify-between text-sm font-bold text-white">
              <span>Toplam Tutar (KDV Dahil)</span>
              <span class="text-blue-400 font-mono">${$sepet.genelToplam.toLocaleString('tr-TR')} USD</span>
            </div>
          </div>

          <!-- Direct Procurement Checkout Action -->
          <button 
            type="button"
            onclick={odemeyeGec}
            class="w-full bg-blue-600 hover:bg-blue-500 text-white font-semibold py-2.5 rounded-lg shadow-sm shadow-blue-600/20 transition-colors flex items-center justify-center gap-2 cursor-pointer text-xs"
          >
            <HeroIcon name="credit-card" size={16} />
            <span>Siparişi Onayla & Ödemeye Geç</span>
          </button>

        </div>
      {/if}

    </div>
  </div>
{/if}
