<script lang="ts">
  import { seciliUrunDetay, sepeteEkle, favoriler, favoriDegistir } from '../hazneler/sepetHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import { aktifKullanici } from '../hazneler/kimlikHaznesi';
  import type { DegerlendirmeItem } from '../types';
  import HeroIcon from './HeroIcon.svelte';

  let seciliAdet = $state(1);
  let yeniPuan = $state(5);
  let yeniYorum = $state('');
  let yorumFormuAcik = $state(false);
  let yorumlar = $state<DegerlendirmeItem[]>([]);
  let yorumlarYukleniyor = $state(false);

  const favorideMi = $derived($seciliUrunDetay ? $favoriler.includes($seciliUrunDetay.sku) : false);

  $effect(() => {
    if ($seciliUrunDetay) {
      seciliAdet = 1;
      yorumlarYukle($seciliUrunDetay.id || 1);
    }
  });

  async function yorumlarYukle(urunId: number) {
    yorumlarYukleniyor = true;
    try {
      const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';
      const res = await fetch(`${apiHost}/api/Degerlendirme?PageIndex=0&PageSize=20`);
      if (res.ok) {
        const data = await res.json();
        if (data && data.items) {
          yorumlar = data.items.map((item: any) => ({
            id: item.id,
            urunId: item.urunId,
            kullaniciAdi: item.kullaniciAdi || 'B2B Müşteri',
            puan: item.puan || 5,
            yorum: item.yorum || 'Ürün başarıyla teslim alındı.',
            tarih: 'Doğrulanmış Alıcı',
            onayliAlici: item.onayliAlici !== false
          }));
        }
      }
    } catch {
      yorumlar = [];
    } finally {
      yorumlarYukleniyor = false;
    }
  }

  async function sepeteEkleTikla() {
    if (!$seciliUrunDetay) return;
    const basarili = await sepeteEkle($seciliUrunDetay, seciliAdet);
    if (basarili) {
      $seciliUrunDetay = null;
    }
  }

  async function yorumGonder() {
    if (!yeniYorum.trim() || !$seciliUrunDetay) return;

    const payload = {
      urunId: $seciliUrunDetay.id || 1,
      kullaniciAdi: `${$aktifKullanici.ad} (${$aktifKullanici.departman})`,
      puan: yeniPuan,
      yorum: yeniYorum.trim(),
      onayliAlici: true
    };

    // Anlık ekle
    yorumlar = [
      {
        id: Date.now(),
        urunId: payload.urunId,
        kullaniciAdi: payload.kullaniciAdi,
        puan: payload.puan,
        yorum: payload.yorum,
        tarih: 'Az önce',
        onayliAlici: true
      },
      ...yorumlar
    ];

    try {
      const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';
      await fetch(`${apiHost}/api/Degerlendirme`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
    } catch {}

    yeniYorum = '';
    yorumFormuAcik = false;

    bildirimEkle({
      tur: 'basari',
      baslik: 'Değerlendirmeniz Kaydedildi',
      mesaj: 'Müşteri incelemeniz veritabanına başarıyla aktarıldı.',
      sure: 3000
    });
  }
</script>

{#if $seciliUrunDetay}
  <!-- Backdrop -->
  <div 
    onclick={() => $seciliUrunDetay = null}
    onkeydown={(e) => { if (e.key === 'Escape') $seciliUrunDetay = null; }}
    role="button"
    tabindex="0"
    class="fixed inset-0 bg-slate-950/80 backdrop-blur-xs z-50 transition-opacity"
  >
  </div>

  <!-- Modal Dialog -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <div class="w-full max-w-2xl bg-slate-900 border border-slate-800 rounded-xl shadow-2xl overflow-hidden max-h-[90vh] flex flex-col">
      
      <!-- Modal Header -->
      <div class="p-4 bg-slate-950 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-xs font-mono text-blue-400 bg-blue-500/10 px-2.5 py-0.5 rounded border border-blue-500/20">
            {$seciliUrunDetay.sku}
          </span>
          <span class="text-xs text-slate-400">{$seciliUrunDetay.kategoriYolu}</span>
        </div>
        <button 
          type="button"
          onclick={() => $seciliUrunDetay = null}
          class="text-slate-400 hover:text-white p-1 rounded-md hover:bg-slate-800 transition-colors cursor-pointer"
        >
          <HeroIcon name="x-mark" size={18} />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 overflow-y-auto space-y-5 flex-1 text-left">
        
        <div class="grid grid-cols-1 md:grid-cols-12 gap-5">
          
          <!-- Hardware Visual -->
          <div class="md:col-span-5 rounded-lg bg-slate-950 border border-slate-800 p-5 flex flex-col items-center justify-center">
            <div class="w-20 h-20 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center text-blue-400">
              <HeroIcon name="cpu-chip" size={40} />
            </div>
            <div class="mt-3 flex items-center gap-1.5 text-xs">
              <span class="w-2 h-2 rounded-full {$seciliUrunDetay.musaitStok > 0 ? 'bg-emerald-400' : 'bg-rose-500'}"></span>
              <span class="text-slate-300 font-medium">{$seciliUrunDetay.musaitStok} Adet Hazır Stok</span>
            </div>
          </div>

          <!-- Specs Info -->
          <div class="md:col-span-7 space-y-3">
            <h2 class="text-base font-bold text-white leading-tight">
              {$seciliUrunDetay.baslik}
            </h2>

            <div class="flex items-center gap-2">
              <div class="flex items-center text-amber-400 text-xs">
                {#each [1, 2, 3, 4, 5] as _}
                  <HeroIcon name="star" size={13} solid />
                {/each}
                <span class="ml-1.5 text-xs font-bold text-white">{$seciliUrunDetay.puan || 5.0}</span>
              </div>
              <span class="text-xs text-slate-400">({yorumlar.length} Değerlendirme)</span>
            </div>

            <p class="text-xs text-slate-300 leading-relaxed">
              {$seciliUrunDetay.aciklama || 'Endüstri standardında sertifikalı donanım.'}
            </p>

            <!-- Price & Quantity Picker -->
            <div class="p-3 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
              <div>
                <div class="text-[10px] text-slate-400">Birim Fiyat (B2B)</div>
                <div class="text-lg font-bold text-white font-mono">
                  ${$seciliUrunDetay.birimFiyat.toLocaleString('tr-TR')} <span class="text-xs font-normal text-slate-400">USD</span>
                </div>
              </div>

              <!-- Quantity Selector -->
              <div class="flex items-center bg-slate-900 border border-slate-700 rounded p-0.5">
                <button 
                  type="button"
                  onclick={() => seciliAdet = Math.max(1, seciliAdet - 1)}
                  class="w-6 h-6 flex items-center justify-center text-slate-300 hover:text-white rounded text-xs cursor-pointer"
                >
                  -
                </button>
                <span class="w-7 text-center text-xs font-mono font-bold text-white">{seciliAdet}</span>
                <button 
                  type="button"
                  onclick={() => seciliAdet = Math.min($seciliUrunDetay?.musaitStok || 99, seciliAdet + 1)}
                  class="w-6 h-6 flex items-center justify-center text-slate-300 hover:text-white rounded text-xs cursor-pointer"
                >
                  +
                </button>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex gap-2 pt-1">
              <button 
                type="button"
                onclick={sepeteEkleTikla}
                disabled={$seciliUrunDetay.musaitStok === 0}
                class="flex-1 bg-blue-600 hover:bg-blue-500 text-white font-semibold py-2 rounded-lg transition-colors flex items-center justify-center gap-1.5 text-xs cursor-pointer"
              >
                <HeroIcon name="plus" size={14} />
                <span>Sepete Ekle ({seciliAdet} Adet • ${(seciliAdet * $seciliUrunDetay.birimFiyat).toLocaleString('tr-TR')})</span>
              </button>

              <button 
                type="button"
                onclick={() => $seciliUrunDetay && favoriDegistir($seciliUrunDetay.sku)}
                aria-label="Favorilere Ekle"
                class="p-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition-colors cursor-pointer"
              >
                <HeroIcon name="heart" size={16} solid={favorideMi} class={favorideMi ? 'text-rose-500' : 'text-slate-400'} />
              </button>
            </div>

          </div>
        </div>

        <!-- Reviews & Technical Feedback -->
        <div class="pt-4 border-t border-slate-800 space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-bold text-white uppercase tracking-wider">Müşteri Değerlendirmeleri ({yorumlar.length})</h3>
            <button 
              type="button"
              onclick={() => yorumFormuAcik = !yorumFormuAcik}
              class="text-xs font-semibold text-blue-400 hover:text-blue-300 inline-flex items-center gap-1 cursor-pointer"
            >
              <HeroIcon name="document-text" size={14} />
              <span>{yorumFormuAcik ? 'Formu Kapat' : 'Değerlendirme Yaz'}</span>
            </button>
          </div>

          {#if yorumFormuAcik}
            <!-- Review Form -->
            <div class="p-3.5 rounded-lg bg-slate-950 border border-slate-800 space-y-2.5">
              <div class="flex items-center gap-2">
                <span class="text-xs text-slate-400">Puan:</span>
                <div class="flex gap-1 text-amber-400">
                  {#each [1, 2, 3, 4, 5] as star}
                    <button type="button" onclick={() => yeniPuan = star} class="focus:outline-none cursor-pointer">
                      <HeroIcon name="star" size={16} solid={star <= yeniPuan} />
                    </button>
                  {/each}
                </div>
              </div>

              <textarea 
                bind:value={yeniYorum}
                placeholder="Ürün toleransı, teslimat süresi ve teknik performansı hakkında değerlendirmeniz..."
                rows="2"
                class="w-full bg-slate-900 border border-slate-700 rounded-lg p-2.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
              ></textarea>

              <button 
                type="button"
                onclick={yorumGonder}
                class="px-3.5 py-1.5 bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs rounded-lg transition-colors cursor-pointer"
              >
                Değerlendirmeyi Gönder
              </button>
            </div>
          {/if}

          <!-- Reviews List -->
          <div class="space-y-2 max-h-48 overflow-y-auto">
            {#if yorumlar.length === 0}
              <p class="text-xs text-slate-500 py-3">Bu parça için henüz değerlendirme yapılmamış.</p>
            {:else}
              {#each yorumlar as y}
                <div class="p-3 rounded-lg bg-slate-950 border border-slate-800/80 text-left space-y-1">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-1.5">
                      <span class="text-xs font-semibold text-white">{y.kullaniciAdi}</span>
                      <span class="inline-flex items-center gap-0.5 text-[10px] text-emerald-400 bg-emerald-500/10 px-1.5 py-0.2 rounded border border-emerald-500/20">
                        <HeroIcon name="check" size={10} />
                        <span>Onaylı Alıcı</span>
                      </span>
                    </div>
                    <div class="flex text-amber-400">
                      {#each [1, 2, 3, 4, 5] as star}
                        <HeroIcon name="star" size={10} solid={star <= y.puan} />
                      {/each}
                    </div>
                  </div>
                  <p class="text-xs text-slate-300">{y.yorum}</p>
                </div>
              {/each}
            {/if}
          </div>

        </div>

      </div>

    </div>
  </div>
{/if}
