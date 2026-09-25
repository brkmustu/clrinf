<script lang="ts">
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  let epostaGirdisi = $state('');

  function aboneOl() {
    if (!epostaGirdisi.trim()) return;
    bildirimEkle({
      tur: 'basari',
      baslik: 'Bülten Aboneliği Başarılı',
      mesaj: 'Endüstriyel parça bülteni ve kurumsal fiyat listeleri e-postanıza tanımlandı.',
      sure: 3500
    });
    epostaGirdisi = '';
  }
</script>

<footer class="mt-16 border-t border-slate-800 bg-slate-950 text-left">
  
  <!-- Trust & Compliance Badges -->
  <div class="max-w-7xl mx-auto px-4 sm:px-8 py-8 border-b border-slate-800/80">
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      
      <div class="flex items-center gap-3.5 p-3.5 rounded-xl bg-slate-900 border border-slate-800">
        <div class="w-10 h-10 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
          <HeroIcon name="truck" size={20} />
        </div>
        <div>
          <h4 class="text-xs font-bold text-white">Global B2B Sevkiyat</h4>
          <p class="text-[11px] text-slate-400">$500+ siparişlerde ücretsiz hava kargo</p>
        </div>
      </div>

      <div class="flex items-center gap-3.5 p-3.5 rounded-xl bg-slate-900 border border-slate-800">
        <div class="w-10 h-10 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
          <HeroIcon name="shield-check" size={20} />
        </div>
        <div>
          <h4 class="text-xs font-bold text-white">Cedar ABAC & 3D Secure</h4>
          <p class="text-[11px] text-slate-400">Çok kiracılı kurumsal veri izolasyonu</p>
        </div>
      </div>

      <div class="flex items-center gap-3.5 p-3.5 rounded-xl bg-slate-900 border border-slate-800">
        <div class="w-10 h-10 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
          <HeroIcon name="document-text" size={20} />
        </div>
        <div>
          <h4 class="text-xs font-bold text-white">Sertifikalı CAD & RoHS</h4>
          <p class="text-[11px] text-slate-400">Tüm parçalarda onaylı teknik datasheet</p>
        </div>
      </div>

      <div class="flex items-center gap-3.5 p-3.5 rounded-xl bg-slate-900 border border-slate-800">
        <div class="w-10 h-10 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center">
          <HeroIcon name="wrench-screwdriver" size={20} />
        </div>
        <div>
          <h4 class="text-xs font-bold text-white">7/24 Mühendislik Desteği</h4>
          <p class="text-[11px] text-slate-400">Endüstriyel otomasyon danışmanlığı</p>
        </div>
      </div>

    </div>
  </div>

  <!-- Footer Navigation Links -->
  <div class="max-w-7xl mx-auto px-4 sm:px-8 py-10">
    <div class="grid grid-cols-1 md:grid-cols-12 gap-8">
      
      <!-- Brand & Tech Overview -->
      <div class="md:col-span-4 space-y-3">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-lg bg-blue-600 flex items-center justify-center font-bold text-white text-xs font-mono">
            CL
          </div>
          <span class="text-base font-bold text-white font-mono tracking-tight">CLRINF Enterprise</span>
        </div>
        <p class="text-xs text-slate-400 leading-relaxed max-w-sm">
          .NET 10 Clean-CQRS, Rust Kural Motoru ve PostgreSQL üzerinde çalışan referans vitrin uygulaması.
        </p>
        <div class="text-[11px] text-slate-500 font-mono">
          Engine: <strong>Rust Axum (:8082)</strong> • API: <strong>.NET 10 (:5000)</strong>
        </div>
      </div>

      <!-- Categories Links -->
      <div class="md:col-span-2 space-y-2.5 text-xs">
        <div class="font-bold text-white uppercase tracking-wider text-[11px]">Kategoriler</div>
        <ul class="space-y-1.5 text-slate-400">
          <li><span class="hover:text-blue-400 cursor-pointer">Sensör / Optik</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">Donanım / IoT</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">Sıcaklık Probları</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">Basınç Transmitterları</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">Robotik Kontrolcüler</span></li>
        </ul>
      </div>

      <!-- Procurement Links -->
      <div class="md:col-span-2 space-y-2.5 text-xs">
        <div class="font-bold text-white uppercase tracking-wider text-[11px]">Kurumsal Alım</div>
        <ul class="space-y-1.5 text-slate-400">
          <li><span class="hover:text-blue-400 cursor-pointer">Sipariş & Kargo Takibi</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">RMA & İade Talebi</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">Teknik Şartnameler</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">Toplu Fiyat Teklifi (RFQ)</span></li>
          <li><span class="hover:text-blue-400 cursor-pointer">ABAC Güvenlik Politikaları</span></li>
        </ul>
      </div>

      <!-- Newsletter -->
      <div class="md:col-span-4 space-y-2.5">
        <div class="font-bold text-white uppercase tracking-wider text-[11px]">Teknik Bültene Katılın</div>
        <p class="text-xs text-slate-400">Yeni çıkan endüstriyel parça modelleri ve B2B fiyat listelerinden haberdar olun.</p>
        
        <div class="flex gap-2">
          <input 
            type="email" 
            placeholder="kurumsal@sirketiniz.com"
            bind:value={epostaGirdisi}
            class="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500"
          />
          <button 
            type="button"
            onclick={aboneOl}
            class="bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs px-3.5 py-2 rounded-lg transition-colors cursor-pointer"
          >
            Abone Ol
          </button>
        </div>
      </div>

    </div>
  </div>

  <!-- Copyright -->
  <div class="border-t border-slate-900 bg-slate-950 py-5 text-xs text-slate-500">
    <div class="max-w-7xl mx-auto px-4 sm:px-8 flex flex-wrap items-center justify-between gap-4">
      <div>
        © 2026 clrinf Enterprise Showcase Platformu. Tüm hakları saklıdır.
      </div>
      <div class="flex items-center gap-4 text-[11px]">
        <span>KVKK ve Gizlilik</span>
        <span>•</span>
        <span>Kullanım Koşulları</span>
        <span>•</span>
        <span class="text-emerald-400 font-mono">Tenant: default</span>
      </div>
    </div>
  </div>

</footer>
