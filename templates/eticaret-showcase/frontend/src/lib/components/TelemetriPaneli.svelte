<script lang="ts">
  interface Props {
    olaylar: any[];
    onTemizle: () => void;
  }

  let { olaylar = [], onTemizle }: Props = $props();
</script>

<div class="space-y-6">
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 bg-base-200 p-5 rounded-2xl border border-base-content/10">
    <div>
      <h3 class="font-bold text-lg text-base-content">NATS JetStream & CloudEvents 1.0 Telemetri Akışı</h3>
      <p class="text-xs text-base-content/60 font-mono mt-0.5">
        Constitution RULE OBSERVABLE-BY-DEFAULT • Asenkron Olay Günlüğü
      </p>
    </div>

    <div class="flex items-center gap-2">
      <a href="http://localhost:4200" target="_blank" class="btn btn-sm btn-primary font-bold">
        Event Inspector Arayüzü (Port 4200) ➔
      </a>
      <button class="btn btn-sm btn-ghost border border-base-content/10" onclick={onTemizle}>
        Temizle
      </button>
    </div>
  </div>

  <div class="card bg-base-200 border border-base-content/10 shadow-xl">
    <div class="card-body p-6 space-y-4">
      <div class="flex items-center justify-between border-b border-base-content/10 pb-3">
        <h4 class="font-bold text-sm text-base-content font-mono">
          Canlı Olay Zarfları ({olaylar.length} Olay)
        </h4>
        <span class="badge badge-sm badge-info font-mono">specversion: 1.0</span>
      </div>

      {#if olaylar.length === 0}
        <div class="text-center py-12 text-base-content/40 space-y-2 font-mono text-xs">
          <div class="text-3xl">📡</div>
          <p class="font-bold">Henüz iletilen bir CloudEvent bulunmuyor.</p>
          <p class="text-[11px]">Sepete ürün ekleyerek veya checkout yaparak NATS olay akışını başlatın.</p>
        </div>
      {:else}
        <div class="space-y-3 overflow-y-auto max-h-[600px] pr-1">
          {#each olaylar as o}
            <div class="p-4 rounded-xl bg-base-300 border border-base-content/5 font-mono text-xs space-y-2">
              <div class="flex items-center justify-between">
                <span class="badge badge-sm badge-primary font-bold">{o.type}</span>
                <span class="text-base-content/40 text-[11px]">{new Date(o.time).toLocaleTimeString()}</span>
              </div>
              <div class="text-[11px] text-base-content/60 grid grid-cols-2 sm:grid-cols-3 gap-2">
                <div>Kaynak: <span class="text-secondary font-semibold">{o.source}</span></div>
                <div>Tenant: <span class="text-accent">{o.tenantid}</span></div>
                <div>Correlation: <span class="text-primary">{o.correlationid?.slice(0, 10)}...</span></div>
              </div>
              <pre class="bg-base-200 p-3 rounded-lg text-emerald-400 text-[11px] overflow-x-auto"><code>{JSON.stringify(o.data, null, 2)}</code></pre>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
