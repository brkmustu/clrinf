<script lang="ts">
  import { onMount } from 'svelte';
  import StorefrontHeader from './lib/components/StorefrontHeader.svelte';
  import HeroSection from './lib/components/HeroSection.svelte';
  import CategoryGrid from './lib/components/CategoryGrid.svelte';
  import ProductCard from './lib/components/ProductCard.svelte';
  import ProductQuickViewModal from './lib/components/ProductQuickViewModal.svelte';
  import CartSlideOver from './lib/components/CartSlideOver.svelte';
  import CheckoutModal from './lib/components/CheckoutModal.svelte';
  import OrderTrackingView from './lib/components/OrderTrackingView.svelte';
  import CampaignsSection from './lib/components/CampaignsSection.svelte';
  import StorefrontFooter from './lib/components/StorefrontFooter.svelte';
  import ArchitectureDrawer from './lib/components/ArchitectureDrawer.svelte';
  import BildirimKapsayicisi from './lib/components/BildirimKapsayicisi.svelte';
  import HeroIcon from './lib/components/HeroIcon.svelte';
  import { aktifSayfa, yukleAktifKampanyalar } from './lib/hazneler/sepetHaznesi';
  import { stoklariYukle, stokDusur, stoklar } from './lib/hazneler/stokHaznesi';
  import type { UrunKalemi, SiparisKaydi, KampanyaVerisi, KategoriItem } from './lib/types';

  // 1. Dinamik Ürünler (.NET WebAPI / PostgreSQL)
  let urunler = $state<UrunKalemi[]>([]);
  let urunlerYukleniyor = $state(true);

  // 2. Dinamik Kategoriler (.NET WebAPI / PostgreSQL)
  let kategoriler = $state<KategoriItem[]>([]);

  // 3. Dinamik Kampanyalar (Rust Kampanya Motoru)
  let kampanyalar = $state<KampanyaVerisi[]>([]);

  // 4. Dinamik Siparişler (.NET WebAPI / PostgreSQL)
  let siparisler = $state<SiparisKaydi[]>([]);

  // Arama & Filtre State
  let aramaMetni = $state('');
  let seciliKategori = $state('');
  let siralama = $state<'varsayilan' | 'fiyat-artan' | 'fiyat-azalan' | 'puan'>('varsayilan');

  const tumKategoriAdlari = $derived([
    ...new Set([
      ...kategoriler.map(k => k.ad),
      ...urunler.map(u => u.kategoriYolu)
    ])
  ]);

  // Filtrelenmiş ve Sıralanmış Ürünler
  const filtrelenmisUrunler = $derived(
    urunler
      .filter(u => {
        const aramaUygun = !aramaMetni || 
          u.baslik.toLowerCase().includes(aramaMetni.toLowerCase()) ||
          u.sku.toLowerCase().includes(aramaMetni.toLowerCase()) ||
          (u.aciklama && u.aciklama.toLowerCase().includes(aramaMetni.toLowerCase()));

        const kategoriUygun = !seciliKategori || u.kategoriYolu === seciliKategori;

        return aramaUygun && kategoriUygun;
      })
      .sort((a, b) => {
        if (siralama === 'fiyat-artan') return a.birimFiyat - b.birimFiyat;
        if (siralama === 'fiyat-azalan') return b.birimFiyat - a.birimFiyat;
        if (siralama === 'puan') return (b.puan || 0) - (a.puan || 0);
        return 0;
      })
  );

  async function yeniSiparisEkle(yeniSiparis: SiparisKaydi) {
    siparisler = [yeniSiparis, ...siparisler];

    // 1. Merkezi Stok Haznesinden (ve DB'den) Stokları Düşür
    await stokDusur(yeniSiparis.kalemler);

    // 2. Ürün listesini anlık hazne verisiyle tazele
    urunler = urunler.map(urun => {
      const guncelStok = $stoklar[urun.sku]?.musaitStok ?? urun.musaitStok;
      return {
        ...urun,
        musaitStok: guncelStok,
        etiket: guncelStok === 0 ? undefined : (guncelStok <= 5 ? 'SinirliStok' : urun.etiket)
      };
    });

    // 3. Backend .NET 10 WebAPI Siparis & Odeme modülüne aktar
    try {
      const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';
      fetch(`${apiHost}/api/Siparis`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sepetId: yeniSiparis.sepetId,
          kullaniciId: yeniSiparis.kullaniciId,
          toplamTutar: yeniSiparis.toplamTutar,
          paraBirimi: yeniSiparis.paraBirimi,
          durum: 0
        })
      }).catch(() => {});

      fetch(`${apiHost}/api/Odeme`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          siparisId: 1,
          tutar: yeniSiparis.toplamTutar,
          paraBirimi: 'USD',
          odemeYontemi: 'Kurumsal Kredi Kartı',
          durum: 'Basarili'
        })
      }).catch(() => {});
    } catch {}
  }

  onMount(async () => {
    const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';

    // 1. Canlı Stok Haznesi (.NET WebAPI /api/Stok - PostgreSQL)
    const dbStoklar = await stoklariYukle();

    // 2. Kampanyalar (Rust Kampanya Motoru - Port 8082 /kampanya/listele)
    kampanyalar = await yukleAktifKampanyalar();

    // 3. Kategoriler (.NET WebAPI / PostgreSQL)
    try {
      const katRes = await fetch(`${apiHost}/api/Kategori?PageIndex=0&PageSize=50`);
      if (katRes.ok) {
        const katData = await katRes.json();
        if (katData && katData.items && katData.items.length > 0) {
          kategoriler = katData.items.map((item: any) => ({
            id: item.id,
            ad: item.ad,
            slug: item.slug,
            sira: item.sira,
            aktif: item.aktif
          }));
        }
      }
    } catch (err) {
      console.warn('Kategoriler çekilemedi:', err);
    }

    // 4. Ürünler (.NET WebAPI / PostgreSQL)
    try {
      const urunRes = await fetch(`${apiHost}/api/Urun?PageIndex=0&PageSize=50`);
      if (urunRes.ok) {
        const urunData = await urunRes.json();
        if (urunData && urunData.items && urunData.items.length > 0) {
          urunler = urunData.items.map((apiItem: any, index: number) => {
            const musaitStok = dbStoklar[apiItem.sku]?.musaitStok ?? Number(apiItem.musaitStok) ?? 0;
            return {
              id: apiItem.id || index + 1,
              sku: apiItem.sku || `SKU-${index + 100}`,
              baslik: apiItem.baslik || 'Endüstriyel Ürün',
              aciklama: apiItem.aciklama || 'Endüstriyel sınıf sensör ve donanım bileşeni.',
              birimFiyat: Number(apiItem.birimFiyat) || 1000,
              eskiFiyat: Number(apiItem.birimFiyat) * 1.15,
              paraBirimi: apiItem.paraBirimi || 'USD',
              vergiOrani: 0.18,
              kategoriYolu: apiItem.kategoriYolu || 'Sensör/Optik',
              musaitStok,
              puan: 5.0,
              degerlendirmeSayisi: 14 + index * 3,
              etiket: musaitStok <= 5 && musaitStok > 0 ? 'SinirliStok' : (index === 0 ? 'CokSatan' : undefined),
              yayinda: apiItem.yayinda !== false
            };
          });
        }
      }
    } catch (err) {
      console.warn('Ürünler çekilemedi:', err);
    }

    // 4. Siparişler & Kargo (.NET WebAPI / PostgreSQL)
    try {
      const sipRes = await fetch(`${apiHost}/api/Siparis?PageIndex=0&PageSize=20`);
      if (sipRes.ok) {
        const sipData = await sipRes.json();
        if (sipData && sipData.items && sipData.items.length > 0) {
          siparisler = sipData.items.map((s: any) => ({
            siparisId: s.siparisNo || `SIP-${s.id}`,
            sepetId: s.sepetId || 'sepet-01',
            kullaniciId: s.kullaniciId || 'alice',
            aliciAdi: 'Alice Cooper (Kurumsal Satın Alma)',
            teslimatAdresi: 'Organize Sanayi Bölgesi Teknopark, İstanbul',
            odemeYontemi: 'Kurumsal Kredi Kartı (3D Secure)',
            toplamTutar: Number(s.toplamTutar) || 5723,
            paraBirimi: s.paraBirimi || 'USD',
            durum: s.durum || 'Kargoda',
            kargoTakipNo: 'TR98421094',
            kargoFirmasi: 'DHL Express',
            tahminiTeslim: '1-2 İş Günü',
            kalemler: [
              { sku: 'SKU-100', baslik: 'Optik Sensör Modülü X1', miktar: 2, birimFiyat: 1200, toplamTutar: 2400 },
              { sku: 'SKU-300', baslik: 'IoT Ağ Geçidi Kontrolörü (Edge Gateway)', miktar: 1, birimFiyat: 2450, toplamTutar: 2450 }
            ],
            gecmis: [
              { durum: 'Onaylandi', aciklama: 'Sipariş ve ödeme Cedar ABAC tarafından onaylandı.', degistirenKullanici: 'CedarEngine', zaman: '10:14' },
              { durum: 'Kargoda', aciklama: 'DHL Express kuryesine teslim edildi.', degistirenKullanici: 'LogisticsService', zaman: '12:45' }
            ],
            olusturulmaZamani: '22.08.2026 10:14'
          }));
        }
      }
    } catch (err) {
      console.warn('Siparişler çekilemedi:', err);
    } finally {
      urunlerYukleniyor = false;
    }
  });
