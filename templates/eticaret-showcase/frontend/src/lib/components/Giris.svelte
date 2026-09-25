<script lang="ts">
  import { KULLANICILAR, oturumAc } from '../hazneler/kimlikHaznesi';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';

  let secilenId = $state('alice');
  let yukleniyor = $state(false);

  let secilenKullanici = $derived(KULLANICILAR.find((k) => k.id === secilenId) || KULLANICILAR[0]);

  function girisYap() {
    yukleniyor = true;
    oturumAc(secilenId);
    bildirimEkle('basari', 'Oturum Açıldı', `Hoş geldiniz, ${secilenKullanici.ad} (${secilenKullanici.rol}) olarak giriş yapıldı.`);
    yukleniyor = false;
  }
</script>

<div class="min-h-screen w-full flex items-center justify-center bg-base-300 p-4 sm:p-6 lg:p-8">
  <div class="max-w-md w-full">
    <!-- Üst Başlık & Logo -->
    <div class="text-center mb-8 space-y-2">
      <div class="inline-flex items-center justify-center w-14 h-14 rounded-2xl bg-gradient-to-tr from-emerald-600 to-teal-600 shadow-xl shadow-emerald-500/20 text-white font-black text-2xl mb-2">
        EC
      </div>
      <h1 class="text-2xl font-black tracking-tight text-base-content">
        clrinf E-Ticaret & Katalog Portalı
      </h1>
      <p class="text-xs text-base-content/60 font-mono">
        Katalog (PIM) • Dinamik Fiyat • 5 Adımlı OdemeSaga • Cedar ABAC
      </p>
    </div>

    <!-- Giriş Kartı -->
    <div class="card bg-base-200 shadow-2xl border border-base-content/10">
      <div class="card-body p-6 sm:p-8 space-y-5">
        <div class="border-b border-base-content/10 pb-3">
          <h2 class="font-bold text-base text-base-content/90">Kurumsal Giriş</h2>
          <p class="text-xs text-base-content/50 mt-0.5">Lütfen rol ve kullanıcı profilinizi seçin</p>
        </div>

        <form onsubmit={(e) => { e.preventDefault(); girisYap(); }} class="space-y-4">
          <div class="form-control">
            <label class="label py-1" for="tenant-input">
              <span class="label-text text-xs font-semibold text-base-content/70">Kurumsal Kiracı (Tenant)</span>
            </label>
            <div id="tenant-input" class="input input-bordered input-sm bg-base-300 flex items-center justify-between font-mono text-xs text-base-content/80">
              <span>Acme E-Commerce Corp</span>
              <span class="badge badge-xs badge-neutral">tenant-acme-corp</span>
            </div>
          </div>

          <div class="form-control">
            <label class="label py-1" for="user-select">
              <span class="label-text text-xs font-semibold text-base-content/70">Kullanıcı Hesabı (Persona)</span>
            </label>
            <select
              id="user-select"
              class="select select-bordered select-sm bg-base-300 font-medium text-xs w-full"
              bind:value={secilenId}
            >
              {#each KULLANICILAR as k}
                <option value={k.id}>
                  {k.ad} — {k.rol} ({k.departman})
                </option>
              {/each}
            </select>
          </div>

          <!-- Seçilen Kullanıcı Özeti -->
          <div class="p-3.5 rounded-xl bg-base-300/80 border border-base-content/5 space-y-1.5 text-xs">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="w-6 h-6 rounded-full bg-primary/20 text-primary font-bold flex items-center justify-center text-[10px]">
                  {secilenKullanici.avatar}
                </div>
                <span class="font-bold text-base-content">{secilenKullanici.ad}</span>
              </div>
              <span class="badge badge-xs {secilenKullanici.rozetRengi} font-mono">{secilenKullanici.rol}</span>
            </div>
            <div class="text-[11px] text-base-content/60">
              Departman: <span class="font-medium text-base-content/80">{secilenKullanici.departman}</span>
            </div>
            <div class="text-[11px] text-base-content/60">
              Sipariş Limiti: <strong class="text-primary font-mono">{secilenKullanici.siparisLimiti > 100000 ? 'Sınırsız (Admin)' : secilenKullanici.siparisLimiti === 0 ? 'Sipariş Yetkisi Yok' : `$${secilenKullanici.siparisLimiti.toLocaleString()} / işlem`}</strong>
            </div>
          </div>

          <button
            type="submit"
            class="btn btn-primary btn-sm w-full font-bold shadow-lg shadow-primary/20 mt-2"
            disabled={yukleniyor}
          >
            {#if yukleniyor}
              <span class="loading loading-spinner loading-xs"></span>
              Giriş Yapılıyor...
            {:else}
              Portala Giriş Yap
            {/if}
          </button>
        </form>

        <div class="border-t border-base-content/5 pt-4 text-center">
          <div class="inline-flex items-center gap-1.5 text-[11px] font-mono text-base-content/50">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
            Rust Cedar ABAC & RS256 JWT Korumalı
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
