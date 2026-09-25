import { writable, derived, get } from 'svelte/store';
import type { SepetVerisi, UrunKalemi, KampanyaVerisi } from '../types';
import { bildirimEkle } from './bildirimHaznesi';
import { aktifKullanici } from './kimlikHaznesi';
import { stokGetir } from './stokHaznesi';

export const sepetId = writable<string>('sepet-' + Math.random().toString(36).substring(2, 8));
export const sepetAcik = writable<boolean>(false);
export const sepetiAc = () => sepetAcik.set(true);
export const seciliUrunDetay = writable<UrunKalemi | null>(null);
export const odemeModalAcik = writable<boolean>(false);
export const aktifSayfa = writable<'vitrin' | 'katalog' | 'kampanyalar' | 'siparisler'>('vitrin');
export const favoriler = writable<string[]>([]);
export const motorHesaplamaSuresi = writable<number>(0);
export const aktifKampanyaListesi = writable<KampanyaVerisi[]>([]);
// Saf Başlangıç Sepeti (Arayüzde hiçbir sahte veri tutulmaz)
export const sepet = writable<SepetVerisi>({
  sepetId: '',
  kullaniciId: 'alice',
  kalemler: [],
  kuponKodu: null,
  indirimOrani: 0,
  araToplam: 0,
  indirimTutari: 0,
  kargoUcreti: 0,
  kdvTutari: 0,
  genelToplam: 0,
  kilitli: false,
});

export const sepetKalemSayisi = derived(sepet, ($s) => $s.kalemler.reduce((acc, k) => acc + k.miktar, 0));

// Rust Kampanya Motorundan (8082) Aktif Kampanyaları Dinamik Çekme
export async function yukleAktifKampanyalar(): Promise<KampanyaVerisi[]> {
  try {
    const kampanyaHost = import.meta.env.VITE_CAMPAIGN_URL || 'http://localhost:8082';
    const res = await fetch(`${kampanyaHost}/kampanya/listele`);
    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data)) {
        const mapped: KampanyaVerisi[] = data.map((k: any) => ({
          kod: k.kod,
          baslik: k.baslik,
          aciklama: k.aciklama,
          tur: k.tur === 'Yuzdesel' || k.tur === 'YUZDESEL' ? 'Yuzdesel' : k.tur === 'SabitTutar' || k.tur === 'SABIT_TUTAR' ? 'SabitTutar' : 'KategoriIndirimi',
          deger: k.deger,
          hedefKategori: k.hedef_kategori || null,
          minimumSepetTutari: k.minimum_sepet_tutari || 0,
          toplamKullanimSayisi: k.toplam_kullanim_sayisi || 0,
          maksimumKullanimSayisi: k.maksimum_kullanim_sayisi || null,
          aktif: k.aktif !== false,
          tukendiMi: false,
          baslangicTarihi: new Date().toISOString()
        }));
        aktifKampanyaListesi.set(mapped);
        return mapped;
      }
    }
  } catch (err) {
    console.warn('Kampanya motoru servis bağlantısı bekleniyor:', err);
  }
  return [];
}

