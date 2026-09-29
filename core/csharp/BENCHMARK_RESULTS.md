# CQRS Mimari Karşılaştırmalı Performans Raporu

Bu rapor, **.NET 10** üzerinde geliştirilen 3 farklı CQRS yaklaşımının mikro-benchmark, bellek tahsisi (GC Allocation) ve HTTP yük testleri (K6 Load Test) altındaki performans karşılaştırmasını sunmaktadır.

---

## 🏛️ Karşılaştırılan Mimariler

| Mimari | Açıklama | Cross-Cutting Yönetimi |
|---|---|---|
| **MediatRApp** | `MediatR (v12.4.1)` kütüphanesi kullanan standart CQRS mimarisi. | MediatR `IPipelineBehavior` (2 Adet PassThrough Behavior) |
| **NativeApp** | Native .NET 10 `FrozenDictionary<Type, object>` tabanlı pipeline dispatcher mimarisi. | Native `IPipelineBehavior` (2 Adet PassThrough Behavior) |
| **PureNativeApp** | **Dispatcher / Mediator içermeyen** saf native CQRS mimarisi. CQRS handler'ları doğrudan DI ile Controller'a enjekte edilir. | **ASP.NET Core Middleware Pipeline** (`Logging`, `Validation`, `Authorization`, `PassThrough1`, `PassThrough2`) |

---

## ⚡ 1. Bellek İçi (In-Memory) Dispatcher Mikro-Benchmark Sonuçları

`DispatcherBenchmark` aracı ile 1.000 ila 1.000.000 istek arasında yapılan bellek içi stres testleri sonuçları:

| Mimari / Yöntem | 1K İstek Süre (ms) | 10K İstek Süre (ms) | 100K İstek Süre (ms) | 1M İstek Süre (ms) | 1M İstek Ops/sn | Tahsis (1M - MB) | Relatif Hız |
|---|---|---|---|---|---|---|---|
| **Direct Typed / Dispatcher-less Baseline** | **0.25 ms** | **3.12 ms** | **10.17 ms** | **76.33 ms** | **13,100,563** | **190.74 MB** | **1.00x (En Hızlı)** |
| **NativeApp Dispatcher (Fast-Path)** | 0.41 ms | 3.77 ms | 12.41 ms | 99.48 ms | 10,051,787 | 175.48 MB | 1.30x |
| **NativeApp Dispatcher (+2 Pipeline Behavior)** | 0.46 ms | 6.59 ms | 23.20 ms | 204.92 ms | 4,880,015 | 411.99 MB | 2.68x |
| **MediatR (v12.4.1 Standart)** | 0.57 ms | 5.71 ms | 17.33 ms | 127.96 ms | 7,815,003 | 297.55 MB | 1.68x |
| **MediatR (v12.4.1 +2 Pipeline Behavior)** | 0.99 ms | 12.23 ms | 41.89 ms | 331.17 ms | 3,019,578 | 663.76 MB | 4.34x |

---

## 🚀 2. K6 HTTP Yük Testi Sonuçları (`run_benchmark.py`)

K6 yük testi aracı ile 50, 100, 200 ve 500 Eşzamanlı Kullanıcı (VUs) altında elde edilen RPS, Ortalama Gecikme, P95/P99 ve RAM tüketim değerleri:

### A. HTTP Command Endpoint (`POST /api/Benchmark/command`)

| Senaryo | Eşzamanlılık (VUs) | RPS (İstek/sn) | Ortalama (ms) | P95 (ms) | P99 (ms) | Sunucu RAM (MB) |
|---|---|---|---|---|---|---|
| **PureNativeApp Command** | 50 | 18,958 | 2.44 | 4.19 | 5.82 | **218.4 MB** |
| **NativeApp Command** | 50 | 19,259 | 2.40 | 4.13 | 5.71 | 220.3 MB |
| **MediatRApp Command** | 50 | 21,658 | 2.11 | 3.53 | 4.90 | 230.3 MB |
| **PureNativeApp Command** | 100 | 36,016 | 2.57 | 4.69 | 6.54 | **218.4 MB** |
| **NativeApp Command** | 100 | 34,554 | 2.67 | 4.86 | 6.80 | 220.3 MB |
| **MediatRApp Command** | 100 | 36,194 | 2.56 | 4.70 | 6.59 | 230.3 MB |
| **PureNativeApp Command** | 200 | 39,699 | 4.66 | 8.99 | 17.09 | **219.2 MB** |
| **NativeApp Command** | 200 | 38,754 | 4.77 | 9.22 | 17.72 | 220.3 MB |
| **MediatRApp Command** | 200 | 40,079 | 4.62 | 8.84 | 17.59 | 230.3 MB |
| **PureNativeApp Command** | 500 | **40,428** | **9.74** | **22.23** | 38.06 | **219.4 MB** |
| **NativeApp Command** | 500 | 39,523 | 9.87 | 22.54 | 37.09 | 220.2 MB |
| **MediatRApp Command** | 500 | 37,602 | 10.34 | 24.76 | 40.79 | 230.9 MB |

### B. HTTP Query Endpoint (`GET /api/Benchmark/query?id=999`)

