use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::{Arc, RwLock};

use super::modeller::*;
use super::motor::KampanyaMotoru;
use clrinf_adapters::AppError;
use clrinf_core::RequestContext;

/// In-memory kampanya veritabanı durumu
pub struct KampanyaDeposu {
    pub kampanyalar: RwLock<Vec<KampanyaKurali>>,
}

impl KampanyaDeposu {
    pub fn new() -> Self {
        Self {
            kampanyalar: RwLock::new(vec![
                KampanyaKurali {
                    kod: "INDIRIM20".to_string(),
                    baslik: "500$ Üzeri %20 Flaş İndirim".to_string(),
                    aciklama: Some(
                        "500$ ve üzeri tüm siparişlerde anında %20 indirim ve ücretsiz kargo."
                            .to_string(),
                    ),
                    tur: KampanyaTuru::Yuzdesel,
                    deger: 0.20,
                    hedef_kategori: None,
                    minimum_sepet_tutari: 500.0,
                    maksimum_kullanim_sayisi: Some(500),
                    toplam_kullanim_sayisi: 42,
                    aktif: true,
                    birlestirilebilir: true,
                    birlestirme_turu: BirlestirmeTuru::Serbest,
                    izinli_kampanya_kodlari: vec![],
                    izinli_etiketler: vec!["SEZON".to_string(), "SADAKAT".to_string()],
                    yasakli_etiketler: vec![],
                    etiketler: vec!["FLAS_INDIRIM".to_string()],
                    uygulama_sirasi: 1,
                },
                KampanyaKurali {
                    kod: "HOSGELDIN50".to_string(),
                    baslik: "$50 B2B Hoş Geldin İndirimi".to_string(),
                    aciklama: Some(
                        "400$ ve üzeri tüm siparişlerde anında $50 indirim.".to_string(),
                    ),
                    tur: KampanyaTuru::SabitTutar,
                    deger: 50.0,
                    hedef_kategori: None,
                    minimum_sepet_tutari: 400.0,
                    maksimum_kullanim_sayisi: Some(200),
                    toplam_kullanim_sayisi: 88,
                    aktif: true,
                    birlestirilebilir: true,
                    birlestirme_turu: BirlestirmeTuru::Serbest,
                    izinli_kampanya_kodlari: vec![],
                    izinli_etiketler: vec![],
                    yasakli_etiketler: vec![],
                    etiketler: vec!["B2B".to_string()],
                    uygulama_sirasi: 1,
                },
                KampanyaKurali {
                    kod: "YAZ2026".to_string(),
                    baslik: "Yaz Sezonu %15 İndirim Kampanyası".to_string(),
                    aciklama: Some(
                        "Tüm IoT ve Donanım siparişlerinde geçerli genel indirim.".to_string(),
                    ),
                    tur: KampanyaTuru::Yuzdesel,
                    deger: 0.15,
                    hedef_kategori: None,
                    minimum_sepet_tutari: 500.0,
                    maksimum_kullanim_sayisi: Some(100),
                    toplam_kullanim_sayisi: 12,
                    aktif: true,
                    birlestirilebilir: true,
                    birlestirme_turu: BirlestirmeTuru::Serbest,
                    izinli_kampanya_kodlari: vec![],
                    izinli_etiketler: vec!["SADAKAT".to_string(), "SEZON".to_string()],
                    yasakli_etiketler: vec!["FLAS_INDIRIM".to_string()],
                    etiketler: vec!["SEZON".to_string()],
                    uygulama_sirasi: 2,
                },
                KampanyaKurali {
                    kod: "SENSOR20".to_string(),
                    baslik: "Optik Sensörlerde Özel %20 İndirim".to_string(),
                    aciklama: Some(
                        "Yalnızca Sensör/Optik kategorisindeki ürünlerde geçerlidir.".to_string(),
                    ),
                    tur: KampanyaTuru::KategoriIndirimi,
                    deger: 0.20,
                    hedef_kategori: Some("Sensör/Optik".to_string()),
                    minimum_sepet_tutari: 1000.0,
                    maksimum_kullanim_sayisi: Some(50),
                    toplam_kullanim_sayisi: 5,
                    aktif: true,
                    birlestirilebilir: true,
                    birlestirme_turu: BirlestirmeTuru::Serbest,
                    izinli_kampanya_kodlari: vec![],
                    izinli_etiketler: vec!["SEZON".to_string(), "SADAKAT".to_string()],
                    yasakli_etiketler: vec!["FLAS_INDIRIM".to_string()],
                    etiketler: vec!["KATEGORI_OZEL".to_string()],
                    uygulama_sirasi: 1,
                },
                KampanyaKurali {
                    kod: "ENTERPRISE200".to_string(),
                    baslik: "B2B $200 Sabit Hoş Geldin İndirimi".to_string(),
                    aciklama: Some(
                        "$1.500 ve üzeri tüm kurumsal sepetlerde $200 anında indirim.".to_string(),
                    ),
                    tur: KampanyaTuru::SabitTutar,
                    deger: 200.0,
                    hedef_kategori: None,
                    minimum_sepet_tutari: 1500.0,
                    maksimum_kullanim_sayisi: Some(20),
                    toplam_kullanim_sayisi: 2,
                    aktif: true,
                    birlestirilebilir: true,
                    birlestirme_turu: BirlestirmeTuru::Serbest,
                    izinli_kampanya_kodlari: vec!["SENSOR20".to_string()],
                    izinli_etiketler: vec![],
                    yasakli_etiketler: vec!["FLAS_INDIRIM".to_string()],
                    etiketler: vec!["B2B".to_string()],
                    uygulama_sirasi: 3,
                },
                KampanyaKurali {
                    kod: "EXCLUSIVE_FLAS50".to_string(),
                    baslik: "Tek Başına Geçerli %50 Flaş İndirim".to_string(),
                    aciklama: Some(
                        "Bu kampanya münhasırdır (Exclusive); başka hiçbir promosyonla birleşemez."
                            .to_string(),
                    ),
                    tur: KampanyaTuru::Yuzdesel,
                    deger: 0.50,
                    hedef_kategori: None,
                    minimum_sepet_tutari: 2000.0,
                    maksimum_kullanim_sayisi: Some(10),
                    toplam_kullanim_sayisi: 1,
                    aktif: true,
                    birlestirilebilir: false,
                    birlestirme_turu: BirlestirmeTuru::TurKisitli,
                    izinli_kampanya_kodlari: vec![],
                    izinli_etiketler: vec![],
                    yasakli_etiketler: vec!["*".to_string()],
                    etiketler: vec!["FLAS_INDIRIM".to_string()],
                    uygulama_sirasi: 1,
                },
            ]),
        }
    }
}

pub async fn hesapla_kampanya(
    Extension(_ctx): Extension<RequestContext>,
    Extension(depo): Extension<Arc<KampanyaDeposu>>,
    Json(payload): Json<KampanyaHesaplamaIstegi>,
) -> Result<impl IntoResponse, AppError> {
    let kampanyalar = depo.kampanyalar.read().unwrap();
    let sonuc = KampanyaMotoru::hesapla(&payload, &kampanyalar);

    Ok((StatusCode::OK, Json(sonuc)))
}

pub async fn listele_kampanyalar(
    Extension(_ctx): Extension<RequestContext>,
    Extension(depo): Extension<Arc<KampanyaDeposu>>,
) -> Result<impl IntoResponse, AppError> {
    let kampanyalar = depo.kampanyalar.read().unwrap().clone();
    Ok((StatusCode::OK, Json(kampanyalar)))
}
