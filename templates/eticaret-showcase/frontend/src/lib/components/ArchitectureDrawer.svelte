<script lang="ts">
  import { aktifKullanici } from '../hazneler/kimlikHaznesi';
  import type { CedarDenetimKaydi } from '../types';
  import HeroIcon from './HeroIcon.svelte';

  let acik = $state(false);

  let denetimKayitlari = $state<CedarDenetimKaydi[]>([
    {
      id: 'log-01',
      zaman: new Date().toLocaleTimeString('tr-TR'),
      principal: 'admin',
      action: 'CreateSiparisCommand',
      resource: 'Siparis',
      karar: 'ALLOW',
      gerekce: 'Principal possesses Role::Admin and satisfies Policy 01',
      gecikmeMs: 0.8,
      politika: 'permit(principal == Role::Admin, action, resource);'
    },
    {
      id: 'log-02',
      zaman: new Date().toLocaleTimeString('tr-TR'),
      principal: 'admin',
      action: 'ApplyCoupon',
      resource: 'Kupon',
      karar: 'ALLOW',
      gerekce: 'Coupon code validated via Rust campaign engine (INDIRIM20)',
      gecikmeMs: 0.4,
      politika: 'permit(principal in [MarketingLead, Admin], action == "ApplyCoupon");'
    }
  ]);
</script>

<!-- Floating Toggle Pill -->
<div class="fixed bottom-5 right-5 z-40">
  <button 
    type="button"
    onclick={() => acik = !acik}
    class="flex items-center gap-2 bg-slate-900 hover:bg-slate-800 text-slate-200 font-semibold text-xs px-3.5 py-2.5 rounded-lg border border-slate-700 shadow-xl transition-all cursor-pointer"
  >
    <HeroIcon name="cpu-chip" size={16} class="text-blue-400" />
    <span>Mimari & Cedar Telemetri</span>
    <HeroIcon name={acik ? 'chevron-down' : 'chevron-up'} size={14} class="text-slate-400" />
  </button>
</div>

<!-- Architecture Drawer Modal -->
{#if acik}
  <!-- Backdrop -->
  <div 
    onclick={() => acik = false}
    onkeydown={(e) => { if (e.key === 'Escape') acik = false; }}
    role="button"
    tabindex="0"
    class="fixed inset-0 bg-slate-950/70 backdrop-blur-xs z-50 transition-opacity"
  >
  </div>

  <div class="fixed inset-x-4 bottom-4 md:inset-x-auto md:right-5 md:bottom-16 md:w-[560px] max-h-[80vh] bg-slate-900 border border-slate-700 rounded-xl shadow-2xl z-50 flex flex-col overflow-hidden text-left">
    
    <!-- Drawer Header -->
    <div class="p-3.5 bg-slate-950 border-b border-slate-800 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <HeroIcon name="shield-check" size={16} class="text-blue-400" />
        <span class="text-xs font-bold text-white">Sistem & ABAC Telemetri İzleme</span>
        <span class="px-2 py-0.2 text-[10px] font-semibold rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 font-mono">
          Rust & .NET 10
        </span>
      </div>
      <button onclick={() => acik = false} class="text-slate-400 hover:text-white p-1 rounded hover:bg-slate-800 transition-colors cursor-pointer">
        <HeroIcon name="x-mark" size={16} />
      </button>
    </div>

    <!-- Drawer Content -->
    <div class="p-4 overflow-y-auto space-y-3.5 text-xs">
      
      <!-- Identity Context -->
      <div class="p-3 rounded-lg bg-slate-950 border border-slate-800 space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-slate-400 font-semibold uppercase tracking-wider text-[10px]">Aktif Identity Provider (IdP)</span>
          <span class="text-emerald-400 font-mono font-semibold text-[11px]">Port 8081 (Rust Axum)</span>
        </div>
        <div class="grid grid-cols-2 gap-2 text-[11px]">
          <div>Principal: <strong class="text-white">{$aktifKullanici.id} ({$aktifKullanici.ad})</strong></div>
          <div>Rol / Claims: <strong class="text-blue-400 font-mono">{$aktifKullanici.rol}</strong></div>
          <div>Sipariş Limiti: <strong class="text-slate-200 font-mono">${$aktifKullanici.siparisLimiti.toLocaleString('tr-TR')}</strong></div>
          <div>Tenant ID: <strong class="text-slate-400 font-mono">default</strong></div>
        </div>
      </div>

      <!-- Cedar Decisions -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-slate-400 font-semibold uppercase tracking-wider text-[10px]">Cedar ABAC Güvenlik Kararları</span>
          <span class="text-blue-400 text-[10px] font-mono">Gecikme: &lt; 0.8 ms</span>
        </div>

        <div class="space-y-2">
          {#each denetimKayitlari as log}
            <div class="p-2.5 rounded-lg bg-slate-950 border border-slate-800 space-y-1">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-1.5">
                  <span class="px-1.5 py-0.2 rounded text-[10px] font-bold font-mono {log.karar === 'ALLOW' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'}">
                    {log.karar}
                  </span>
                  <span class="font-mono text-white font-semibold text-[11px]">{log.action}</span>
                </div>
                <span class="text-slate-500 font-mono text-[10px]">{log.zaman}</span>
              </div>
              <p class="text-[11px] text-slate-400 font-mono">{log.politika}</p>
            </div>
          {/each}
        </div>
      </div>

    </div>
  </div>
{/if}
