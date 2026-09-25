<script lang="ts">
  import { aktifKullanici, cedarDenetimGunlugu } from '../hazneler/kimlikHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import { sepet, sepetId } from '../hazneler/sepetHaznesi';
  import type { KampanyaVerisi } from '../types';

  interface Props {
    kampanyalar: KampanyaVerisi[];
    onKampanyaOlustur: (yeni: any) => Promise<void>;
    sekmeDegistir: (s: string) => void;
  }

  let { kampanyalar = [], onKampanyaOlustur, sekmeDegistir }: Props = $props();

  let yeniModalAcik = $state(false);
  let yeniKod = $state('');
  let yeniBaslik = $state('');
  let yeniAciklama = $state('');
  let yeniTur: 'Yuzdesel' | 'SabitTutar' | 'KategoriIndirimi' = $state('Yuzdesel');
  let yeniDeger = $state(0.15);
  let yeniHedefKategori = $state('');
  let yeniMinSepet = $state(500);
  let yeniMaksKota = $state(100);

  async function sepeteUygula(kampanya: KampanyaVerisi) {
    if (!$sepet || $sepet.kalemler.length === 0) {
      bildirimEkle('uyari', 'Sepet Boş', 'Lütfen önce ürün kataloğundan sepete ürün ekleyin.');
      sekmeDegistir('katalog');
      return;
    }

    if ($sepet.genelToplam < kampanya.minimumSepetTutari) {
      bildirimEkle(
        'uyari',
        'Minimum Sepet Tutarı Yetersiz',
        `Bu kampanya en az $${kampanya.minimumSepetTutari.toLocaleString()} tutarındaki sepetlerde geçerlidir (Mevcut: $${$sepet.genelToplam.toLocaleString()}).`
      );
      return;
    }

    try {
      const res = await fetch('http://localhost:5000/api/eticaret/sepet/kupon', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-tenant-id': 'tenant-acme-corp',
          'x-user-id': $aktifKullanici.id,
        },
        body: JSON.stringify({
          sepetId: $sepetId,
          kuponKodu: kampanya.kod,
        }),
      });

      if (res.ok) {
        const data = await res.json();
        sepet.set({
          sepetId: data.sepetId,
          kullaniciId: data.kullaniciId,
          kalemler: data.kalemler,
          kuponKodu: data.kuponKodu,
          araToplam: data.araToplam,
          indirimTutari: data.indirimTutari,
          genelToplam: data.genelToplam,
          kilitli: false,
        });

        bildirimEkle('basari', 'Kampanya Uygulandı', `${kampanya.kod} kampanyası ile $${data.indirimTutari.toLocaleString()} indirim uygulandı!`);
        sekmeDegistir('sepet');
      } else {
        const err = await res.json();
        bildirimEkle('hata', 'Kampanya Uygulanamadı', err.message || 'Hata oluştu.');
      }
    } catch (e: any) {
      bildirimEkle('hata', 'Bağlantı Hatası', e?.message || 'Servis hatası.');
    }
  }

  function kampanyaOlusturModalAc() {
    // Cedar ABAC Ön Kontrolü
    if ($aktifKullanici.rol !== 'Admin' && $aktifKullanici.rol !== 'MarketingLead') {
      bildirimEkle('hata', 'Cedar Yetki Reddi (FORBID)', `Role::"${$aktifKullanici.rol}" kampanya tanımlama yetkisine sahip değildir. Yalnızca MarketingLead veya Admin kampanya oluşturabilir.`);
      kaydetCedarDenetimi('KampanyaOlustur', 0, 'DENY', `FORBID: Role::"${$aktifKullanici.rol}" kampanya oluşturamaz.`);
      return;
    }

    kaydetCedarDenetimi('KampanyaOlustur', 0, 'ALLOW', `Cedar PERMIT: ${$aktifKullanici.ad} (${$aktifKullanici.rol}) kampanya yönetimi onaylandı.`);
    yeniModalAcik = true;
  }

  async function yeniKampanyayiKaydet() {
    if (!yeniKod.trim() || !yeniBaslik.trim() || yeniDeger <= 0) return;

    await onKampanyaOlustur({
      kod: yeniKod.toUpperCase().trim(),
      baslik: yeniBaslik.trim(),
      aciklama: yeniAciklama.trim(),
      tur: yeniTur,
      deger: yeniDeger,
      hedefKategori: yeniHedefKategori.trim() || null,
      minimumSepetTutari: yeniMinSepet,
      maksimumKullanimSayisi: yeniMaksKota > 0 ? yeniMaksKota : null,
    });

    yeniModalAcik = false;
    yeniKod = '';
    yeniBaslik = '';
    yeniAciklama = '';
  }

  function kaydetCedarDenetimi(action: string, tutar: number, karar: 'ALLOW' | 'DENY', gerekce: string) {
    cedarDenetimGunlugu.update((l) => [
      {
        id: crypto.randomUUID(),
        zaman: new Date().toLocaleTimeString(),
        principal: `User::"${$aktifKullanici.id}" (Role::"${$aktifKullanici.rol}")`,
        action: `Action::"${action}"`,
        resource: `Campaign::"promo" { discount: ${tutar} }`,
        karar,
        gerekce,
        gecikmeMs: Math.round((Math.random() * 0.2 + 0.1) * 100) / 100,
        politika: karar === 'ALLOW' ? 'permit(principal in Role::"MarketingLead", action, resource);' : 'Explicit Denial / Role restricted.',
      },
      ...l.slice(0, 19),
    ]);
  }
