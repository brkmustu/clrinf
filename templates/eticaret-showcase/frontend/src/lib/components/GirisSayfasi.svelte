<script lang="ts">
  import { aktifKullanici } from '../hazneler/kimlikHaznesi';
  import type { SiparisKaydi } from '../types';

  interface Props {
    siparisler: SiparisKaydi[];
    sekmeDegistir: (s: string) => void;
  }

  let { siparisler = [], sekmeDegistir }: Props = $props();

  let onaylananSiparisler = $derived(siparisler.filter((s) => s.durum === 'Onaylandi' || s.durum === 'TeslimEdildi' || s.durum === 'Tamamlandi' || s.durum === 'Kargoda' || s.durum === 'Paketlendi' || s.durum === 'Hazirlaniyor'));
  let toplamCiro = $derived(onaylananSiparisler.reduce((toplam, s) => toplam + s.toplamTutar, 0));
  let telafiEdilenler = $derived(siparisler.filter((s) => s.durum === 'IptalEdildi').length);
</script>

<div class="space-y-8">
  <!-- Hoş Geldiniz Banner -->
  <div class="card bg-base-200 border border-base-content/10 shadow-xl p-6 sm:p-8">
    <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
      <div class="space-y-1.5">
        <div class="flex items-center gap-2">
          <span class="badge badge-primary badge-sm font-mono">{$aktifKullanici?.rol}</span>
          <span class="text-xs font-mono text-base-content/50">clrinf E-Ticaret Referans Şablonu</span>
        </div>
        <h2 class="text-2xl font-black text-base-content">
          Hoş Geldiniz, {$aktifKullanici?.ad}
        </h2>
        <p class="text-xs text-base-content/70 max-w-2xl leading-relaxed">
          {$aktifKullanici?.departman} paneli üzerinden ürün kataloğunu inceleyebilir, sepet oluşturup 5 adımlı dağıtık <strong>OdemeSaga</strong> iş akışını ve OMS sipariş yaşam döngüsünü test edebilirsiniz.
        </p>
      </div>

      <div class="bg-base-300 p-4 rounded-xl border border-base-content/5 font-mono text-xs space-y-1 shrink-0">
        <div class="text-base-content/50 text-[10px] uppercase tracking-wider">Yetki Matrisi</div>
        <div>Sipariş Limiti: <strong class="text-primary">{$aktifKullanici?.siparisLimiti > 100000 ? 'Sınırsız (Admin)' : $aktifKullanici?.siparisLimiti === 0 ? 'Sipariş Yetkisi Yok' : `$${$aktifKullanici?.siparisLimiti.toLocaleString()} / işlem`}</strong></div>
        <div>Fiyat Güncelleme: <strong class="{$aktifKullanici?.rol === 'Admin' || $aktifKullanici?.rol === 'PricingOfficer' ? 'text-success' : 'text-base-content/40'}">{$aktifKullanici?.rol === 'Admin' || $aktifKullanici?.rol === 'PricingOfficer' ? 'İzinli (Cedar)' : 'Kısıtlı'}</strong></div>
        <div>Kupon Yönetimi: <strong class="{$aktifKullanici?.rol === 'Admin' || $aktifKullanici?.rol === 'MarketingLead' ? 'text-success' : 'text-base-content/40'}">{$aktifKullanici?.rol === 'Admin' || $aktifKullanici?.rol === 'MarketingLead' ? 'İzinli (Cedar)' : 'Kısıtlı'}</strong></div>
        <div>Depo & Kargo: <strong class="{$aktifKullanici?.rol === 'Admin' || $aktifKullanici?.rol === 'WarehouseManager' ? 'text-success' : 'text-base-content/40'}">{$aktifKullanici?.rol === 'Admin' || $aktifKullanici?.rol === 'WarehouseManager' ? 'İzinli (Cedar)' : 'Kısıtlı'}</strong></div>
      </div>
    </div>
  </div>

  <!-- KPI İstatistik Kartları -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
    <div class="card bg-base-200 border border-base-content/10 shadow-lg p-5">
      <div class="text-xs font-mono text-base-content/50 uppercase tracking-wider">Toplam E-Ticaret Cirosu</div>
      <div class="text-2xl font-black font-mono text-primary mt-2">
        ${toplamCiro.toLocaleString('tr-TR', { minimumFractionDigits: 2 })}
      </div>
      <div class="text-[11px] text-base-content/50 mt-1">5 Adımlı Saga Onaylı</div>
    </div>

    <div class="card bg-base-200 border border-base-content/10 shadow-lg p-5">
      <div class="text-xs font-mono text-base-content/50 uppercase tracking-wider">Aktif & Tamamlanan</div>
      <div class="text-2xl font-black font-mono text-success mt-2">
        {onaylananSiparisler.length} Adet
      </div>
      <div class="text-[11px] text-base-content/50 mt-1">OMS Durum Makinesi</div>
    </div>

    <div class="card bg-base-200 border border-base-content/10 shadow-lg p-5">
      <div class="text-xs font-mono text-base-content/50 uppercase tracking-wider">İptal Edilenler</div>
      <div class="text-2xl font-black font-mono text-warning mt-2">
        {telafiEdilenler} Adet
      </div>
      <div class="text-[11px] text-base-content/50 mt-1">Stok İadesi Tamamlandı</div>
    </div>

    <div class="card bg-base-200 border border-base-content/10 shadow-lg p-5">
      <div class="text-xs font-mono text-base-content/50 uppercase tracking-wider">Yetki & ABAC Motoru</div>
      <div class="text-2xl font-black font-mono text-info mt-2">
        Rust Cedar
      </div>
      <div class="text-[11px] text-base-content/50 mt-1">Sub-ms Karar Süresi</div>
    </div>
  </div>

  <!-- Hızlı İşlem Kartları -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <div class="card bg-base-200 border border-base-content/10 shadow-xl hover:border-primary/40 transition-all">
      <div class="card-body p-6">
        <h3 class="card-title text-base font-bold">Ürün Kataloğu & PIM</h3>
        <p class="text-xs text-base-content/70">
          Katalogdaki ürünleri listeleyin, sepete ekleyin ve fiyat güncellemelerini test edin.
        </p>
        <div class="card-actions justify-end mt-4">
          <button class="btn btn-primary btn-sm font-bold" onclick={() => sekmeDegistir('katalog')}>
            Kataloğa Git ➔
          </button>
        </div>
      </div>
    </div>

    <div class="card bg-base-200 border border-base-content/10 shadow-xl hover:border-primary/40 transition-all">
      <div class="card-body p-6">
        <h3 class="card-title text-base font-bold">Siparişler & OMS</h3>
        <p class="text-xs text-base-content/70">
          Siparişlerin durum makinesi akışını, kargo takip barkodlarını ve RMA iade süreçlerini yönetin.
        </p>
        <div class="card-actions justify-end mt-4">
          <button class="btn btn-outline btn-sm font-bold" onclick={() => sekmeDegistir('siparisler')}>
            Siparişleri İncele ➔
          </button>
        </div>
      </div>
    </div>

    <div class="card bg-base-200 border border-base-content/10 shadow-xl hover:border-primary/40 transition-all">
      <div class="card-body p-6">
        <h3 class="card-title text-base font-bold">Saga & Olay Akışı</h3>
        <p class="text-xs text-base-content/70">
          5 adımlı OdemeSaga telafi mekanizmalarını ve NATS CloudEvents akışını inceleyin.
        </p>
        <div class="card-actions justify-end mt-4">
          <button class="btn btn-outline btn-sm font-bold" onclick={() => sekmeDegistir('telemetri')}>
            Telemetriye Git ➔
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
