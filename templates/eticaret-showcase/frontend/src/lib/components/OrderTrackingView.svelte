<script lang="ts">
  import type { SiparisKaydi, SiparisDurumTuru } from '../types';
  import { bildirimEkle } from '../hazneler/bildirimHaznesi';
  import HeroIcon from './HeroIcon.svelte';

  interface Props {
    siparisler: SiparisKaydi[];
  }

  let { siparisler }: Props = $props();

  const adimlar: { durum: SiparisDurumTuru; baslik: string; icon: string }[] = [
    { durum: 'Onaylandi', baslik: 'Sipariş Onaylandı', icon: 'document-text' },
    { durum: 'Hazirlaniyor', baslik: 'Hazırlanıyor', icon: 'cube' },
    { durum: 'Paketlendi', baslik: 'Paketlendi', icon: 'tag' },
    { durum: 'Kargoda', baslik: 'Kargoya Verildi', icon: 'truck' },
    { durum: 'TeslimEdildi', baslik: 'Teslim Edildi', icon: 'check-circle' }
  ];

  function adimIndex(durum: SiparisDurumTuru): number {
    switch (durum) {
      case 'Onaylandi': return 0;
      case 'Hazirlaniyor': return 1;
      case 'Paketlendi': return 2;
      case 'Kargoda': return 3;
      case 'TeslimEdildi':
      case 'Tamamlandi': return 4;
      default: return 0;
    }
  }

  function takipNoKopyala(takipNo?: string | null) {
    if (!takipNo) return;
    navigator.clipboard.writeText(takipNo);
    bildirimEkle({
      tur: 'basari',
      baslik: 'Takip Numarası Kopyalandı',
      mesaj: `${takipNo} panoya kopyalandı.`,
      sure: 2500
    });
  }
</script>

<div class="space-y-5 text-left">
  
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-xl font-bold text-white tracking-tight">Siparişlerim & Sevkiyat Takibi</h2>
      <p class="text-xs text-slate-400">Verdiğiniz endüstriyel satın alma siparişleri ve kargo hareketleri</p>
    </div>
    <span class="px-2.5 py-1 rounded-lg bg-slate-900 border border-slate-800 text-xs text-slate-300 font-mono">
      Toplam {siparisler.length} Sipariş
    </span>
  </div>

  {#if siparisler.length === 0}
    <div class="text-center py-16 rounded-xl bg-slate-900 border border-slate-800 space-y-3">
      <div class="w-12 h-12 rounded-full bg-slate-800 text-slate-400 flex items-center justify-center mx-auto">
        <HeroIcon name="cube" size={24} />
      </div>
      <h3 class="text-sm font-bold text-white">Kayıtlı Sipariş Bulunmuyor</h3>
      <p class="text-xs text-slate-400 max-w-sm mx-auto">Katalogdaki endüstriyel parçaları sepetinize ekleyip siparişinizi oluşturabilirsiniz.</p>
    </div>
  {:else}
    <div class="space-y-4">
      {#each siparisler as sip}
        {@const aktifAdim = adimIndex(sip.durum)}
        
        <div class="rounded-xl bg-slate-900 border border-slate-800 p-5 space-y-5">
          
          <!-- Order Meta Header -->
          <div class="flex flex-wrap items-center justify-between gap-3 pb-3.5 border-b border-slate-800">
            <div>
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold text-white font-mono">{sip.siparisId}</span>
                <span class="px-2 py-0.5 text-[10px] font-semibold rounded bg-blue-500/10 text-blue-400 border border-blue-500/20">
                  {sip.durum}
                </span>
              </div>
              <div class="text-[11px] text-slate-400 mt-1">
                Tarih: {sip.olusturulmaZamani} • Alıcı: <strong class="text-slate-300">{sip.aliciAdi}</strong>
              </div>
            </div>

            <div class="flex items-center gap-4">
              <div class="text-right">
                <div class="text-[10px] text-slate-400">Toplam Tutar</div>
                <div class="text-sm font-bold text-white font-mono">
                  ${sip.toplamTutar.toLocaleString('tr-TR')} {sip.paraBirimi}
                </div>
              </div>

              {#if sip.kargoTakipNo}
                <button 
                  type="button"
                  onclick={() => takipNoKopyala(sip.kargoTakipNo)}
                  class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-xs font-medium text-slate-300 border border-slate-700 transition-colors cursor-pointer"
                >
                  <HeroIcon name="truck" size={14} class="text-blue-400" />
                  <span>{sip.kargoFirmasi || 'DHL'}: {sip.kargoTakipNo}</span>
                </button>
              {/if}
            </div>
          </div>

          <!-- Progress Stepper -->
          <div class="py-2">
            <div class="grid grid-cols-5 gap-2 relative">
              
              <!-- Progress Bar -->
              <div class="absolute top-4 left-6 right-6 h-0.5 bg-slate-800 -z-0">
                <div 
                  class="h-full bg-blue-500 transition-all duration-500" 
                  style="width: {(aktifAdim / 4) * 100}%"
                ></div>
              </div>

              {#each adimlar as adim, i}
                {@const tamamlandi = i <= aktifAdim}
                {@const siradaki = i === aktifAdim}

                <div class="flex flex-col items-center text-center space-y-1.5 relative z-10">
                  <div class="w-8 h-8 rounded-full flex items-center justify-center text-xs transition-colors {tamamlandi ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-500 border border-slate-700'} {siradaki ? 'ring-2 ring-blue-400 ring-offset-2 ring-offset-slate-900' : ''}">
                    <HeroIcon name={adim.icon} size={14} solid={tamamlandi} />
                  </div>
                  <span class="text-[10px] font-medium {tamamlandi ? 'text-white' : 'text-slate-500'}">
                    {adim.baslik}
                  </span>
                </div>
              {/each}

            </div>
          </div>

          <!-- Order Item Rows -->
          {#if sip.kalemler && sip.kalemler.length > 0}
            <div class="pt-3 border-t border-slate-800/80">
              <div class="text-[11px] font-semibold text-slate-400 mb-2 uppercase tracking-wider">Sipariş Kalemleri</div>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {#each sip.kalemler as k}
                  <div class="flex items-center justify-between p-2.5 rounded-lg bg-slate-950 border border-slate-800 text-xs">
                    <div class="truncate mr-2">
                      <div class="font-medium text-white truncate">{k.baslik}</div>
                      <div class="text-[10px] text-slate-400 font-mono">{k.sku} • {k.miktar} adet</div>
                    </div>
                    <div class="font-bold text-slate-200 font-mono shrink-0">
                      ${k.toplamTutar.toLocaleString('tr-TR')}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

        </div>
      {/each}
    </div>
  {/if}

</div>
