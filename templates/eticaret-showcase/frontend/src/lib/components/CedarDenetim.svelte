<script lang="ts">
  import { cedarDenetimGunlugu, aktifKullanici } from '../hazneler/kimlikHaznesi';

  const CEDAR_POLITIKA_METNI = `// Cedar ABAC E-Ticaret ve Katalog Politikaları

// 1. Kural: Sistem Yöneticisi (Admin) tüm eylemleri gerçekleştirebilir
permit (
    principal in Role::"Admin",
    action,
    resource
);

// 2. Kural: Fiyat Yöneticisi katalog fiyatlarını güncelleyebilir
permit (
    principal in Role::"PricingOfficer",
    action in [Action::"UpdatePrice", Action::"FiyatGuncelle"],
    resource
);

// 3. Kural: Satınalma Uzmanı yalnız $50.000 altındaki siparişleri verebilir
permit (
    principal in Role::"Purchaser",
    action in [Action::"Checkout", Action::"Ode", Action::"PlaceOrder"],
    resource
)
when {
    resource.amount <= 50000
};

// 4. Kural: Pazarlama Yetkilisi kupon oluşturabilir ve %20 üzeri indirim uygulayabilir
permit (
    principal in Role::"MarketingLead",
    action in [Action::"ApplyCoupon", Action::"KuponUygula"],
    resource
);
`;
</script>

<div class="space-y-6">
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-base-200 p-5 rounded-2xl border border-base-content/10">
    <div>
      <h3 class="font-bold text-lg text-base-content">Cedar ABAC Yetki Matrisi & Canlı Karar Günlüğü</h3>
      <p class="text-xs text-base-content/60 font-mono mt-0.5">
        Rust Cedar Motoru • Statik Rol ve Özellik Tabanlı (ABAC) Denetim
      </p>
    </div>
    <span class="badge badge-sm badge-primary badge-outline font-mono">eticaret-politikalar.cedar</span>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
    <!-- Sol: Yetki Matrisi ve Politika Dosyası -->
    <div class="lg:col-span-6 space-y-6">
      <div class="card bg-base-200 border border-base-content/10 shadow-xl">
        <div class="card-body p-6 space-y-4">
          <h4 class="font-bold text-sm text-base-content flex items-center gap-2">
            <span>🛡️</span> E-Ticaret Rol Yetki Matrisi
          </h4>
          <div class="overflow-x-auto">
            <table class="table table-sm w-full text-xs font-mono">
              <thead class="bg-base-300 text-base-content/70">
                <tr>
                  <th>Rol</th>
                  <th>Checkout Limiti</th>
                  <th>Fiyat Güncelleme</th>
                  <th>Kupon (%20+)</th>
                </tr>
              </thead>
              <tbody>
                <tr class="{$aktifKullanici?.rol === 'Admin' ? 'bg-primary/10 font-bold' : ''}">
                  <td><span class="badge badge-xs badge-primary">Admin</span></td>
                  <td class="text-success">Sınırsız</td>
                  <td class="text-success">İzinli</td>
                  <td class="text-success">İzinli</td>
                </tr>
                <tr class="{$aktifKullanici?.rol === 'Purchaser' ? 'bg-secondary/10 font-bold' : ''}">
                  <td><span class="badge badge-xs badge-secondary">Purchaser</span></td>
                  <td class="text-warning">Maks. $50,000</td>
                  <td class="text-error">Kısıtlı</td>
                  <td class="text-error">Kısıtlı</td>
                </tr>
                <tr class="{$aktifKullanici?.rol === 'PricingOfficer' ? 'bg-info/10 font-bold' : ''}">
                  <td><span class="badge badge-xs badge-info">PricingOfficer</span></td>
                  <td class="text-error">Kısıtlı</td>
                  <td class="text-success">İzinli</td>
                  <td class="text-error">Kısıtlı</td>
                </tr>
                <tr class="{$aktifKullanici?.rol === 'MarketingLead' ? 'bg-warning/10 font-bold' : ''}">
                  <td><span class="badge badge-xs badge-warning">MarketingLead</span></td>
                  <td class="text-error">Kısıtlı</td>
                  <td class="text-error">Kısıtlı</td>
                  <td class="text-success">İzinli</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <div class="card bg-base-200 border border-base-content/10 shadow-xl">
        <div class="card-body p-6 space-y-2">
          <h4 class="font-bold text-sm text-base-content/90 font-mono">eticaret-politikalar.cedar</h4>
          <pre class="bg-base-300 p-4 rounded-xl font-mono text-xs text-emerald-400 overflow-x-auto border border-base-content/5 leading-relaxed"><code>{CEDAR_POLITIKA_METNI}</code></pre>
        </div>
      </div>
    </div>

    <!-- Sağ: Anlık Karar Günlüğü -->
    <div class="lg:col-span-6">
      <div class="card bg-base-200 border border-base-content/10 shadow-xl h-full">
        <div class="card-body p-6 flex flex-col">
          <div class="flex items-center justify-between border-b border-base-content/10 pb-3 mb-3">
            <h4 class="font-bold text-sm text-base-content flex items-center gap-2">
              <span>🚦</span> Canlı Cedar Karar Denetim Günlüğü
            </h4>
            <span class="badge badge-sm badge-neutral font-mono">{$cedarDenetimGunlugu.length} Denetim</span>
          </div>

          {#if $cedarDenetimGunlugu.length === 0}
            <div class="flex-1 flex flex-col items-center justify-center text-center p-8 text-base-content/50 space-y-2 font-mono text-xs">
              <div class="text-3xl">🛡️</div>
              <p class="font-bold">Henüz bir Cedar yetki denetimi gerçekleşmedi.</p>
              <p class="text-[11px]">Kupon uygulayarak, fiyat güncelleyerek veya sipariş vererek yetki motorunu test edin.</p>
            </div>
          {:else}
            <div class="space-y-3 overflow-y-auto max-h-[500px] pr-1">
              {#each $cedarDenetimGunlugu as k}
                <div class="p-3.5 rounded-xl border font-mono text-xs space-y-1.5 {k.karar === 'ALLOW' ? 'bg-emerald-950/20 border-emerald-500/30' : 'bg-rose-950/20 border-rose-500/30'}">
                  <div class="flex items-center justify-between">
                    <span class="badge badge-sm font-bold {k.karar === 'ALLOW' ? 'badge-success text-black' : 'badge-error text-white'}">
                      {k.karar}
                    </span>
                    <span class="text-base-content/40 text-[11px]">{k.zaman} (⏱️ {k.gecikmeMs}ms)</span>
                  </div>
                  <div class="text-[11px] text-base-content/70">
                    <div><span class="text-base-content/40">Principal:</span> <span class="font-bold text-primary">{k.principal}</span></div>
                    <div><span class="text-base-content/40">Action:</span> <span class="text-secondary">{k.action}</span></div>
                  </div>
                  <p class="text-xs font-semibold mt-1 {k.karar === 'ALLOW' ? 'text-emerald-300' : 'text-rose-300'}">
                    {k.gerekce}
                  </p>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
