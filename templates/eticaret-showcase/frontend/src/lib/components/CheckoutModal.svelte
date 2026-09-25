<script lang="ts">
  import { sepet, odemeModalAcik, sepetiTemizle, aktifSayfa } from '../hazneler/sepetHaznesi';
  import { aktifKullanici } from '../hazneler/kimlikHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import type { SiparisKaydi, UrunKalemi } from '../types';
  import HeroIcon from './HeroIcon.svelte';

  interface Props {
    urunler?: UrunKalemi[];
    onSiparisTamamlandi: (siparis: SiparisKaydi) => void;
  }

  let { urunler = [], onSiparisTamamlandi }: Props = $props();

  let adim = $state<1 | 2 | 3 | 4>(1);
  let isleniyor = $state(false);

  // Form Verileri
  let aliciAdi = $state('Alice Cooper');
  let eposta = $state('alice@clrinf.dev');
  let telefon = $state('+90 (555) 123-4567');
  let adres = $state('Organize Sanayi Bölgesi 4. Cadde No:12, Teknopark Kat:3, İstanbul');
  let kargoFirmasi = $state('DHL Express');
  let kartNumarasi = $state('4532 •••• •••• 8890');
  let sonKullanma = $state('12/28');
  let cvv = $state('342');
  let siparisNo = $state('');

  function sonrakiAdim() {
    if (adim < 3) {
      adim++;
    } else if (adim === 3) {
      siparisOlustur();
    }
  }

  function oncekiAdim() {
    if (adim > 1) adim--;
  }

  async function siparisOlustur() {
    // 1. Stok Doğrulaması
    for (const kalem of $sepet.kalemler) {
      const urun = urunler.find(u => u.sku === kalem.sku);
      if (urun && kalem.miktar > urun.musaitStok) {
        bildirimEkle({
          tur: 'uyari',
          baslik: 'Sipariş Reddedildi (Yetersiz Stok)',
          mesaj: `${kalem.baslik} için talep edilen ${kalem.miktar} adet, mevcut ${urun.musaitStok} adetlik stok sınırını aşıyor.`,
          sure: 5000
        });
        return;
      }
    }

    isleniyor = true;
    siparisNo = 'SIP-' + Math.floor(100000 + Math.random() * 900000);

    setTimeout(() => {
      const yeniSiparis: SiparisKaydi = {
        siparisId: siparisNo,
        sepetId: $sepet.sepetId,
        kullaniciId: $aktifKullanici.id,
        aliciAdi,
        teslimatAdresi: adres,
        odemeYontemi: 'Kurumsal Kredi Kartı (3D Secure)',
        toplamTutar: $sepet.genelToplam,
        paraBirimi: 'USD',
        durum: 'Hazirlaniyor',
        kargoTakipNo: 'TR' + Math.floor(10000000 + Math.random() * 90000000),
        kargoFirmasi: kargoFirmasi,
        tahminiTeslim: '1-2 İş Günü',
        kalemler: $sepet.kalemler.map(k => ({
          sku: k.sku,
          baslik: k.baslik,
          miktar: k.miktar,
          birimFiyat: k.birimFiyat,
          toplamTutar: k.toplamTutar
        })),
        gecmis: [
          {
            durum: 'Onaylandi',
            aciklama: 'Sipariş ve ödeme Cedar ABAC tarafından onaylandı.',
            degistirenKullanici: 'System (CedarEngine)',
            zaman: new Date().toLocaleTimeString('tr-TR')
          },
          {
            durum: 'Hazirlaniyor',
            aciklama: 'Depo stok rezervasyonu tamamlandı, paketleme sırasına alındı.',
            degistirenKullanici: 'WarehouseService',
            zaman: new Date().toLocaleTimeString('tr-TR')
          }
        ],
        olusturulmaZamani: new Date().toLocaleString('tr-TR')
      };

      onSiparisTamamlandi(yeniSiparis);
      sepetiTemizle();
      isleniyor = false;
      adim = 4;

      bildirimEkle({
        tur: 'basari',
        baslik: 'Siparişiniz Alındı',
        mesaj: `${siparisNo} numaralı satın alma siparişi oluşturuldu.`,
        sure: 5000
      });
    }, 800);
  }

  function modalKapat() {
    $odemeModalAcik = false;
    adim = 1;
  }
</script>

