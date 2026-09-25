# E-Ticaret Checkout ve 5 Adımlı OdemeSaga Rehberi

> Bu belge, vitrin uygulamasının OdemeSaga akışını ve telafi mekanizmalarını açıklar.

Bu belge, örnek uygulama için [`NO-DISTRIBUTED-TX`](../../../clrinf-contracts/constitution.md) ilkesi doğrultusunda tasarlanan **OdemeSaga** akışını ve telafi mekanizmalarını açıklar.

---

## 1. Neden 2PC Yerine 5 Adımlı Saga?

E-ticaret sipariş sürecinde sepet, stok, promosyon ve ödeme servislerinin tek bir dağıtık veri tabanı transaction'ı (2PC/XA) içine alınması kaskat kilitlenmelere ve SPOF riskine yol açar.

`clrinf` mimarisinde her adım kendi yerel transaction'ını yürütür ve olası hata durumunda telafi edici işlem (Compensating Action) devreye girer:

```
[İstemci] ──(Checkout Başlat)──▶ [OdemeSaga Orkestratörü]
                                       │
      ┌────────────────────────────────┼──────────────────────────────┐
      │ 1. SepetiKilitle               │ 2. Fiyatlandirma             │ 3. StokRezerve
      ▼                                ▼                              ▼
[Sepet Kilitlendi]              [İndirim Hesaplandı]           [Stok Rezerve Edildi]
      │                                                               │
      └────────────────────────────────┬──────────────────────────────┘
                                       │
                                       ▼
                              4. OdemeTahsilati
                              (Harici Gateway)
                                       │
                  ┌────────────────────┴────────────────────┐
                  │                                         │
            [Ödeme Başarılı]                          [Ödeme Başarısız]
                  │                                         │
                  ▼                                         ▼
         5. SiparisiKesinlestir                  Compensating Actions:
         • StokOnayla                            1. StokSerbetBirak (İade)
         • SepetTemizle                          2. SepetKilidiniAc (Müşteriye İade)
         • CloudEvents Yayınla                   3. SiparisTelafiBekliyor Olayı
```

---

## 2. Telafi Edici İşlemler (Compensating Actions)

| Adım | Yürütülen Komut / İşlem | Hata Halinde Telafi İşlemi |
|---|---|---|
| 1. Sepet | `SepetObegi.Kilitle` | `SepetObegi.KilidiAc` |
| 2. Fiyat | `FiyatlandirmaMotoru.Hesapla` | (Salt okunur / Yan etki yok) |
| 3. Stok | `StokRezerveEtKomut` (`StokObegi`) | `StokSerbetBirakKomut` |
| 4. Ödeme | `OdemeTahsilati` | `OdemeIadeEt` (Provizyon iptali) |
| 5. Kesinleştirme | `StokObegi.Onayla` | (Nihai durum, telafi gerekmez) |

---

## 3. Idempotency ve Çift İstek Koruması

Her checkout talebi zorunlu bir `korelasyonId` (`correlation_id`) taşır. `IdempotensiDavranisi` Pipeline Behavior'ı sayesinde ağ kopması veya mükerrer basımlarda karttan iki kez para çekilmesi veya mükerrer stok rezervasyonu yapılması engellenir.