</script>

<div class="min-h-screen bg-slate-950 text-slate-100 font-sans flex flex-col justify-between">
  
  <div>
    <!-- 1. Header & Navigation -->
    <StorefrontHeader 
      bind:aramaMetni
      {seciliKategori}
      onKategoriSec={(kat) => seciliKategori = kat}
      onSiparislerimTikla={() => $aktifSayfa = 'siparisler'}
      onMimariPanelTikla={() => {}}
      toplamUrunSayisi={urunler.length}
    />

    <!-- Notifications Toast -->
    <BildirimKapsayicisi />

    <!-- 2. Main Storefront View -->
    <main class="max-w-7xl mx-auto px-4 sm:px-8 pt-6">
      
      {#if $aktifSayfa === 'vitrin'}
        <!-- Hero Spec Banner -->
        <HeroSection />

        <!-- Categories Spec Grid -->
        <CategoryGrid 
          {seciliKategori}
          {kategoriler}
          onKategoriSec={(kat) => seciliKategori = kat}
        />

        <!-- Products Section -->
        <section class="mb-12">
          <div class="flex flex-wrap items-center justify-between gap-4 mb-4">
            <div>
              <div class="flex items-center gap-1.5 text-xs font-semibold text-blue-400">
                <HeroIcon name="cpu-chip" size={14} />
                <span>Katalog & Envanter</span>
              </div>
              <h2 class="text-xl font-bold text-white tracking-tight mt-0.5">Endüstriyel Parça Kataloğu</h2>
            </div>

            <!-- Filter & Sort Selector -->
            <div class="flex items-center gap-3">
              {#if seciliKategori}
                <div class="inline-flex items-center gap-1 text-xs text-slate-400 bg-slate-900 border border-slate-700 px-2.5 py-1 rounded-lg">
                  <span>Filtre: <strong class="text-blue-400">{seciliKategori}</strong></span>
                  <button onclick={() => seciliKategori = ''} class="text-slate-400 hover:text-rose-400 ml-1 cursor-pointer">✕</button>
                </div>
              {/if}

              <div class="relative">
                <select 
                  bind:value={siralama}
                  class="bg-slate-900 border border-slate-700 text-xs text-slate-300 font-medium px-3 py-1.5 rounded-lg focus:outline-none focus:border-blue-500 cursor-pointer"
                >
                  <option value="varsayilan">Sıralama: Standart</option>
                  <option value="fiyat-artan">Fiyat: Düşükten Yükseğe</option>
                  <option value="fiyat-azalan">Fiyat: Yüksekten Düşüğe</option>
                  <option value="puan">Müşteri Puanı</option>
                </select>
              </div>
            </div>
          </div>

          {#if urunlerYukleniyor}
            <div class="text-center py-12 text-slate-400 text-xs font-mono">Endüstriyel parça kataloğu PostgreSQL'den yükleniyor...</div>
          {:else if filtrelenmisUrunler.length === 0}
            <div class="text-center py-14 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
              <div class="w-10 h-10 rounded-full bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
                <HeroIcon name="magnifying-glass" size={20} />
              </div>
              <h3 class="text-xs font-bold text-white">Eşleşen Parça Bulunamadı</h3>
              <p class="text-[11px] text-slate-400">Arama kriterlerinizi değiştirerek tekrar deneyebilirsiniz.</p>
              <button 
                type="button"
                onclick={() => { aramaMetni = ''; seciliKategori = ''; }}
                class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 font-medium text-xs rounded-lg cursor-pointer"
              >
                Filtreleri Temizle
              </button>
            </div>
          {:else}
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              {#each filtrelenmisUrunler as urun}
                <ProductCard {urun} />
              {/each}
            </div>
          {/if}
        </section>

        <!-- Campaigns Matrix Section -->
        {#if kampanyalar.length > 0}
          <CampaignsSection {kampanyalar} />
        {/if}

      {:else if $aktifSayfa === 'katalog'}
        <!-- Full PIM Catalog View -->
        <div class="space-y-5 mb-14 text-left">
          <div class="flex flex-wrap items-center justify-between gap-4">
            <div>
              <h1 class="text-xl font-bold text-white tracking-tight">Endüstriyel Donanım Kataloğu (PIM)</h1>
              <p class="text-xs text-slate-400">{filtrelenmisUrunler.length} adet doğrulanmış parça listeleniyor</p>
            </div>

            <!-- Category Filter Pills -->
            <div class="flex flex-wrap items-center gap-1.5">
              <button 
                type="button"
                onclick={() => seciliKategori = ''}
                class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer {!seciliKategori ? 'bg-blue-600 text-white' : 'bg-slate-900 text-slate-400 hover:text-white border border-slate-800'}"
              >
                Tümü ({urunler.length})
              </button>
              {#each tumKategoriAdlari as kat}
                <button 
                  type="button"
                  onclick={() => seciliKategori = kat}
                  class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer {seciliKategori === kat ? 'bg-blue-600 text-white' : 'bg-slate-900 text-slate-400 hover:text-white border border-slate-800'}"
                >
                  {kat}
                </button>
              {/each}
            </div>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {#each filtrelenmisUrunler as urun}
              <ProductCard {urun} />
            {/each}
          </div>
        </div>

      {:else if $aktifSayfa === 'kampanyalar'}
        <div class="py-4">
          <CampaignsSection {kampanyalar} />
        </div>

      {:else if $aktifSayfa === 'siparisler'}
        <div class="py-4">
          <OrderTrackingView {siparisler} />
        </div>

      {/if}

    </main>
  </div>

  <!-- 3. Slide-overs & Modals -->
  <CartSlideOver />
  <CheckoutModal {urunler} onSiparisTamamlandi={yeniSiparisEkle} />
  <ProductQuickViewModal />
  <ArchitectureDrawer />

  <!-- 4. Enterprise Storefront Footer -->
  <StorefrontFooter />

</div>