// Rust Kampanya Motoru (POST /kampanya/hesapla) & .NET Sepet Senkronizasyonu
export async function senkronizeEtVeHesapla(guncel: SepetVerisi): Promise<SepetVerisi> {
  const araToplam = guncel.kalemler.reduce((toplam, k) => toplam + (k.birimFiyat * k.miktar), 0);
  let indirimTutari = 0;

  // 1. Rust Kampanya Motoru API Çağrısı
  if (guncel.kuponKodu && araToplam > 0) {
    try {
      const kampanyaHost = import.meta.env.VITE_CAMPAIGN_URL || 'http://localhost:8082';
      const kampanyaIstegi = {
        sepet_id: guncel.sepetId || get(sepetId),
        kullanici_id: guncel.kullaniciId || get(aktifKullanici).id,
        sepet_tutari: araToplam,
        kalemler: guncel.kalemler.map(k => ({
          sku: k.sku,
          miktar: k.miktar,
          birim_fiyat: k.birimFiyat,
          kategori_yolu: k.kategoriYolu || 'Genel'
        })),
        talep_edilen_kampanyalar: [guncel.kuponKodu]
      };

      const res = await fetch(`${kampanyaHost}/kampanya/hesapla`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(kampanyaIstegi)
      });

      if (res.ok) {
        const data = await res.json();
        if (data.cozumleme_suresi_mikrosaniye) {
          motorHesaplamaSuresi.set(data.cozumleme_suresi_mikrosaniye);
        }
        if (typeof data.toplam_indirim === 'number') {
          indirimTutari = data.toplam_indirim;
        }

        if (data.reddedilen_kampanyalar && data.reddedilen_kampanyalar.length > 0) {
          const red = data.reddedilen_kampanyalar[0];
          bildirimEkle({
            tur: 'uyari',
            baslik: `Kampanya Reddedildi (${red.kod})`,
            mesaj: red.sebep,
            sure: 4000
          });
        }
      }
    } catch (err) {
      console.warn('Kampanya motoru hesaplama hatası:', err);
    }
  }

  // 2. .NET 10 WebAPI Sepet Kalıcılığı
  try {
    const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';
    fetch(`${apiHost}/api/Sepet`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        kullaniciId: guncel.kullaniciId || get(aktifKullanici).id,
        toplamTutar: araToplam,
        indirimTutari: indirimTutari,
        kilitli: false
      })
    }).catch(() => {});
  } catch {}

  const kargoUcreti = araToplam >= 500 || araToplam === 0 ? 0 : 45;
  const vergiMatrahi = Math.max(0, araToplam - indirimTutari);
  const kdvTutari = Math.round(vergiMatrahi * 0.18 * 100) / 100;
  const genelToplam = Math.round((vergiMatrahi + kdvTutari + kargoUcreti) * 100) / 100;

  return {
    ...guncel,
    araToplam,
    indirimTutari,
    kargoUcreti,
    kdvTutari,
    genelToplam
  };
}

export async function sepeteEkle(urun: UrunKalemi, miktar: number = 1): Promise<boolean> {
  const s = get(sepet);
  const mevcutIndex = s.kalemler.findIndex(k => k.sku === urun.sku);
  const mevcutMiktar = mevcutIndex >= 0 ? s.kalemler[mevcutIndex].miktar : 0;
  const talepEdilenToplam = mevcutMiktar + miktar;

  // 1. Canlı Stok Haznesi Kontrolü (Single Source of Truth)
  const anlikMusaitStok = stokGetir(urun.sku);

  if (anlikMusaitStok <= 0) {
    bildirimEkle({
      tur: 'uyari',
      baslik: 'Stok Tükendi',
      mesaj: `${urun.baslik} için depoda mevcut stok kalmamıştır.`,
      sure: 4000
    });
    return false;
  }

  if (talepEdilenToplam > anlikMusaitStok) {
    bildirimEkle({
      tur: 'uyari',
      baslik: 'Stok Sınırı Aşıldı',
      mesaj: `${urun.baslik} için toplam stok ${anlikMusaitStok} adettir. Sepetinizde zaten ${mevcutMiktar} adet bulunuyor.`,
      sure: 4500
    });
    return false;
  }

  let yeniKalemler = [...s.kalemler];

  if (mevcutIndex >= 0) {
    yeniKalemler[mevcutIndex].miktar = talepEdilenToplam;
    yeniKalemler[mevcutIndex].toplamTutar = talepEdilenToplam * yeniKalemler[mevcutIndex].birimFiyat;
  } else {
    yeniKalemler.push({
      sku: urun.sku,
      baslik: urun.baslik,
      miktar: miktar,
      birimFiyat: urun.birimFiyat,
      eskiFiyat: urun.eskiFiyat,
      toplamTutar: urun.birimFiyat * miktar,
      kategoriYolu: urun.kategoriYolu
    });
  }

  const hesapli = await senkronizeEtVeHesapla({ ...s, kalemler: yeniKalemler });
  sepet.set(hesapli);

  bildirimEkle({
    tur: 'basari',
    baslik: 'Sepete Eklendi',
    mesaj: `${urun.baslik} (${mevcutIndex >= 0 ? `Toplam ${talepEdilenToplam} adet` : `${miktar} adet`}) eklendi.`,
    sure: 3000
  });
  return true;
}

