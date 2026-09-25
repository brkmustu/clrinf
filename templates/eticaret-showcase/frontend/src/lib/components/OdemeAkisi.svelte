<script lang="ts">
  import type { SiparisKaydi } from '../types';

  interface Props {
    secilenSiparis?: SiparisKaydi | null;
    isleniyor?: boolean;
  }

  let { secilenSiparis = null, isleniyor = false }: Props = $props();
</script>

<div class="card bg-base-200 border border-base-content/10 shadow-2xl p-6 sm:p-8 space-y-6">
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-base-content/10 pb-4">
    <div class="space-y-1">
      <div class="flex items-center gap-2">
        <span class="badge badge-primary font-bold text-xs font-mono">5-Step Checkout Saga</span>
        <span class="badge badge-outline badge-info font-mono text-xs">C# CQRS • NATS JetStream • BASE</span>
      </div>
      <h3 class="text-lg font-black text-base-content">
        Dağıtık E-Ticaret Checkout Akışı (OdemeSaga)
      </h3>
      <p class="text-xs text-base-content/60 font-mono">
        RULE NO-DISTRIBUTED-TX: 2PC yerine adım adım yerel commit ve telafi edici işlem (Compensating Action).
      </p>
    </div>

    {#if secilenSiparis}
      <div class="flex items-center gap-2 font-mono text-xs">
        <span class="badge badge-neutral">{secilenSiparis.siparisId}</span>
        <span class="badge font-bold {secilenSiparis.durum === 'Onaylandi' ? 'badge-success text-black' : secilenSiparis.durum === 'IptalEdildi' ? 'badge-warning' : 'badge-primary text-white'}">
          {secilenSiparis.durum}
        </span>
      </div>
    {/if}
  </div>

  {#if isleniyor}
    <div class="flex flex-col items-center justify-center py-12 space-y-4">
      <span class="loading loading-ring loading-lg text-primary"></span>
      <p class="font-mono text-xs text-primary animate-pulse font-bold">
        5 Adımlı Dağıtık OdemeSaga yürütülüyor (SepetKilitle ➔ Fiyatlandirma ➔ StokRezerve ➔ Odeme ➔ Kesinlestir)...
      </p>
    </div>
  {:else if !secilenSiparis}
    <div class="text-center py-10 text-base-content/40 space-y-2 font-mono text-xs">
      <div class="text-3xl">🔄</div>
      <p class="font-bold">Henüz tetiklenen aktif bir OdemeSaga akışı yok.</p>
      <p class="text-[11px]">Sepet sayfasından 'Siparişi Tamamla' butonuna basarak 5 adımlı dağıtık orkestrasyonu başlatın.</p>
    </div>
  {:else}
    <!-- Saga Adımları Listesi -->
    <div class="space-y-3">
      {#each secilenSiparis.gecmis as adim, i}
        <div class="p-4 rounded-xl border font-mono text-xs flex items-start gap-3 transition-all {adim.durum === 'IptalEdildi' ? 'bg-rose-950/20 border-rose-500/30' : 'bg-base-300 border-base-content/5'}">
          <div class="w-6 h-6 rounded-full flex items-center justify-center font-bold text-[11px] shrink-0 mt-0.5 {adim.durum === 'IptalEdildi' ? 'bg-rose-500/20 text-rose-400 border border-rose-500/40' : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/40'}">
            {i + 1}
          </div>
          <div class="flex-1 space-y-1">
            <div class="font-bold text-base-content/90 flex items-center justify-between">
              <span>{adim.durum}</span>
              <span class="text-[10px] text-base-content/40 font-normal">{new Date(adim.zaman).toLocaleTimeString()}</span>
            </div>
            <div class="text-[11px] {adim.durum === 'IptalEdildi' ? 'text-rose-400 font-bold' : 'text-emerald-300'}">
              {adim.aciklama || 'Adım tamamlandı.'} ({adim.degistirenKullanici})
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
