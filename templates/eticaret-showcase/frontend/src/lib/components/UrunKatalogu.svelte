<script lang="ts">
  import { aktifKullanici, cedarDenetimGunlugu } from '../hazneler/kimlikHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import { sepet, sepetId } from '../hazneler/sepetHaznesi';
  import type { UrunKalemi } from '../types';

  interface Props {
    urunler: UrunKalemi[];
    onFiyatGuncelle: (sku: string, yeniFiyat: number) => Promise<void>;
  }

  let { urunler = [], onFiyatGuncelle }: Props = $props();

  let aramaMetni = $state('');
  let secilenKategori = $state('Tümü');
  let secilenUrunFiyatGuncelle = $state<UrunKalemi | null>(null);
  let yeniFiyatGirdisi = $state(0);

  let filtrelenmisUrunler = $derived(
    urunler.filter((u) => {
      const metinEslesti = !aramaMetni || u.baslik.toLowerCase().includes(aramaMetni.toLowerCase()) || u.sku.toLowerCase().includes(aramaMetni.toLowerCase());
      const kategoriEslesti = secilenKategori === 'Tümü' || u.kategoriYolu.startsWith(secilenKategori);
      return metinEslesti && kategoriEslesti;
    })
  );

  async function sepeteEkle(urun: UrunKalemi) {
    if (urun.musaitStok <= 0) {
      bildirimEkle('hata', 'Stok Yetersiz', `${urun.baslik} stokta tükenmiştir.`);
      return;
    }

    try {
      const res = await fetch('http://localhost:5000/api/eticaret/sepet/ekle', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-tenant-id': 'tenant-acme-corp',
          'x-user-id': $aktifKullanici.id,
        },
        body: JSON.stringify({
          sepetId: $sepetId,
          sku: urun.sku,
          miktar: 1,
          birimFiyat: urun.birimFiyat,
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

        bildirimEkle('basari', 'Sepete Eklendi', `1x ${urun.baslik} sepete eklendi.`);
      } else {
        const err = await res.json();
        bildirimEkle('hata', 'Sepete Eklenemedi', err.message || 'Hata oluştu.');
      }
    } catch (e: any) {
      bildirimEkle('hata', 'Bağlantı Hatası', e?.message || 'Servise ulaşılamadı.');
    }
  }

  function fiyatGuncelleModalAc(urun: UrunKalemi) {
    // Cedar ABAC Ön Kontrolü
    if ($aktifKullanici.rol !== 'Admin' && $aktifKullanici.rol !== 'PricingOfficer') {
      bildirimEkle('hata', 'Cedar Yetki Reddi (FORBID)', `Role::"${$aktifKullanici.rol}" ürün fiyatı değiştirme yetkisine sahip değildir. Yalnızca PricingOfficer veya Admin fiyat güncelleyebilir.`);
      kaydetCedarDenetimi('FiyatGuncelle', urun.birimFiyat, 'DENY', `FORBID: Role::"${$aktifKullanici.rol}" fiyat güncelleyemez.`);
      return;
    }

    kaydetCedarDenetimi('FiyatGuncelle', urun.birimFiyat, 'ALLOW', `Cedar PERMIT: ${$aktifKullanici.ad} (${$aktifKullanici.rol}) için fiyat güncelleme onaylandı.`);
    secilenUrunFiyatGuncelle = urun;
    yeniFiyatGirdisi = urun.birimFiyat;
  }

  async function fiyatGuncellemeyiKaydet() {
    if (!secilenUrunFiyatGuncelle || yeniFiyatGirdisi <= 0) return;

    await onFiyatGuncelle(secilenUrunFiyatGuncelle.sku, yeniFiyatGirdisi);
    secilenUrunFiyatGuncelle = null;
  }

  function kaydetCedarDenetimi(action: string, tutar: number, karar: 'ALLOW' | 'DENY', gerekce: string) {
    cedarDenetimGunlugu.update((l) => [
      {
        id: crypto.randomUUID(),
        zaman: new Date().toLocaleTimeString(),
        principal: `User::"${$aktifKullanici.id}" (Role::"${$aktifKullanici.rol}")`,
        action: `Action::"${action}"`,
        resource: `Product::"item" { amount: $${tutar} }`,
        karar,
        gerekce,
        gecikmeMs: Math.round((Math.random() * 0.2 + 0.1) * 100) / 100,
        politika: karar === 'ALLOW' ? 'permit(principal in Role::"PricingOfficer", action, resource);' : 'Explicit Denial / No permit matched.',
      },
      ...l.slice(0, 19),
    ]);
  }
</script>

<div class="space-y-6">
  <!-- Filtreleme & Arama Çubuğu -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-base-200 p-5 rounded-2xl border border-base-content/10">
    <div class="flex-1 flex flex-col sm:flex-row items-stretch sm:items-center gap-3 w-full sm:w-auto">
      <input
        type="text"
        placeholder="Ürün adı veya SKU ile ara..."
        class="input input-bordered input-sm bg-base-300 font-mono text-xs flex-1 max-w-sm"
        bind:value={aramaMetni}
      />
      <div class="flex items-center gap-2">
        <span class="text-xs text-base-content/60 font-semibold">Kategori:</span>
        <select class="select select-bordered select-sm bg-base-300 text-xs font-mono" bind:value={secilenKategori}>
          <option value="Tümü">Tüm Kategoriler</option>
          <option value="Donanım">Donanım / IoT</option>
          <option value="Sensör">Sensör / Optik</option>
          <option value="Otomasyon">Otomasyon / Robotik</option>
        </select>
      </div>
    </div>
    <span class="badge badge-sm badge-neutral font-mono">{filtrelenmisUrunler.length} Ürün Listelendi</span>
  </div>

  <!-- Ürün Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    {#each filtrelenmisUrunler as u}
      <div class="card bg-base-200 border border-base-content/10 shadow-xl hover:border-primary/40 transition-all flex flex-col justify-between">
        <div class="card-body p-6 space-y-4">
          <div class="flex items-center justify-between">
            <span class="badge badge-sm badge-outline font-mono">{u.sku}</span>
            <span class="badge badge-sm badge-ghost text-xs">{u.kategoriYolu}</span>
          </div>

          <div>
            <h4 class="card-title text-base font-bold text-base-content leading-snug">{u.baslik}</h4>
            <p class="text-xs text-base-content/60 mt-1 line-clamp-2">{u.aciklama || ''}</p>
            <div class="text-2xl font-black font-mono text-primary mt-2">
              ${u.birimFiyat.toLocaleString()}
              <span class="text-xs text-base-content/50 font-normal">/ birim</span>
            </div>
          </div>

          <!-- Müsait Stok & Varyantlar -->
          <div class="bg-base-300 p-3.5 rounded-xl space-y-1.5 font-mono text-xs border border-base-content/5">
            <div class="flex justify-between">
              <span class="text-base-content/60">Müsait Stok:</span>
              <span class="font-bold {u.musaitStok > 0 ? 'text-success' : 'text-error'}">
                {u.musaitStok} Adet
              </span>
            </div>
            {#if u.varyantlar && u.varyantlar.length > 0}
              <div class="flex justify-between text-[11px]">
                <span class="text-base-content/60">Varyant:</span>
                <span class="text-secondary">{u.varyantlar[0].ad}: {u.varyantlar[0].deger}</span>
              </div>
            {/if}
          </div>

          <!-- Butonlar -->
          <div class="flex items-center gap-2 pt-2 border-t border-base-content/5">
            <button
              class="btn btn-primary btn-sm flex-1 font-bold shadow-md shadow-primary/20"
              onclick={() => sepeteEkle(u)}
              disabled={u.musaitStok <= 0}
            >
              <span>🛒</span>
              {u.musaitStok > 0 ? 'Sepete Ekle' : 'Tükendi'}
            </button>

            <button
              class="btn btn-square btn-sm btn-ghost border border-base-content/10 tooltip"
              data-tip="Fiyatı Güncelle (Cedar Korumalı)"
              onclick={() => fiyatGuncelleModalAc(u)}
            >
              ✏️
            </button>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Fiyat Güncelle Modal -->
  {#if secilenUrunFiyatGuncelle}
    <div class="modal modal-open">
      <div class="modal-box bg-base-200 border border-base-content/10 space-y-4">
        <h3 class="font-bold text-base text-base-content">
          Birim Fiyat Güncelle ({secilenUrunFiyatGuncelle.sku})
        </h3>
        <p class="text-xs text-base-content/60">
          {secilenUrunFiyatGuncelle.baslik} için yeni birim satış fiyatını belirleyin.
        </p>

        <div class="form-control">
          <label class="label py-1" for="new-price-input">
            <span class="label-text text-xs font-semibold">Yeni Birim Fiyat ($):</span>
          </label>
          <input
            id="new-price-input"
            type="number"
            min="1"
            bind:value={yeniFiyatGirdisi}
            class="input input-bordered input-sm bg-base-300 font-mono text-sm"
          />
        </div>

        <div class="modal-action">
          <button class="btn btn-sm btn-ghost" onclick={() => (secilenUrunFiyatGuncelle = null)}>
            İptal
          </button>
          <button class="btn btn-sm btn-primary font-bold" onclick={fiyatGuncellemeyiKaydet}>
            Fiyatı Güncelle (Cedar)
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
