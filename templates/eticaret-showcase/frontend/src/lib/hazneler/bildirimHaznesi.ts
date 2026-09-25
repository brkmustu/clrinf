import { writable } from 'svelte/store';

export interface BildirimKaydi {
  id: string;
  tip: 'basari' | 'hata' | 'uyari' | 'bilgi';
  baslik: string;
  mesaj: string;
  zaman: string;
}

export interface BildirimSecenekleri {
  tur?: 'basari' | 'hata' | 'uyari' | 'bilgi';
  tip?: 'basari' | 'hata' | 'uyari' | 'bilgi';
  baslik: string;
  mesaj: string;
  sure?: number;
  sureMs?: number;
}

export const bildirimler = writable<BildirimKaydi[]>([]);

export function bildirimEkle(
  param: 'basari' | 'hata' | 'uyari' | 'bilgi' | BildirimSecenekleri,
  baslikArg?: string,
  mesajArg?: string,
  sureMsArg = 4000
) {
  let tip: 'basari' | 'hata' | 'uyari' | 'bilgi' = 'bilgi';
  let baslik = '';
  let mesaj = '';
  let sureMs = sureMsArg;

  if (typeof param === 'object' && param !== null) {
    tip = param.tur || param.tip || 'bilgi';
    baslik = param.baslik || '';
    mesaj = param.mesaj || '';
    sureMs = param.sure || param.sureMs || 4000;
  } else {
    tip = param;
    baslik = baslikArg || '';
    mesaj = mesajArg || '';
  }

  const id = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : 'b-' + Math.random().toString(36).substring(2, 9);
  const kayit: BildirimKaydi = {
    id,
    tip,
    baslik,
    mesaj,
    zaman: new Date().toLocaleTimeString('tr-TR', { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
  };

  bildirimler.update((list) => [kayit, ...list]);

  setTimeout(() => {
    bildirimSil(id);
  }, sureMs);
}

export function bildirimSil(id: string) {
  bildirimler.update((list) => list.filter((b) => b.id !== id));
}
