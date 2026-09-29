use super::modeller::*;
use std::time::Instant;

/// Ultra yüksek performanslı Rust Kampanya Kısıt ve Birleştirme Çözücüsü
pub struct KampanyaMotoru;

impl KampanyaMotoru {
    pub fn hesapla(
        istek: &KampanyaHesaplamaIstegi,
        tum_kampanyalar: &[KampanyaKurali],
    ) -> KampanyaHesaplamaYaniti {
        let baslangic = Instant::now();

        let mut uygulananlar: Vec<UygulananKampanyaSonucu> = Vec::new();
        let mut reddedilenler: Vec<RedGerekcesi> = Vec::new();

        // 1. Talep edilen kampanyaları bul ve filtrele
        let mut aday_kampanyalar: Vec<&KampanyaKurali> = Vec::new();

        for kod in &istek.talep_edilen_kampanyalar {
            if let Some(k) = tum_kampanyalar
                .iter()
                .find(|k| k.kod.eq_ignore_ascii_case(kod))
            {
                if !k.aktif {
                    reddedilenler.push(RedGerekcesi {
                        kod: kod.clone(),
                        sebep: "Kampanya aktif değil.".to_string(),
                    });
                    continue;
                }

                if let Some(maks) = k.maksimum_kullanim_sayisi {
                    if k.toplam_kullanim_sayisi >= maks {
                        reddedilenler.push(RedGerekcesi {
                            kod: kod.clone(),
                            sebep: "Kampanya maksimum kullanım kotası dolmuştur.".to_string(),
                        });
                        continue;
                    }
                }

                if istek.sepet_tutari < k.minimum_sepet_tutari {
                    reddedilenler.push(RedGerekcesi {
                        kod: kod.clone(),
                        sebep: format!(
                            "Minimum sepet tutarı (${:.2}) sağlanamadı (Mevcut: ${:.2}).",
                            k.minimum_sepet_tutari, istek.sepet_tutari
                        ),
                    });
                    continue;
                }

                aday_kampanyalar.push(k);
            } else {
                reddedilenler.push(RedGerekcesi {
                    kod: kod.clone(),
                    sebep: "Kampanya kodu sistemde bulunamadı.".to_string(),
                });
            }
        }

        // 2. Çakışma ve Birleştirilebilirlik (Stackability) Analizi
        let mut secilen_kampanyalar: Vec<&KampanyaKurali> = Vec::new();

        for aday in aday_kampanyalar {
            // Aday Exclusive (Birleştirilemez) ise
            if !aday.birlestirilebilir {
                if secilen_kampanyalar.is_empty() {
                    secilen_kampanyalar.push(aday);
                } else {
                    reddedilenler.push(RedGerekcesi {
                        kod: aday.kod.clone(),
                        sebep: "Münhasır (Exclusive) bir kampanyadır; başka kampanyalarla birleştirilemez.".to_string(),
                    });
                }
                continue;
            }

            // Halihazırda seçilen bir kampanya Exclusive ise yeni kampanya eklenemez
            if secilen_kampanyalar.iter().any(|s| !s.birlestirilebilir) {
                reddedilenler.push(RedGerekcesi {
                    kod: aday.kod.clone(),
                    sebep:
                        "Sepette halihazırda birleştirilemez münhasır bir kampanya bulunmaktadır."
                            .to_string(),
                });
                continue;
            }

            // Yasaklı etiket kontrolü (Blacklist)
            let mut yasakli_cakisti = false;
            for secilen in &secilen_kampanyalar {
                if aday
                    .yasakli_etiketler
                    .iter()
                    .any(|y| secilen.etiketler.contains(y))
                    || secilen
                        .yasakli_etiketler
                        .iter()
                        .any(|y| aday.etiketler.contains(y))
                {
                    reddedilenler.push(RedGerekcesi {
                        kod: aday.kod.clone(),
                        sebep: format!(
                            "'{}' kampanyası ile etiket çakışması nedeniyle birleştirilemez.",
                            secilen.kod
                        ),
                    });
                    yasakli_cakisti = true;
                    break;
                }
            }
            if yasakli_cakisti {
                continue;
            }

            // İzinli kod/etiket kontrolü (Whitelist)
            if !aday.izinli_kampanya_kodlari.is_empty() {
                let kod_uyumlu = secilen_kampanyalar
                    .iter()
                    .all(|s| aday.izinli_kampanya_kodlari.contains(&s.kod));
                if !kod_uyumlu && !secilen_kampanyalar.is_empty() {
                    reddedilenler.push(RedGerekcesi {
                        kod: aday.kod.clone(),
                        sebep:
                            "Yalnızca izin verilen özel kampanya kodları listesiyle birleşebilir."
                                .to_string(),
                    });
                    continue;
                }
            }

            secilen_kampanyalar.push(aday);
        }

        // 3. Sıralı İndirim Uygulaması (Uygulama Sırasına Göre Artan Sırala)
        secilen_kampanyalar.sort_by_key(|k| k.uygulama_sirasi);

        let mut kalan_bakiye = istek.sepet_tutari;
        let mut toplam_indirim = 0.0;

        for kampanya in secilen_kampanyalar {
            let indirim = match kampanya.tur {
                KampanyaTuru::Yuzdesel => (kalan_bakiye * kampanya.deger * 100.0).round() / 100.0,
                KampanyaTuru::SabitTutar => kalan_bakiye.min(kampanya.deger),
                KampanyaTuru::KategoriIndirimi => {
                    if let Some(ref hedef_kat) = kampanya.hedef_kategori {
                        let kat_tutari: f64 = istek
                            .kalemler
                            .iter()
                            .filter(|k| k.kategori_yolu.starts_with(hedef_kat))
                            .map(|k| (k.miktar as f64) * k.birim_fiyat)
                            .sum();
                        (kat_tutari * kampanya.deger * 100.0).round() / 100.0
                    } else {
                        0.0
                    }
                }
            };

            if indirim > 0.0 {
                toplam_indirim += indirim;
                kalan_bakiye = (kalan_bakiye - indirim).max(0.0);

                uygulananlar.push(UygulananKampanyaSonucu {
                    kod: kampanya.kod.clone(),
                    baslik: kampanya.baslik.clone(),
                    tur: kampanya.tur.clone(),
                    indirim_tutari: indirim,
                    uygulama_sirasi: kampanya.uygulama_sirasi,
                });
            }
        }

        let gecen_sure = baslangic.elapsed().as_micros() as u64;

        KampanyaHesaplamaYaniti {
            sepet_id: istek.sepet_id.clone(),
            orijinal_tutar: istek.sepet_tutari,
            toplam_indirim: (toplam_indirim * 100.0).round() / 100.0,
            odenecek_tutar: (kalan_bakiye * 100.0).round() / 100.0,
            uygulanan_kampanyalar: uygulananlar,
            reddedilen_kampanyalar: reddedilenler,
            cozumleme_suresi_mikrosaniye: gecen_sure,
        }
    }
}