export async function miktarGuncelle(sku: string, yeniMiktar: number) {
  const s = get(sepet);
  const hedefKalem = s.kalemler.find(k => k.sku === sku);
  if (!hedefKalem) return;

  const anlikMusaitStok = stokGetir(sku);

  if (yeniMiktar > anlikMusaitStok) {
    bildirimEkle({
      tur: 'uyari',
      baslik: 'Maksimum Stok Sınırı',
      mesaj: `Bu ürün için depoda mevcut müsait stok en fazla ${anlikMusaitStok} adettir.`,
      sure: 3500
    });
    return;
  }

  if (yeniMiktar <= 0) {
    await sepettenCikar(sku);
    return;
  }

  let yeniKalemler = s.kalemler.map(k => {
    if (k.sku === sku) {
      return {
        ...k,
        miktar: yeniMiktar,
        toplamTutar: yeniMiktar * k.birimFiyat
      };
    }
    return k;
  });

  const hesapli = await senkronizeEtVeHesapla({ ...s, kalemler: yeniKalemler });
  sepet.set(hesapli);
}

export async function sepettenCikar(sku: string) {
  const s = get(sepet);
  const yeniKalemler = s.kalemler.filter(k => k.sku !== sku);
  
  bildirimEkle({
    tur: 'bilgi',
    baslik: 'Ürün Çıkarıldı',
    mesaj: `Ürün sepetten çıkarıldı.`,
    sure: 2500
  });

  const hesapli = await senkronizeEtVeHesapla({ ...s, kalemler: yeniKalemler });
  sepet.set(hesapli);
}

export async function kuponKoduUygula(kod: string) {
  const s = get(sepet);
  const kodTemiz = kod.toUpperCase().trim();
  const guncel = {
    ...s,
    kuponKodu: kodTemiz
  };
  const hesapli = await senkronizeEtVeHesapla(guncel);
  sepet.set(hesapli);

  if (hesapli.indirimTutari > 0) {
    bildirimEkle({
      tur: 'basari',
      baslik: 'Kampanya Doğrulandı! 🏷️',
      mesaj: `${kodTemiz} kuponu uygulandı: -$${hesapli.indirimTutari.toLocaleString('tr-TR')} indirim!`,
      sure: 3000
    });
  }
}

export async function kuponuKaldir() {
  const s = get(sepet);
  const hesapli = await senkronizeEtVeHesapla({ ...s, kuponKodu: null, indirimOrani: 0 });
  sepet.set(hesapli);

  bildirimEkle({
    tur: 'bilgi',
    baslik: 'Kupon Kaldırıldı',
    mesaj: 'Kupon kodu iptal edildi.',
    sure: 2500
  });
}

export function sepetiTemizle() {
  sepet.set({
    sepetId: 'sepet-' + Math.random().toString(36).substring(2, 8),
    kullaniciId: get(aktifKullanici).id,
    kalemler: [],
    kuponKodu: null,
    indirimOrani: 0,
    araToplam: 0,
    indirimTutari: 0,
    kargoUcreti: 0,
    kdvTutari: 0,
    genelToplam: 0,
    kilitli: false,
  });
}

export function favoriDegistir(sku: string) {
  favoriler.update(f => {
    if (f.includes(sku)) {
      bildirimEkle({ tur: 'bilgi', baslik: 'Favorilerden Çıkarıldı', mesaj: 'Ürün favori listenizden çıkarıldı.', sure: 2000 });
      return f.filter(x => x !== sku);
    } else {
      bildirimEkle({ tur: 'basari', baslik: 'Favorilere Eklendi ❤️', mesaj: 'Ürün favori listenize eklendi.', sure: 2000 });
      return [...f, sku];
    }
  });
}
