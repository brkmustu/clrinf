<script lang="ts">
  import type { SiparisKaydi, KullaniciProfili, SiparisDurumTuru } from '../types';

  interface Props {
    siparisler: SiparisKaydi[];
    aktifKullanici: KullaniciProfili;
    yukleniyor: boolean;
    onDurumGuncelle: (siparisId: string, yeniDurum: SiparisDurumTuru, kargoTakipNo?: string) => Promise<void>;
    onSiparisIptal: (siparisId: string, gerekce: string) => Promise<void>;
    onIadeTalebi: (siparisId: string, neden: string) => Promise<void>;
    onYenile: () => Promise<void>;
  }

  let {
    siparisler,
    aktifKullanici,
    yukleniyor,
    onDurumGuncelle,
    onSiparisIptal,
    onIadeTalebi,
    onYenile
  }: Props = $props();

  let secilenSiparis: SiparisKaydi | null = $state(null);
  let iptalGerekcesi = $state('');
  let iadeNedeni = $state('');
  let modalTuru: 'detay' | 'iptal' | 'iade' | null = $state(null);
  let islemde = $state(false);
  let durumFiltresi: string = $state('TUMU');

  const adimlar: { durum: SiparisDurumTuru; baslik: string; ikon: string }[] = [
    { durum: 'Onaylandi', baslik: 'Onaylandı', ikon: '✓' },
    { durum: 'Hazirlaniyor', baslik: 'Hazırlanıyor', ikon: '📦' },
    { durum: 'Paketlendi', baslik: 'Paketlendi', ikon: '🏷️' },
    { durum: 'Kargoda', baslik: 'Kargoda', ikon: '🚚' },
    { durum: 'TeslimEdildi', baslik: 'Teslim Edildi', ikon: '🏠' }
  ];

  function adimIndex(durum: SiparisDurumTuru): number {
    return adimlar.findIndex(a => a.durum === durum);
  }

  let filtrelenmisSiparisler = $derived(
    durumFiltresi === 'TUMU' 
      ? siparisler 
      : siparisler.filter(s => s.durum === durumFiltresi)
  );

  function durumRozeti(durum: SiparisDurumTuru): { sinif: string; etiket: string } {
    switch (durum) {
      case 'Onaylandi': return { sinif: 'badge-info', etiket: 'Onaylandı' };
      case 'Hazirlaniyor': return { sinif: 'badge-warning', etiket: 'Hazırlanıyor' };
      case 'Paketlendi': return { sinif: 'badge-accent', etiket: 'Paketlendi' };
      case 'Kargoda': return { sinif: 'badge-primary', etiket: 'Kargoda' };
      case 'TeslimEdildi': return { sinif: 'badge-success', etiket: 'Teslim Edildi' };
      case 'Tamamlandi': return { sinif: 'badge-success', etiket: 'Tamamlandı' };
      case 'IptalEdildi': return { sinif: 'badge-error', etiket: 'İptal Edildi' };
      case 'IadeTalebi': return { sinif: 'badge-secondary', etiket: 'İade Talebi' };
      case 'IadeTamamlandi': return { sinif: 'badge-neutral', etiket: 'İade Tamamlandı' };
      default: return { sinif: 'badge-ghost', etiket: durum };
    }
  }

  async function durumaIlerle(siparis: SiparisKaydi, hedef: SiparisDurumTuru) {
    islemde = true;
    try {
      await onDurumGuncelle(siparis.siparisId, hedef);
    } finally {
      islemde = false;
    }
  }

  async function iptalOnayla() {
    if (!secilenSiparis || !iptalGerekcesi) return;
    islemde = true;
    try {
      await onSiparisIptal(secilenSiparis.siparisId, iptalGerekcesi);
      modalTuru = null;
      secilenSiparis = null;
      iptalGerekcesi = '';
    } finally {
      islemde = false;
    }
  }

  async function iadeOnayla() {
    if (!secilenSiparis || !iadeNedeni) return;
    islemde = true;
    try {
      await onIadeTalebi(secilenSiparis.siparisId, iadeNedeni);
      modalTuru = null;
      secilenSiparis = null;
      iadeNedeni = '';
    } finally {
      islemde = false;
    }
  }
</script>