</script>

<div class="space-y-6">
  <!-- Üst Başlık & Eylem Çubuğu -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-base-200 p-5 rounded-2xl border border-base-content/10">
    <div>
      <h3 class="font-bold text-lg text-base-content">Promosyon ve Kampanya Yönetim Merkezi</h3>
      <p class="text-xs text-base-content/60 font-mono mt-0.5">
        Dinamik Kural Motoru • Yüzdesel, Sabit Tutar ve Kategori Bazlı İndirimler
      </p>
    </div>

    <button class="btn btn-sm btn-primary font-bold shadow-md shadow-primary/20" onclick={kampanyaOlusturModalAc}>
      <span>✨</span> Yeni Kampanya Tanımla (Cedar)
    </button>
  </div>

  <!-- Kampanyalar Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    {#each kampanyalar as k}
      <div class="card bg-base-200 border border-base-content/10 shadow-xl hover:border-primary/40 transition-all flex flex-col justify-between">
        <div class="card-body p-6 space-y-4">
          <div class="flex items-center justify-between">
            <span class="badge badge-sm badge-primary font-mono font-bold tracking-wider">{k.kod}</span>
            <span class="badge badge-sm font-mono {k.aktif ? 'badge-success text-black' : 'badge-error text-white'}">
              {k.aktif ? 'AKTİF' : 'PASİF'}
            </span>
          </div>

          <div>
            <h4 class="card-title text-base font-bold text-base-content">{k.baslik}</h4>
            <p class="text-xs text-base-content/60 mt-1 line-clamp-2">{k.aciklama || ''}</p>
          </div>

          <!-- İndirim Oranı & Koşullar -->
          <div class="bg-base-300 p-4 rounded-xl space-y-2 font-mono text-xs border border-base-content/5">
            <div class="flex justify-between items-center">
              <span class="text-base-content/60">İndirim Oranı/Tutar:</span>
              <span class="font-black text-primary text-sm">
                {k.tur === 'Yuzdesel' || k.tur === 'KategoriIndirimi' ? `%${(k.deger * 100).toFixed(0)} İndirim` : `$${k.deger.toLocaleString()} Sabit İndirim`}
              </span>
            </div>
            <div class="flex justify-between text-[11px]">
              <span class="text-base-content/60">Min. Sepet Eşiği:</span>
              <span class="font-bold text-base-content/90">${k.minimumSepetTutari.toLocaleString()}</span>
            </div>
            {#if k.hedefKategori}
              <div class="flex justify-between text-[11px]">
                <span class="text-base-content/60">Hedef Kategori:</span>
                <span class="text-secondary">{k.hedefKategori}</span>
              </div>
            {/if}
            {#if k.maksimumKullanimSayisi}
              <div class="flex justify-between text-[11px]">
                <span class="text-base-content/60">Kota Kullanımı:</span>
                <span class="text-accent">{k.toplamKullanimSayisi} / {k.maksimumKullanimSayisi}</span>
              </div>
            {/if}
          </div>

          <button
            class="btn btn-outline btn-primary btn-sm w-full font-bold shadow-sm"
            onclick={() => sepeteUygula(k)}
          >
            <span>🏷️</span> Sepete Uygula ➔
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Yeni Kampanya Modal -->
  {#if yeniModalAcik}
    <div class="modal modal-open">
      <div class="modal-box bg-base-200 border border-base-content/10 max-w-lg space-y-4">
        <h3 class="font-bold text-base text-base-content">Yeni Kampanya Tanımla (Cedar Korumalı)</h3>
        <p class="text-xs text-base-content/60">
          Pazarlama departmanı kurallarına uygun olarak yeni promosyon ve kota yapılandırın.
        </p>

        <div class="space-y-3 font-mono text-xs">
          <div class="grid grid-cols-2 gap-3">
            <div class="form-control">
              <label class="label py-0.5" for="campaign-code-input"><span class="label-text text-[11px]">Kampanya Kodu:</span></label>
              <input id="campaign-code-input" type="text" placeholder="Örn: BAHAR25" bind:value={yeniKod} class="input input-bordered input-sm bg-base-300 uppercase" />
            </div>
            <div class="form-control">
              <label class="label py-0.5" for="campaign-type-select"><span class="label-text text-[11px]">İndirim Türü:</span></label>
              <select id="campaign-type-select" bind:value={yeniTur} class="select select-bordered select-sm bg-base-300">
                <option value="Yuzdesel">Yüzdesel (%)</option>
                <option value="SabitTutar">Sabit Tutar ($)</option>
                <option value="KategoriIndirimi">Kategori Bazlı (%)</option>
              </select>
            </div>
          </div>

          <div class="form-control">
            <label class="label py-0.5" for="campaign-title-input"><span class="label-text text-[11px]">Kampanya Başlığı:</span></label>
            <input id="campaign-title-input" type="text" placeholder="Örn: Bahar İndirimi" bind:value={yeniBaslik} class="input input-bordered input-sm bg-base-300" />
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div class="form-control">
              <label class="label py-0.5" for="discount-value-input"><span class="label-text text-[11px]">Değer ({yeniTur === 'SabitTutar' ? '$' : '0.15 = %15'}):</span></label>
              <input id="discount-value-input" type="number" step="0.01" min="0" bind:value={yeniDeger} class="input input-bordered input-sm bg-base-300" />
            </div>
            <div class="form-control">
              <label class="label py-0.5" for="min-cart-input"><span class="label-text text-[11px]">Min. Sepet ($):</span></label>
              <input id="min-cart-input" type="number" min="0" bind:value={yeniMinSepet} class="input input-bordered input-sm bg-base-300" />
            </div>
            <div class="form-control">
              <label class="label py-0.5" for="max-quota-input"><span class="label-text text-[11px]">Maks Kota:</span></label>
              <input id="max-quota-input" type="number" min="1" bind:value={yeniMaksKota} class="input input-bordered input-sm bg-base-300" />
            </div>
          </div>
        </div>

        <div class="modal-action">
          <button class="btn btn-sm btn-ghost" onclick={() => (yeniModalAcik = false)}>İptal</button>
          <button class="btn btn-sm btn-primary font-bold" onclick={yeniKampanyayiKaydet}>
            Kampanyayı Yayınla (Cedar)
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
