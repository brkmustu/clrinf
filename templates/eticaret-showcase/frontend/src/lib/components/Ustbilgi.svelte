<script lang="ts">
  import { aktifKullanici, oturumuKapat } from '../hazneler/kimlikHaznesi';
  import { sepet } from '../hazneler/sepetHaznesi';

  interface Props {
    aktifSekme: string;
    sekmeDegistir: (sekme: string) => void;
  }

  let { aktifSekme, sekmeDegistir }: Props = $props();

  let toplamKalemSayisi = $derived($sepet?.kalemler?.reduce((t: number, k: any) => t + k.miktar, 0) || 0);
</script>

<header class="navbar bg-base-100 border-b border-base-content/10 px-4 sm:px-6 lg:px-8 sticky top-0 z-40">
  <div class="navbar-start flex items-center gap-4">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-xl bg-gradient-to-tr from-emerald-600 to-teal-600 flex items-center justify-center font-black text-white text-lg shadow-md shadow-emerald-500/20">
        EC
      </div>
      <div>
        <div class="font-extrabold text-base tracking-tight text-base-content flex items-center gap-2">
          clrinf E-Ticaret
          <span class="badge badge-xs badge-primary font-mono">PIM & Cart</span>
        </div>
        <div class="text-[11px] font-mono text-base-content/50">
          Acme E-Commerce Corp • Çoklu Kiracı
        </div>
      </div>
    </div>
  </div>

  <div class="navbar-center hidden lg:flex">
    <ul class="menu menu-horizontal px-1 gap-1 font-medium text-xs">
      <li>
        <button class="{aktifSekme === 'giris' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('giris')}>
          Genel Bakış
        </button>
      </li>
      <li>
        <button class="{aktifSekme === 'katalog' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('katalog')}>
          Ürün Kataloğu (PIM)
        </button>
      </li>
      <li>
        <button class="{aktifSekme === 'kampanyalar' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('kampanyalar')}>
          Kampanyalar & Promosyon
        </button>
      </li>
      <li>
        <button class="{aktifSekme === 'sepet' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('sepet')}>
          Sepetim
          {#if toplamKalemSayisi > 0}
            <span class="badge badge-xs badge-primary font-mono">{toplamKalemSayisi}</span>
          {/if}
        </button>
      </li>
      <li>
        <button class="{aktifSekme === 'siparisler' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('siparisler')}>
          Siparişler (OMS)
        </button>
      </li>
      <li>
        <button class="{aktifSekme === 'cedar' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('cedar')}>
          Cedar Yetki Matrisi
        </button>
      </li>
      <li>
        <button class="{aktifSekme === 'telemetri' ? 'active font-bold' : ''}" onclick={() => sekmeDegistir('telemetri')}>
          Saga & Olay Akışı
        </button>
      </li>
    </ul>
  </div>

  <div class="navbar-end flex items-center gap-3">
    <!-- Sepet Butonu Hızlı Erişim -->
    <button
      class="btn btn-ghost btn-circle btn-sm relative"
      onclick={() => sekmeDegistir('sepet')}
      aria-label="Sepeti Aç"
    >
      <span class="text-base">🛒</span>
      {#if toplamKalemSayisi > 0}
        <span class="badge badge-xs badge-primary absolute top-0 right-0 font-mono">
          {toplamKalemSayisi}
        </span>
      {/if}
    </button>

    <div class="hidden sm:flex flex-col text-right font-mono text-xs">
      <div class="font-bold text-base-content/90 flex items-center gap-1.5 justify-end">
        {$aktifKullanici?.ad}
        <span class="badge badge-xs {$aktifKullanici?.rozetRengi} font-mono">{$aktifKullanici?.rol}</span>
      </div>
      <span class="text-[10px] text-base-content/50">{$aktifKullanici?.departman}</span>
    </div>

    <div class="dropdown dropdown-end">
      <div tabindex="0" role="button" class="btn btn-ghost btn-circle avatar placeholder">
        <div class="bg-base-300 text-base-content/80 rounded-full w-9 text-xs font-bold ring ring-primary ring-offset-base-100 ring-offset-1">
          {$aktifKullanici?.avatar}
        </div>
      </div>
      <ul class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow-2xl bg-base-200 rounded-box w-56 border border-base-content/10">
        <li class="menu-title text-xs font-bold text-base-content/60">Aktif Oturum</li>
        <li class="px-2 py-1 text-xs">
          <div class="font-bold">{$aktifKullanici?.ad}</div>
          <div class="text-[10px] text-base-content/50">{$aktifKullanici?.eposta}</div>
        </li>
        <li class="divider my-1"></li>
        <li>
          <button onclick={oturumuKapat} class="text-error font-semibold">
            Oturumu Kapat
          </button>
        </li>
      </ul>
    </div>
  </div>
</header>