<div class="space-y-6">
  <!-- Üst Başlık & Filtreler -->
  <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-base-200/50 p-5 rounded-2xl border border-base-content/5 backdrop-blur-md">
    <div>
      <h2 class="text-xl font-bold flex items-center gap-2">
        <span class="p-2 bg-primary/10 text-primary rounded-xl">📦</span>
        Sipariş Yönetimi (OMS & Lojistik)
      </h2>
      <p class="text-xs text-base-content/60 mt-1">
        Sipariş yaşam döngüsü, depo hazırlık adımları, kargo barkodları ve RMA iade süreçleri
      </p>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <!-- Durum Filtresi -->
      <select bind:value={durumFiltresi} class="select select-sm select-bordered">
        <option value="TUMU">Tüm Durumlar ({siparisler.length})</option>
        <option value="Onaylandi">Onaylananlar</option>
        <option value="Hazirlaniyor">Hazırlananlar</option>
        <option value="Paketlendi">Paketlenenler</option>
        <option value="Kargoda">Kargodakiler</option>
        <option value="TeslimEdildi">Teslim Edilenler</option>
        <option value="IptalEdildi">İptal Edilenler</option>
        <option value="IadeTamamlandi">İadeler</option>
      </select>

      <button onclick={onYenile} class="btn btn-sm btn-ghost gap-1" disabled={yukleniyor}>
        <span class={yukleniyor ? 'loading loading-spinner loading-xs' : ''}>🔄</span>
        Yenile
      </button>
    </div>
  </div>

  <!-- Sipariş Kartları Listesi -->
  {#if filtrelenmisSiparisler.length === 0}
    <div class="card bg-base-200/30 border border-dashed border-base-content/10 p-12 text-center">
      <div class="text-4xl mb-2">📭</div>
      <h3 class="text-base font-bold">Kayıtlı sipariş bulunamadı</h3>
      <p class="text-xs text-base-content/50 mt-1">
        Katalogdan ürün seçip OdemeSaga ile sipariş vererek akışı başlatabilirsiniz.
      </p>
    </div>
  {:else}
    <div class="space-y-4">
      {#each filtrelenmisSiparisler as siparis (siparis.siparisId)}
        {@const rozet = durumRozeti(siparis.durum)}
        {@const idx = adimIndex(siparis.durum)}

        <div class="card bg-base-100 border border-base-content/10 shadow-lg hover:border-primary/30 transition-all">
          <div class="card-body p-6 space-y-4">
            <!-- Kart Üst Barı -->
            <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 border-b border-base-content/5 pb-4">
              <div class="flex items-center gap-3">
                <span class="font-mono text-base font-bold text-primary">{siparis.siparisId}</span>
                <span class={`badge ${rozet.sinif} font-bold text-xs gap-1`}>
                  {rozet.etiket}
                </span>
                {#if siparis.kargoTakipNo}
                  <span class="badge badge-outline text-xs font-mono gap-1">
                    🚚 {siparis.kargoTakipNo}
                  </span>
                {/if}
                {#if siparis.rmaKodu}
                  <span class="badge badge-warning badge-outline text-xs font-mono gap-1">
                    ⚠️ {siparis.rmaKodu}
                  </span>
                {/if}
              </div>

              <div class="text-right flex items-center gap-4">
                <div>
                  <span class="text-xs text-base-content/50">Müşteri: </span>
                  <span class="text-xs font-bold">{siparis.kullaniciId}</span>
                </div>
                <div class="text-lg font-black text-primary">
                  ${siparis.toplamTutar.toLocaleString('tr-TR', { minimumFractionDigits: 2 })}
                </div>
              </div>
            </div>

            <!-- OMS Görsel Durum Stepper'ı (İptal/İade değilse) -->
            {#if siparis.durum !== 'IptalEdildi' && siparis.durum !== 'IadeTamamlandi' && siparis.durum !== 'IadeTalebi'}
              <div class="py-2">
                <ul class="steps steps-horizontal w-full text-xs">
                  {#each adimlar as adim, i}
                    <li class={`step ${i <= idx ? 'step-primary font-bold' : 'text-base-content/40'}`}>
                      {adim.baslik}
                    </li>
                  {/each}
                </ul>
              </div>
            {:else if siparis.durum === 'IptalEdildi'}
              <div class="alert alert-error text-xs p-3 rounded-xl">
                <span>❌ Bu sipariş iptal edilmiştir. Stoklar otomatik olarak envantere iade edilmiştir.</span>
              </div>
            {:else}
              <div class="alert alert-warning text-xs p-3 rounded-xl">
                <span>↩️ Bu sipariş için iade (RMA) işlemi tamamlanmış ve bedel iade edilmiştir.</span>
              </div>
            {/if}

            <!-- Sipariş Kalemleri Özeti -->
            <div class="bg-base-200/40 rounded-xl p-3 text-xs space-y-1">
              <div class="font-semibold text-base-content/60 mb-1">Sipariş Kalemleri:</div>
              {#each siparis.kalemler as k}
                <div class="flex justify-between items-center text-base-content/80">
                  <span>{k.miktar}x <strong>{k.baslik}</strong> ({k.sku})</span>
                  <span class="font-mono">${k.toplamTutar.toFixed(2)}</span>
                </div>
              {/each}
            </div>

            <!-- Alt İşlem Butonları (Rol Yetkilerine Göre) -->
            <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-base-content/5">
              <button 
                onclick={() => { secilenSiparis = siparis; modalTuru = 'detay'; }}
                class="btn btn-xs btn-ghost text-xs gap-1"
              >
                🔍 Detay & Geçmiş
              </button>

              <div class="flex flex-wrap items-center gap-2">
                <!-- Depo Yetkilisi (WarehouseManager / Admin) Aksiyonları -->
                {#if aktifKullanici.rol === 'WarehouseManager' || aktifKullanici.rol === 'Admin'}
                  {#if siparis.durum === 'Onaylandi'}
                    <button 
                      onclick={() => durumaIlerle(siparis, 'Hazirlaniyor')}
                      class="btn btn-xs btn-warning gap-1"
                      disabled={islemde}
                    >
                      📦 Hazırlamaya Başla
                    </button>
                  {:else if siparis.durum === 'Hazirlaniyor'}
                    <button 
                      onclick={() => durumaIlerle(siparis, 'Paketlendi')}
                      class="btn btn-xs btn-accent gap-1"
                      disabled={islemde}
                    >
                      🏷️ Paketle & Barkod Bas
                    </button>
                  {:else if siparis.durum === 'Paketlendi'}
                    <button 
                      onclick={() => durumaIlerle(siparis, 'Kargoda')}
                      class="btn btn-xs btn-primary gap-1"
                      disabled={islemde}
                    >
                      🚚 Kargoya Ver
                    </button>
                  {:else if siparis.durum === 'Kargoda'}
                    <button 
                      onclick={() => durumaIlerle(siparis, 'TeslimEdildi')}
                      class="btn btn-xs btn-success gap-1"
                      disabled={islemde}
                    >
                      🏠 Teslim Edildi Olarak İşaretle
                    </button>
                  {/if}
                {/if}

                <!-- Müşteri / Satınalma (Purchaser / Admin) Aksiyonları -->
                {#if siparis.durum === 'Onaylandi' || siparis.durum === 'Hazirlaniyor'}
                  <button 
                    onclick={() => { secilenSiparis = siparis; modalTuru = 'iptal'; }}
                    class="btn btn-xs btn-error btn-outline gap-1"
                    disabled={islemde}
                  >
                    ❌ Siparişi İptal Et
                  </button>
                {/if}

                {#if siparis.durum === 'TeslimEdildi' || siparis.durum === 'Tamamlandi'}
                  <button 
                    onclick={() => { secilenSiparis = siparis; modalTuru = 'iade'; }}
                    class="btn btn-xs btn-secondary btn-outline gap-1"
                    disabled={islemde}
                  >
                    ↩️ İade Talebi Aç (RMA)
                  </button>
                {/if}
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- DETAY MODALI -->
{#if modalTuru === 'detay' && secilenSiparis}
  <dialog class="modal modal-open">
    <div class="modal-box max-w-2xl bg-base-100 border border-base-content/10 shadow-2xl space-y-4">
      <div class="flex justify-between items-center border-b border-base-content/10 pb-3">
        <div>
          <h3 class="font-bold text-lg flex items-center gap-2">
            <span class="font-mono text-primary">{secilenSiparis.siparisId}</span>
            <span class="badge badge-sm">{secilenSiparis.durum}</span>
          </h3>
          <p class="text-xs text-base-content/50">Sepet ID: {secilenSiparis.sepetId}</p>
        </div>
        <button onclick={() => { modalTuru = null; secilenSiparis = null; }} class="btn btn-sm btn-circle btn-ghost">✕</button>
      </div>

      <!-- Kargo Bilgileri -->
      {#if secilenSiparis.kargoTakipNo}
        <div class="bg-primary/10 border border-primary/20 rounded-xl p-3 flex justify-between items-center text-xs">
          <div>
            <div class="font-bold text-primary">Lojistik Kargo Takibi</div>
            <div>{secilenSiparis.kargoFirmasi || 'Yurtiçi Kargo'}</div>
          </div>
          <div class="font-mono font-black text-sm bg-base-100 px-3 py-1 rounded-lg border border-primary/30">
            {secilenSiparis.kargoTakipNo}
          </div>
        </div>
      {/if}

      <!-- Kalem Tablosu -->
      <table class="table table-xs w-full">
        <thead>
          <tr class="text-base-content/60">
            <th>Ürün & SKU</th>
            <th class="text-center">Miktar</th>
            <th class="text-right">Birim Fiyat</th>
            <th class="text-right">Toplam</th>
          </tr>
        </thead>
        <tbody>
          {#each secilenSiparis.kalemler as k}
            <tr>
              <td>
                <div class="font-bold">{k.baslik}</div>
                <div class="font-mono text-[10px] text-base-content/50">{k.sku}</div>
              </td>
              <td class="text-center">{k.miktar}</td>
              <td class="text-right font-mono">${k.birimFiyat.toFixed(2)}</td>
              <td class="text-right font-mono font-bold">${k.toplamTutar.toFixed(2)}</td>
            </tr>
          {/each}
        </tbody>
      </table>

      <!-- Durum Değişim Tarihçesi (Audit Log) -->
      <div class="space-y-2 pt-2 border-t border-base-content/10">
        <h4 class="font-bold text-xs text-base-content/70">Durum Geçiş Tarihçesi (Audit Trail)</h4>
        <div class="space-y-2 max-h-40 overflow-y-auto pr-1">
          {#each secilenSiparis.gecmis as g}
            <div class="flex items-start justify-between text-xs bg-base-200/50 p-2 rounded-lg">
              <div>
                <span class="font-bold text-primary">{g.durum}</span>
                <span class="text-base-content/70 ml-1">- {g.aciklama || 'Durum güncellendi.'}</span>
              </div>
              <div class="text-[10px] text-base-content/40 text-right whitespace-nowrap ml-2">
                <div>{g.degistirenKullanici}</div>
                <div>{new Date(g.zaman).toLocaleTimeString('tr-TR')}</div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <div class="modal-action">
        <button onclick={() => { modalTuru = null; secilenSiparis = null; }} class="btn btn-sm btn-ghost">Kapat</button>
      </div>
    </div>
  </dialog>
{/if}

<!-- İPTAL MODALI -->
{#if modalTuru === 'iptal' && secilenSiparis}
  <dialog class="modal modal-open">
    <div class="modal-box bg-base-100 border border-error/30 shadow-2xl space-y-4">
      <h3 class="font-bold text-base text-error flex items-center gap-2">
        <span>⚠️</span> Sipariş İptal Onayı ({secilenSiparis.siparisId})
      </h3>
      <p class="text-xs text-base-content/70">
        Sipariş iptal edildiğinde rezerve/tahsisli stoklar anında envantere iade edilir ve ödeme provizyonu telafi edilir.
      </p>

      <div class="form-control">
        <label class="label text-xs font-bold" for="iptal-gerekce">İptal Gerekçesi:</label>
        <textarea 
          id="iptal-gerekce"
          bind:value={iptalGerekcesi} 
          class="textarea textarea-bordered textarea-sm w-full" 
          placeholder="Müşteri vazgeçti, yanlış adet seçildi vb."
          rows="3"
        ></textarea>
      </div>

      <div class="modal-action">
        <button onclick={() => { modalTuru = null; secilenSiparis = null; }} class="btn btn-sm btn-ghost" disabled={islemde}>Vazgeç</button>
        <button onclick={iptalOnayla} class="btn btn-sm btn-error" disabled={islemde || !iptalGerekcesi}>
          {islemde ? 'İptal Ediliyor...' : 'İptali Kesinleştir'}
        </button>
      </div>
    </div>
  </dialog>
{/if}

<!-- İADE (RMA) MODALI -->
{#if modalTuru === 'iade' && secilenSiparis}
  <dialog class="modal modal-open">
    <div class="modal-box bg-base-100 border border-secondary/30 shadow-2xl space-y-4">
      <h3 class="font-bold text-base text-secondary flex items-center gap-2">
        <span>↩️</span> Müşteri RMA İade Talebi ({secilenSiparis.siparisId})
      </h3>
      <p class="text-xs text-base-content/70">
        Teslim edilen sipariş için iade talebi açılacak, depo yetkilisi onayıyla ürünler depoya geri girecektir.
      </p>

      <div class="form-control">
        <label class="label text-xs font-bold" for="iade-nedeni">İade Nedeni:</label>
        <textarea 
          id="iade-nedeni"
          bind:value={iadeNedeni} 
          class="textarea textarea-bordered textarea-sm w-full" 
          placeholder="Kusurlu/Kırık ürün, yanlış model gönderimi vb."
          rows="3"
        ></textarea>
      </div>

      <div class="modal-action">
        <button onclick={() => { modalTuru = null; secilenSiparis = null; }} class="btn btn-sm btn-ghost" disabled={islemde}>Vazgeç</button>
        <button onclick={iadeOnayla} class="btn btn-sm btn-secondary" disabled={islemde || !iadeNedeni}>
          {islemde ? 'Talep Açılıyor...' : 'RMA Talebini Onayla'}
        </button>
      </div>
    </div>
  </dialog>
{/if}
