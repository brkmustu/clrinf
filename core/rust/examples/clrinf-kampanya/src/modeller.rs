use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KampanyaTuru {
    Yuzdesel,
    SabitTutar,
    KategoriIndirimi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BirlestirmeTuru {
    Serbest,
    TurKisitli,
    OzelListe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KampanyaKurali {
    pub kod: String,
    pub baslik: String,
    pub aciklama: Option<String>,
    pub tur: KampanyaTuru,
    pub deger: f64,
    pub hedef_kategori: Option<String>,
    pub minimum_sepet_tutari: f64,
    pub maksimum_kullanim_sayisi: Option<u32>,
    pub toplam_kullanim_sayisi: u32,
    pub aktif: bool,
    pub birlestirilebilir: bool,
    pub birlestirme_turu: BirlestirmeTuru,
    pub izinli_kampanya_kodlari: Vec<String>,
    pub izinli_etiketler: Vec<String>,
    pub yasakli_etiketler: Vec<String>,
    pub etiketler: Vec<String>,
    pub uygulama_sirasi: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SepetKalemi {
    pub sku: String,
    pub miktar: u32,
    pub birim_fiyat: f64,
    pub kategori_yolu: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KampanyaHesaplamaIstegi {
    pub sepet_id: String,
    pub kullanici_id: Option<String>,
    pub sepet_tutari: f64,
    pub kalemler: Vec<SepetKalemi>,
    pub talep_edilen_kampanyalar: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UygulananKampanyaSonucu {
    pub kod: String,
    pub baslik: String,
    pub tur: KampanyaTuru,
    pub indirim_tutari: f64,
    pub uygulama_sirasi: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KampanyaHesaplamaYaniti {
    pub sepet_id: String,
    pub orijinal_tutar: f64,
    pub toplam_indirim: f64,
    pub odenecek_tutar: f64,
    pub uygulanan_kampanyalar: Vec<UygulananKampanyaSonucu>,
    pub reddedilen_kampanyalar: Vec<RedGerekcesi>,
    pub cozumleme_suresi_mikrosaniye: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedGerekcesi {
    pub kod: String,
    pub sebep: String,
}
