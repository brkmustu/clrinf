<script lang="ts">
  import { aktifSayfa, kuponKoduUygula } from '../hazneler/sepetHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  function hizliKuponUygula() {
    kuponKoduUygula('INDIRIM20', 0.20);
  }
</script>

<section class="relative overflow-hidden rounded-2xl bg-slate-900 border border-slate-800 shadow-xl p-6 sm:p-10 mb-10">
  <!-- Subtle enterprise grid pattern overlay -->
  <div class="absolute inset-0 bg-[linear-gradient(to_right,#1e293b_1px,transparent_1px),linear-gradient(to_bottom,#1e293b_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] opacity-25 pointer-events-none"></div>

  <div class="relative z-10 grid grid-cols-1 lg:grid-cols-12 gap-8 items-center">
    
    <!-- Left Copy & Enterprise CTA -->
    <div class="lg:col-span-7 space-y-5">
      
      <div class="inline-flex items-center gap-2 px-3 py-1 rounded-md bg-blue-500/10 border border-blue-500/20 text-blue-400 text-xs font-semibold">
        <HeroIcon name="shield-check" size={16} />
        <span>Endüstriyel Sınıf B2B Tedarik Platformu</span>
      </div>

      <h1 class="text-3xl sm:text-4xl lg:text-5xl font-bold text-white tracking-tight leading-tight">
        Hassas Ölçüm Sensörleri & <span class="text-blue-400">Endüstriyel IoT</span> Donanımları
      </h1>

      <p class="text-slate-300 text-sm sm:text-base leading-relaxed max-w-xl">
        ToF lazer sensörleri, dijital proses transmitterları ve edge gateway denetleyicileri. Rust constraint solver motoru ve kurumsal ABAC yetkilendirme altyapısıyla güvenli tedarik.
      </p>

      <!-- CTA Action Buttons -->
      <div class="flex flex-wrap items-center gap-3 pt-2">
        <button 
          type="button"
          onclick={() => $aktifSayfa = 'katalog'}
          class="inline-flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white font-semibold px-5 py-2.5 rounded-lg shadow-sm shadow-blue-600/20 transition-all text-sm cursor-pointer"
        >
          <span>Ürün Kataloğunu İncele</span>
          <HeroIcon name="arrow-right" size={16} />
        </button>

        <button 
          type="button"
          onclick={hizliKuponUygula}
          class="inline-flex items-center gap-2 bg-slate-800 hover:bg-slate-700 text-slate-200 font-medium px-4 py-2.5 rounded-lg border border-slate-700 transition-all text-sm cursor-pointer"
        >
          <HeroIcon name="tag" size={16} class="text-blue-400" />
          <span>Flaş Kampanya Tanımla (INDIRIM20)</span>
        </button>
      </div>

      <!-- Trust Metrics & Compliance -->
      <div class="grid grid-cols-3 gap-4 pt-5 border-t border-slate-800 max-w-lg">
        <div>
          <div class="text-xl font-bold text-white font-mono">10.000+</div>
          <div class="text-xs text-slate-400 mt-0.5">Aktif Stoklu Parça</div>
        </div>
        <div>
          <div class="text-xl font-bold text-blue-400 font-mono">&lt; 1 µs</div>
          <div class="text-xs text-slate-400 mt-0.5">Rust Motor Karar Süresi</div>
        </div>
        <div>
          <div class="text-xl font-bold text-emerald-400 font-mono">%100</div>
          <div class="text-xs text-slate-400 mt-0.5">İşlem ve Stok Güvencesi</div>
        </div>
      </div>

    </div>

    <!-- Right Featured Spec Card -->
    <div class="lg:col-span-5">
      <div class="rounded-xl bg-slate-950/90 border border-slate-800 p-5 shadow-lg">
        
        <div class="flex items-center justify-between pb-3 mb-3 border-b border-slate-800">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-blue-500"></span>
            <span class="text-xs font-bold text-slate-200 uppercase tracking-wider">Öne Çıkan Ürün</span>
          </div>
          <span class="text-xs font-mono text-slate-400 bg-slate-900 px-2 py-0.5 rounded border border-slate-800">SKU-300</span>
        </div>

        <div class="w-full h-40 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center relative overflow-hidden">
          <div class="w-20 h-20 rounded-xl bg-blue-600/20 border border-blue-500/30 flex items-center justify-center text-blue-400">
            <HeroIcon name="cpu-chip" size={40} />
          </div>
          <div class="absolute bottom-2 right-2 text-[11px] bg-slate-950/90 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/30 font-medium">
            Mevcut Stok: 35 Adet
          </div>
        </div>

        <div class="mt-4 space-y-1">
          <h3 class="text-sm font-bold text-white">IoT Ağ Geçidi Kontrolörü (Edge Gateway)</h3>
          <p class="text-xs text-slate-400 line-clamp-2">Modbus, OPC-UA ve MQTT endüstriyel telemetri protokolleri destekli Linux edge donanımı.</p>
        </div>

        <div class="mt-4 flex items-center justify-between pt-3 border-t border-slate-800">
          <div>
            <span class="text-xs text-slate-500 line-through">$2,800</span>
            <div class="text-lg font-bold text-white font-mono">$2,450.00 <span class="text-xs font-normal text-slate-400">USD</span></div>
          </div>
          <button 
            type="button"
            onclick={() => $aktifSayfa = 'katalog'}
            class="inline-flex items-center gap-1.5 bg-blue-600 hover:bg-blue-500 text-white font-medium text-xs px-3.5 py-2 rounded-lg transition-colors cursor-pointer"
          >
            <span>Detayları İncele</span>
            <HeroIcon name="chevron-right" size={14} />
          </button>
        </div>

      </div>
    </div>

  </div>
</section>
