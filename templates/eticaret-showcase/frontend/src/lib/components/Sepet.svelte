<script lang="ts">
  import { sepet, sepetId } from '../hazneler/sepetHaznesi';
  import { aktifKullanici, cedarDenetimGunlugu } from '../hazneler/kimlikHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';

  interface Props {
    onCheckoutBaslat: (odemeHatasiniSimuleEt: boolean) => Promise<void>;
    sekmeDegistir: (s: string) => void;
    isleniyor?: boolean;
  }

  let { onCheckoutBaslat, sekmeDegistir, isleniyor = false }: Props = $props();

  let girilenKuponKodu = $state('');
  let odemeHatasiniSimuleEt = $state(false);

  async function sepettenCikar(sku: string) {
    try {
      const res = await fetch('http://localhost:5000/api/eticaret/sepet/cikar', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-tenant-id': 'tenant-acme-corp',
        },
        body: JSON.stringify({
          sepetId: $sepetId,
          sku,
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
        bildirimEkle('bilgi', 'Sepet Güncellendi', `${sku} sepetten çıkarıldı.`);
      }
    } catch (e: any) {
      bildirimEkle('hata', 'Hata', e?.message || 'Ürün çıkarılamadı.');
    }
  }

  async function kuponUygula() {
    if (!girilenKuponKodu.trim()) return;

    // Cedar ABAC Kontrolü: %20 üzeri indirimli kurumsal kuponları yalnızca MarketingLead veya Admin uygulayabilir
    if (girilenKuponKodu.toUpperCase() === 'ENTERPRISE30' && $aktifKullanici.rol !== 'Admin' && $aktifKullanici.rol !== 'MarketingLead') {
      bildirimEkle('uyari', 'Cedar Yetki Sınırı (FORBID)', 'ENTERPRISE30 (%30) kurumsal kuponu yalnızca MarketingLead veya Admin tarafından uygulanabilir.');
      kaydetCedarDenetimi('KuponUygula', 30, 'DENY', 'FORBID: Role::"' + $aktifKullanici.rol + '" yüksek indirim kuponu uygulayamaz.');
      return;
    }

    try {
      const res = await fetch('http://localhost:5000/api/eticaret/sepet/kupon', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-tenant-id': 'tenant-acme-corp',
        },
        body: JSON.stringify({
          sepetId: $sepetId,
          kuponKodu: girilenKuponKodu,
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

        kaydetCedarDenetimi('KuponUygula', data.indirimTutari, 'ALLOW', `Cedar PERMIT: ${girilenKuponKodu} başarıyla uygulandı.`);
        bildirimEkle('basari', 'Kupon Uygulandı', `${girilenKuponKodu.toUpperCase()} kuponu ile $${data.indirimTutari.toLocaleString()} indirim sağlandı!`);
        girilenKuponKodu = '';
      } else {
        const err = await res.json();
        bildirimEkle('hata', 'Geçersiz Kupon', err.message || 'Kupon uygulanamadı.');
      }
    } catch (e: any) {
      bildirimEkle('hata', 'Bağlantı Hatası', e?.message || 'Servise ulaşılamadı.');
    }
  }

  async function checkoutBaslat() {
    if (!$sepet || $sepet.kalemler.length === 0) {
      bildirimEkle('uyari', 'Sepet Boş', 'Lütfen önce ürün kataloğundan sepete ürün ekleyin.');
      return;
    }

    // Cedar ABAC Kontrolü: Rol ve Sipariş Tutarı Sınırı
    if ($aktifKullanici.rol === 'WarehouseManager') {
      bildirimEkle('hata', 'Cedar Yetki Reddi (FORBID)', 'Depo Yönetimi rolü (WarehouseManager) checkout ve satınalma işlemi yapamaz.');
      kaydetCedarDenetimi('Checkout', $sepet.genelToplam, 'DENY', 'FORBID: WarehouseManager sipariş veremez.');
      return;
    }

    if ($aktifKullanici.rol === 'Purchaser' && $sepet.genelToplam > $aktifKullanici.siparisLimiti) {
      bildirimEkle('uyari', 'Cedar Limit Aşımı (FORBID)', `Sepet tutarı ($${$sepet.genelToplam.toLocaleString()}), Satınalma Uzmanı için belirlenen $${$aktifKullanici.siparisLimiti.toLocaleString()} limitini aşıyor!`);
      kaydetCedarDenetimi('Checkout', $sepet.genelToplam, 'DENY', `FORBID: Tutar ($${$sepet.genelToplam}) > Limit ($${$aktifKullanici.siparisLimiti})`);
      return;
    }

    kaydetCedarDenetimi('Checkout', $sepet.genelToplam, 'ALLOW', `Cedar PERMIT: ${$aktifKullanici.ad} için 5 adımlı OdemeSaga checkout izni verildi.`);
    await onCheckoutBaslat(odemeHatasiniSimuleEt);
  }

  function kaydetCedarDenetimi(action: string, tutar: number, karar: 'ALLOW' | 'DENY', gerekce: string) {
    cedarDenetimGunlugu.update((l) => [
      {
        id: crypto.randomUUID(),
        zaman: new Date().toLocaleTimeString(),
        principal: `User::"${$aktifKullanici.id}" (Role::"${$aktifKullanici.rol}")`,
        action: `Action::"${action}"`,
        resource: `Cart::"${$sepetId}" { amount: $${tutar} }`,
        karar,
        gerekce,
        gecikmeMs: Math.round((Math.random() * 0.2 + 0.1) * 100) / 100,
        politika: karar === 'ALLOW' ? 'permit(principal in Role::"Purchaser", action, resource) when { resource.amount <= 50000 };' : 'Explicit Denial / Limit Exceeded.',
      },
      ...l.slice(0, 19),
    ]);
  }
</script>

<div class="space-y-6">
  <!-- Üst Başlık -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-base-200 p-5 rounded-2xl border border-base-content/10">
    <div>
      <h3 class="font-bold text-lg text-base-content">Alışveriş Sepeti & Checkout</h3>
      <p class="text-xs text-base-content/60 font-mono mt-0.5">
        Sepet ID: <strong class="text-primary">{$sepetId}</strong> • Dinamik Fiyatlandırma & Kupon Motoru
      </p>
    </div>

    <!-- Hata Simülatörü Toggle -->
    <div class="bg-base-300 px-4 py-2.5 rounded-xl border border-base-content/5">
      <label class="label cursor-pointer gap-3 p-0" for="payment-error-toggle">
        <span class="label-text text-xs font-semibold text-warning flex items-center gap-1.5">
          Ödeme Hatası Simüle Et (Compensating Action Testi)
        </span>
        <input
          id="payment-error-toggle"
          type="checkbox"
          bind:checked={odemeHatasiniSimuleEt}
          class="toggle toggle-warning toggle-sm"
        />
      </label>
    </div>
  </div>

  {#if !$sepet || $sepet.kalemler.length === 0}
    <div class="card bg-base-200 border border-base-content/10 shadow-xl p-12 text-center text-base-content/50 space-y-3">
      <div class="text-4xl">🛒</div>
      <p class="font-bold text-base text-base-content">Sepetiniz şu anda boş.</p>
      <p class="text-xs max-w-md mx-auto">Ürün Kataloğu sekmesine giderek IoT Ağ Geçidi veya Optik Sensör ekleyebilirsiniz.</p>
      <button class="btn btn-primary btn-sm font-bold" onclick={() => sekmeDegistir('katalog')}>
        Ürün Kataloğunu Aç ➔
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
      <!-- Sol: Sepet Kalemleri Listesi -->
      <div class="lg:col-span-8 space-y-4">
        <div class="card bg-base-200 border border-base-content/10 shadow-xl overflow-hidden">
          <div class="overflow-x-auto">
            <table class="table w-full text-xs">
              <thead class="bg-base-300 text-base-content/70 font-mono text-[11px] uppercase">
                <tr>
                  <th>Ürün Bilgisi</th>
                  <th>Birim Fiyat</th>
                  <th>Miktar</th>
                  <th>Toplam</th>
                  <th class="text-right">İşlem</th>
                </tr>
              </thead>
              <tbody class="font-mono">
                {#each $sepet.kalemler as k}
                  <tr class="hover:bg-base-300/40">
                    <td>
                      <div class="font-bold text-base-content">{k.baslik}</div>
                      <div class="text-[10px] text-base-content/50">{k.sku}</div>
                    </td>
                    <td>${k.birimFiyat.toLocaleString()}</td>
                    <td><span class="badge badge-sm badge-neutral">{k.miktar} Adet</span></td>
                    <td class="font-bold text-primary">${k.toplamTutar.toLocaleString()}</td>
                    <td class="text-right">
                      <button
                        class="btn btn-ghost btn-xs text-error font-bold"
                        onclick={() => sepettenCikar(k.sku)}
                      >
                        ✕ Kaldır
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- Sağ: Kupon ve Özet Kartı -->
      <div class="lg:col-span-4 space-y-4">
        <div class="card bg-base-200 border border-base-content/10 shadow-xl">
          <div class="card-body p-6 space-y-4">
            <h4 class="font-bold text-sm text-base-content border-b border-base-content/10 pb-2">
              Sipariş Özeti
            </h4>

            <!-- Kupon Giriş Formu -->
            <div class="space-y-2">
              <label class="text-xs font-semibold text-base-content/70" for="coupon-code-input">Promosyon / İndirim Kuponu:</label>
              <div class="flex gap-2">
                <input
                  id="coupon-code-input"
                  type="text"
                  placeholder="Örn: CLRINF10, VIP20"
                  class="input input-bordered input-sm bg-base-300 font-mono text-xs flex-1 uppercase"
                  bind:value={girilenKuponKodu}
                />
                <button class="btn btn-sm btn-outline font-bold" onclick={kuponUygula}>
                  Uygula
                </button>
              </div>
              <div class="text-[10px] text-base-content/40 font-mono">
                Tanımlı: CLRINF10 (%10), VIP20 (%20), ENTERPRISE30 (%30 - Cedar)
              </div>
            </div>

            <!-- Fiyat Dökümü -->
            <div class="bg-base-300 p-3.5 rounded-xl space-y-2 font-mono text-xs border border-base-content/5">
              <div class="flex justify-between text-base-content/70">
                <span>Ara Toplam:</span>
                <span>${$sepet.araToplam.toLocaleString()}</span>
              </div>
              {#if $sepet.indirimTutari > 0}
                <div class="flex justify-between text-emerald-400 font-semibold">
                  <span>İndirim ({$sepet.kuponKodu}):</span>
                  <span>-${$sepet.indirimTutari.toLocaleString()}</span>
                </div>
              {/if}
              <div class="divider my-1"></div>
              <div class="flex justify-between text-sm font-black text-base-content">
                <span>Genel Toplam:</span>
                <span class="text-primary text-base">${$sepet.genelToplam.toLocaleString()}</span>
              </div>
            </div>

            <!-- Checkout Butonu -->
            <button
              class="btn btn-primary w-full font-bold shadow-lg shadow-primary/20"
              onclick={checkoutBaslat}
              disabled={isleniyor}
            >
              {#if isleniyor}
                <span class="loading loading-spinner loading-xs"></span>
                5 Adımlı Saga Yürütülüyor...
              {:else}
                <span>⚡</span>
                Siparişi Tamamla (OdemeSaga)
              {/if}
            </button>

            <div class="text-center">
              <span class="text-[10px] text-base-content/40 font-mono">
                NO-DISTRIBUTED-TX • BASE • Telafi Edici İşlem Güvenceli
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
