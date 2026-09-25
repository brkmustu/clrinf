export interface KullaniciProfili {
  id: string;
  ad: string;
  eposta: string;
  rol: 'Admin' | 'Purchaser' | 'PricingOfficer' | 'MarketingLead' | 'WarehouseManager';
  departman: string;
  avatar: string;
  rozetRengi: string;
  siparisLimiti: number;
}

export interface KategoriItem {
  id: number;
  ad: string;
  slug: string;
  ikon: string;
  urunSayisi: number;
  aciklama: string;
  oneCikan: boolean;
}

export interface DegerlendirmeItem {
  id: number;
  urunId: number;
  kullaniciAdi: string;
  puan: number;
  yorum: string;
  tarih: string;
  onayliAlici: boolean;
}

export interface UrunKalemi {
  id?: number;
  sku: string;
  baslik: string;
  aciklama?: string;
  birimFiyat: number;
  eskiFiyat?: number;
  paraBirimi: string;
  vergiOrani: number;
  kategoriYolu: string;
  kategoriId?: number;
  musaitStok: number;
  puan: number;
  degerlendirmeSayisi: number;
  etiket?: 'Yeni' | 'CokSatan' | 'Firsat' | 'SinirliStok';
  resimUrl?: string;
  varyantlar?: { ad: string; deger: string }[];
  ozellikler?: { [anahtar: string]: string };
  yayinda: boolean;
}

export interface SepetKalemiModeli {
  sku: string;
  baslik: string;
  miktar: number;
  birimFiyat: number;
  eskiFiyat?: number;
  toplamTutar: number;
  kategoriYolu?: string;
  resimUrl?: string;
}

export interface SepetVerisi {
  sepetId: string;
  kullaniciId: string;
  kalemler: SepetKalemiModeli[];
  kuponKodu?: string | null;
  indirimOrani?: number;
  araToplam: number;
  indirimTutari: number;
  kargoUcreti: number;
  kdvTutari: number;
  genelToplam: number;
  kilitli: boolean;
}

export type SiparisDurumTuru = 
  | 'Onaylandi' 
  | 'Hazirlaniyor' 
  | 'Paketlendi' 
  | 'Kargoda' 
  | 'TeslimEdildi' 
  | 'Tamamlandi' 
  | 'IptalEdildi' 
  | 'IadeTalebi' 
  | 'IadeTamamlandi';

export interface SiparisKalemiVerisi {
  sku: string;
  baslik: string;
  miktar: number;
  birimFiyat: number;
  toplamTutar: number;
}

export interface DurumGecmisVerisi {
  durum: SiparisDurumTuru;
  aciklama?: string;
  degistirenKullanici: string;
  zaman: string;
}

export interface SiparisKaydi {
  siparisId: string;
  sepetId: string;
  kullaniciId: string;
  aliciAdi: string;
  teslimatAdresi: string;
  odemeYontemi: string;
  toplamTutar: number;
  paraBirimi: string;
  durum: SiparisDurumTuru;
  kargoTakipNo?: string | null;
  kargoFirmasi?: string | null;
  tahminiTeslim?: string;
  rmaKodu?: string | null;
  kalemler: SiparisKalemiVerisi[];
  gecmis: DurumGecmisVerisi[];
  olusturulmaZamani: string;
}

export interface KampanyaVerisi {
  kod: string;
  baslik: string;
  aciklama?: string;
  tur: 'Yuzdesel' | 'SabitTutar' | 'KategoriIndirimi';
  deger: number;
  hedefKategori?: string | null;
  minimumSepetTutari: number;
  toplamKullanimSayisi: number;
  maksimumKullanimSayisi?: number | null;
  aktif: boolean;
  tukendiMi: boolean;
  baslangicTarihi: string;
  bitisTarihi?: string | null;
  renkDegeri?: string;
}

export interface CedarDenetimKaydi {
  id: string;
  zaman: string;
  principal: string;
  action: string;
  resource: string;
  karar: 'ALLOW' | 'DENY';
  gerekce: string;
  gecikmeMs: number;
  politika: string;
}