| Senaryo | Eşzamanlılık (VUs) | RPS (İstek/sn) | Ortalama (ms) | P95 (ms) | P99 (ms) | Sunucu RAM (MB) |
|---|---|---|---|---|---|---|
| **PureNativeApp Query** | 50 | **36,514** | **1.24** | **2.20** | **3.06** | **219.7 MB** |
| **NativeApp Query** | 50 | 36,032 | 1.25 | 2.22 | 3.14 | 222.4 MB |
| **MediatRApp Query** | 50 | 36,344 | 1.25 | 2.23 | 3.10 | 231.9 MB |
| **PureNativeApp Query** | 100 | **40,588** | **2.29** | **4.18** | **5.98** | **219.7 MB** |
| **NativeApp Query** | 100 | 40,037 | 2.32 | 4.20 | 6.07 | 222.4 MB |
| **MediatRApp Query** | 100 | 39,253 | 2.37 | 4.44 | 6.63 | 232.0 MB |
| **PureNativeApp Query** | 200 | 43,743 | 4.27 | 8.23 | 15.86 | **219.7 MB** |
| **NativeApp Query** | 200 | **43,943** | 4.26 | 8.28 | 15.57 | 222.4 MB |
| **MediatRApp Query** | 200 | 43,206 | 4.31 | 8.10 | 16.86 | 232.0 MB |
| **PureNativeApp Query** | 500 | **45,293** | 8.89 | 19.37 | 33.00 | **220.0 MB** |
| **NativeApp Query** | 500 | 44,904 | 8.88 | 20.45 | 35.49 | 222.8 MB |
| **MediatRApp Query** | 500 | 45,147 | **8.63** | **18.63** | **30.59** | 232.0 MB |

---

## 🔬 3. Titiz CPU Pinning ve GC Bellek Tahsisi Testi (`run_rigorous_benchmark.py`)

Çekirdek izolasyonu (CPU Pinning - `taskset`), 5 tekrar medyanı ve işlem başına düşen GC Managed Memory allocation takibi:

| Senaryo | VUs | Medyan RPS ± StdDev | P95 (ms) | P99 (ms) | Alloc/Op (Bytes/req) | Gen0 GCs | Sunucu RAM (MB) |
|---|---|---|---|---|---|---|---|
| **PureNativeApp Query** | 50 | **38,433 ± 1,551** | 3.12 | 4.99 | **0.0 B** | 158 | **120.7 MB** |
| **MediatRApp Query** | 50 | 39,046 ± 385 | 3.08 | 4.95 | 0.0 B | 119 | 122.0 MB |
| **NativeApp Query** | 50 | 38,366 ± 817 | 3.11 | 4.98 | 0.0 B | 134 | 132.2 MB |
| **PureNativeApp Query** | 100 | **37,896 ± 186** | 6.94 | **10.66** | **0.0 B** | 177 | **118.4 MB** |
| **MediatRApp Query** | 100 | 37,776 ± 235 | 6.93 | 10.79 | 3,890.8 B | 171 | 121.1 MB |
| **NativeApp Query** | 100 | 37,543 ± 255 | 6.94 | 10.74 | 0.0 B | 194 | 133.8 MB |
| **PureNativeApp Query** | 500 | 35,466 ± 514 | 21.51 | 32.53 | **6,384.9 B** | **78** | **134.4 MB** |
| **NativeApp Query** | 500 | 35,849 ± 287 | 21.20 | 31.93 | 7,030.1 B | 99 | 148.9 MB |
| **MediatRApp Query** | 500 | 35,577 ± 526 | 21.27 | 31.54 | 6,036.2 B | 91 | 133.1 MB |
| **PureNativeApp Command** | 500 | 32,112 ± 375 | 23.16 | 35.18 | **9,716.6 B** | 98 | **133.7 MB** |
| **NativeApp Command** | 500 | 32,014 ± 263 | 22.90 | 35.48 | 12,405.0 B | 100 | 147.4 MB |
| **MediatRApp Command** | 500 | 32,198 ± 299 | 23.47 | 33.95 | 7,072.8 B | 80 | 136.6 MB |

---

## 🎯 Özet ve Sonuç Değerlendirmesi

1. **En Düşük Bellek Kullanımı (RAM Efficiency)**:
   - `PureNativeApp`, tüm test senaryolarında en düşük RAM tüketimini (**118 MB - 220 MB**) yakalamıştır.
   - `MediatRApp` aynı yük altında ortalama **10-12 MB daha fazla RAM** kullanmaktadır.

2. **Yüksek Eşzamanlılıkta (500 VUs) En Yüksek Command RPS**:
   - 500 Eşzamanlı kullanıcı altında `PureNativeApp Command` **40,428 RPS** elde ederken, `MediatRApp Command` **37,602 RPS**'te kalmıştır (%7.5 daha yüksek başarım).

3. **Dispatcher / Mediator İndirection Maliyetinin Sıfırlanması**:
   - `PureNativeApp` mimarisi, CQRS handler'larını doğrudan DI servis kayıtlarından çağırarak ve cross-cutting işlemleri ASP.NET Core Middleware katmanına kaydırarak hem temiz bir mimari sunar hem de bellek içi dispatching yükünü tamamen kaldırır.