{#if $odemeModalAcik}
  <!-- Backdrop -->
  <div 
    onclick={modalKapat}
    onkeydown={(e) => { if (e.key === 'Escape') modalKapat(); }}
    role="button"
    tabindex="0"
    class="fixed inset-0 bg-slate-950/80 backdrop-blur-xs z-50 transition-opacity"
  >
  </div>

  <!-- Modal Dialog -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <div class="w-full max-w-xl bg-slate-900 border border-slate-800 rounded-xl shadow-2xl overflow-hidden text-left">
      
      <!-- Modal Header & Stepper -->
      <div class="p-5 bg-slate-950 border-b border-slate-800">
        <div class="flex items-center justify-between mb-3.5">
          <div class="flex items-center gap-2.5">
            <div class="w-8 h-8 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
              <HeroIcon name="credit-card" size={18} />
            </div>
            <div>
              <h3 class="text-sm font-bold text-white">Güvenli Sipariş Onayı & Ödeme</h3>
              <p class="text-[11px] text-slate-400">256-Bit SSL & Rust Cedar ABAC Güvenlik Protokolü</p>
            </div>
          </div>
          <button onclick={modalKapat} class="text-slate-400 hover:text-white p-1 rounded hover:bg-slate-800 transition-colors cursor-pointer">
            <HeroIcon name="x-mark" size={16} />
          </button>
        </div>

        {#if adim < 4}
          <!-- Step Indicator -->
          <div class="grid grid-cols-3 gap-2">
            <div class="flex items-center gap-2 p-2 rounded-lg {adim >= 1 ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20' : 'bg-slate-900 text-slate-500'}">
              <span class="w-4 h-4 rounded-full {adim >= 1 ? 'bg-blue-600 text-white' : 'bg-slate-800'} flex items-center justify-center text-[10px] font-bold">1</span>
              <span class="text-xs font-medium">Teslimat</span>
            </div>
            <div class="flex items-center gap-2 p-2 rounded-lg {adim >= 2 ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20' : 'bg-slate-900 text-slate-500'}">
              <span class="w-4 h-4 rounded-full {adim >= 2 ? 'bg-blue-600 text-white' : 'bg-slate-800'} flex items-center justify-center text-[10px] font-bold">2</span>
              <span class="text-xs font-medium">Lojistik</span>
            </div>
            <div class="flex items-center gap-2 p-2 rounded-lg {adim >= 3 ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20' : 'bg-slate-900 text-slate-500'}">
              <span class="w-4 h-4 rounded-full {adim >= 3 ? 'bg-blue-600 text-white' : 'bg-slate-800'} flex items-center justify-center text-[10px] font-bold">3</span>
              <span class="text-xs font-medium">Ödeme</span>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-4 max-h-[60vh] overflow-y-auto">

        {#if adim === 1}
          <!-- Step 1: Shipping Address -->
          <div class="space-y-3 text-xs">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div>
                <label for="checkout-alici" class="block text-slate-400 mb-1 font-medium">Fatura Ünvanı / Alıcı</label>
                <input id="checkout-alici" type="text" bind:value={aliciAdi} class="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-blue-500 text-xs" />
              </div>
              <div>
                <label for="checkout-eposta" class="block text-slate-400 mb-1 font-medium">Kurumsal E-Posta</label>
                <input id="checkout-eposta" type="email" bind:value={eposta} class="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-blue-500 text-xs" />
              </div>
            </div>

            <div>
              <label for="checkout-telefon" class="block text-slate-400 mb-1 font-medium">İletişim Telefonu</label>
              <input id="checkout-telefon" type="text" bind:value={telefon} class="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-blue-500 text-xs" />
            </div>

            <div>
              <label for="checkout-adres" class="block text-slate-400 mb-1 font-medium">Fabrika / Depo Teslimat Adresi</label>
              <textarea id="checkout-adres" rows="2" bind:value={adres} class="w-full bg-slate-950 border border-slate-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-blue-500 text-xs"></textarea>
            </div>
          </div>

        {:else if adim === 2}
          <!-- Step 2: Logistics -->
          <div class="space-y-2.5">
            <span class="block text-xs text-slate-400 mb-1 font-medium">Lojistik Taşıyıcı Seçimi:</span>
            
            <label class="flex items-center justify-between p-3.5 rounded-lg border cursor-pointer transition-colors {kargoFirmasi === 'DHL Express' ? 'bg-blue-500/10 border-blue-500' : 'bg-slate-950 border-slate-800'}">
              <div class="flex items-center gap-3">
                <input type="radio" bind:group={kargoFirmasi} value="DHL Express" class="text-blue-500" />
                <div>
                  <div class="text-xs font-bold text-white">DHL Express (Hızlı Hava Kargo)</div>
                  <div class="text-[11px] text-slate-400">1-2 İş Günü Teslimat • Gümrükleme Dahil</div>
                </div>
              </div>
              <span class="text-xs font-bold text-emerald-400">ÜCRETSİZ</span>
            </label>

            <label class="flex items-center justify-between p-3.5 rounded-lg border cursor-pointer transition-colors {kargoFirmasi === 'Yurtiçi Kargo' ? 'bg-blue-500/10 border-blue-500' : 'bg-slate-950 border-slate-800'}">
              <div class="flex items-center gap-3">
                <input type="radio" bind:group={kargoFirmasi} value="Yurtiçi Kargo" class="text-blue-500" />
                <div>
                  <div class="text-xs font-bold text-white">Yurtiçi Kargo Endüstriyel</div>
                  <div class="text-[11px] text-slate-400">2-3 İş Günü Teslimat</div>
                </div>
              </div>
              <span class="text-xs font-bold text-emerald-400">ÜCRETSİZ</span>
            </label>
          </div>

        {:else if adim === 3}
          <!-- Step 3: Payment & Summary -->
          <div class="space-y-3.5 text-xs">
            <div class="p-3.5 rounded-lg bg-slate-950 border border-slate-800 space-y-2.5">
              <div class="flex items-center justify-between">
                <span class="text-slate-400 font-semibold uppercase tracking-wider text-[10px]">Kurumsal Kart Bilgileri</span>
                <span class="text-blue-400 font-medium text-[11px]">3D Secure</span>
              </div>
              
              <div>
                <label for="checkout-kart-no" class="block text-slate-400 mb-1 font-medium">Kart Numarası</label>
                <input id="checkout-kart-no" type="text" bind:value={kartNumarasi} class="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-white font-mono text-xs" />
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div>
                  <label for="checkout-son-kullanma" class="block text-slate-400 mb-1 font-medium">Son Kullanma</label>
                  <input id="checkout-son-kullanma" type="text" bind:value={sonKullanma} class="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-white font-mono text-xs" />
                </div>
                <div>
                  <label for="checkout-cvv" class="block text-slate-400 mb-1 font-medium">CVV</label>
                  <input id="checkout-cvv" type="text" bind:value={cvv} class="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-white font-mono text-xs" />
                </div>
              </div>
            </div>

            <!-- Order Total Summary -->
            <div class="p-3 rounded-lg bg-slate-950 border border-slate-800 flex items-center justify-between">
              <div>
                <div class="text-[11px] text-slate-300">Ödenecek Tutar (KDV Dahil)</div>
                <div class="text-[10px] text-slate-400">{$sepet.kalemler.length} kalem parça</div>
              </div>
              <div class="text-base font-bold text-white font-mono">
                ${$sepet.genelToplam.toLocaleString('tr-TR')} USD
              </div>
            </div>
          </div>

        {:else if adim === 4}
          <!-- Step 4: Success Screen -->
          <div class="text-center py-6 space-y-3">
            <div class="w-12 h-12 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center justify-center mx-auto">
              <HeroIcon name="check" size={24} />
            </div>
            
            <h3 class="text-base font-bold text-white">Siparişiniz Başarıyla Onaylandı</h3>
            <p class="text-xs text-slate-400 max-w-sm mx-auto">
              Sipariş Numaranız: <strong class="text-blue-400 font-mono">{siparisNo}</strong>. Depo hazırlığı başlatıldı.
            </p>

            <div class="pt-3 flex justify-center gap-2.5">
              <button 
                type="button"
                onclick={() => { modalKapat(); $aktifSayfa = 'siparisler'; }}
                class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs rounded-lg transition-colors cursor-pointer"
              >
                Sipariş Takibine Git
              </button>

              <button 
                type="button"
                onclick={modalKapat}
                class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs rounded-lg transition-colors cursor-pointer"
              >
                Alışverişe Devam Et
              </button>
            </div>
          </div>

        {/if}

      </div>

      <!-- Modal Footer Actions -->
      {#if adim < 4}
        <div class="p-4 bg-slate-950 border-t border-slate-800 flex justify-between items-center">
          {#if adim > 1}
            <button 
              type="button"
              onclick={oncekiAdim}
              class="px-3.5 py-2 text-slate-400 hover:text-white text-xs font-medium rounded-lg transition-colors cursor-pointer"
            >
              ← Geri
            </button>
          {:else}
            <div></div>
          {/if}

          <button 
            type="button"
            onclick={sonrakiAdim}
            disabled={isleniyor}
            class="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white font-semibold text-xs px-5 py-2 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
          >
            {#if isleniyor}
              <span>İşleniyor...</span>
            {:else if adim === 3}
              <HeroIcon name="shield-check" size={14} />
              <span>Siparişi Onayla & Öde</span>
            {:else}
              <span>Devam Et →</span>
            {/if}
          </button>
        </div>
      {/if}

    </div>
  </div>
{/if}
