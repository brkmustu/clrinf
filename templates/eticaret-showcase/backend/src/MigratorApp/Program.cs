using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Contexts;

Console.WriteLine("=================================================");
Console.WriteLine(" [EticaretApp] Database Migrator & Seeder Starting ");
Console.WriteLine("=================================================");

var environment = Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT") ?? "Development";

var configuration = new ConfigurationBuilder()
    .SetBasePath(Directory.GetCurrentDirectory())
    .AddJsonFile("appsettings.json", optional: true, reloadOnChange: true)
    .AddJsonFile($"appsettings.{environment}.json", optional: true, reloadOnChange: true)
    .AddEnvironmentVariables()
    .Build();

var connectionString = configuration.GetConnectionString("DefaultConnection") 
    ?? configuration.GetConnectionString("BaseDb") 
    ?? Environment.GetEnvironmentVariable("ConnectionStrings__DefaultConnection")
    ?? "Host=localhost;Port=5432;Database=clrinf_eticaret;Username=postgres;Password=postgres;";

Console.WriteLine($"[INFO] Target Environment: {environment}");
Console.WriteLine($"[INFO] Connection String: {connectionString}");

var optionsBuilder = new DbContextOptionsBuilder<BaseDbContext>();
optionsBuilder.UseNpgsql(connectionString);

using var dbContext = new BaseDbContext(optionsBuilder.Options);

try
{
    Console.WriteLine("[INFO] Ensuring database schema and tables are created...");
    dbContext.Database.EnsureCreated();
    Console.WriteLine("[SUCCESS] Database schema initialized.");

    // 1. Kategoriler Seeding
    if (!dbContext.Set<Kategori>().Any())
    {
        Console.WriteLine("[INFO] Seeding Kategoriler...");
        dbContext.Set<Kategori>().AddRange(
            new Kategori { Ad = "Sensör/Optik", Slug = "sensor-optik", Sira = 1, Aktif = true },
            new Kategori { Ad = "Donanım/IoT Gateway", Slug = "donanim-iot", Sira = 2, Aktif = true },
            new Kategori { Ad = "Sensör/Sıcaklık", Slug = "sensor-sicaklik", Sira = 3, Aktif = true },
            new Kategori { Ad = "Sensör/Basınç", Slug = "sensor-basinc", Sira = 4, Aktif = true },
            new Kategori { Ad = "Otomasyon/Robotik", Slug = "otomasyon-robotik", Sira = 5, Aktif = true }
        );
        dbContext.SaveChanges();
    }

    // 2. Ürünler Seeding
    if (!dbContext.Set<Urun>().Any())
    {
        Console.WriteLine("[INFO] Seeding Urunler...");
        dbContext.Set<Urun>().AddRange(
            new Urun { Sku = "SKU-100", Baslik = "Optik Sensör Modülü X1", Aciklama = "Endüstriyel sınıf yüksek hassasiyetli optik lazer sensör modülü.", BirimFiyat = 1200.00m, ParaBirimi = "USD", KategoriYolu = "Sensör/Optik", MusaitStok = 50, Yayinda = true },
            new Urun { Sku = "SKU-200", Baslik = "Endüstriyel Sıcaklık Sensörü T-500", Aciklama = "Geniş sıcaklık aralığı destekli paslanmaz çelik dijital sensör probu.", BirimFiyat = 850.00m, ParaBirimi = "USD", KategoriYolu = "Sensör/Sıcaklık", MusaitStok = 120, Yayinda = true },
            new Urun { Sku = "SKU-300", Baslik = "IoT Ağ Geçidi Kontrolörü (Edge Gateway)", Aciklama = "Modbus, OPC-UA ve MQTT protokollerini destekleyen Linux edge gateway.", BirimFiyat = 2450.00m, ParaBirimi = "USD", KategoriYolu = "Donanım/IoT Gateway", MusaitStok = 35, Yayinda = true },
            new Urun { Sku = "SKU-990", Baslik = "Düşük Stoklu Basınç Transmitteri P-10", Aciklama = "0-400 bar aralıklı yüksek hassasiyetli hidrolik transmitter.", BirimFiyat = 450.00m, ParaBirimi = "USD", KategoriYolu = "Sensör/Basınç", MusaitStok = 3, Yayinda = true },
            new Urun { Sku = "SKU-880", Baslik = "Endüstriyel Lazer Mesafe Sensörü L-880", Aciklama = "0.01 mm çözünürlüklü lazer triangülasyon sensörü. IP67 koruma sınıfı.", BirimFiyat = 3450.00m, ParaBirimi = "USD", KategoriYolu = "Sensör/Optik", MusaitStok = 22, Yayinda = true },
            new Urun { Sku = "SKU-770", Baslik = "6-Eksen Robotik Kol Servo Sürücüsü R-6", Aciklama = "EtherCAT ve CANopen destekli yüksek hızlı dijital servo motor sürücü kartı.", BirimFiyat = 1850.00m, ParaBirimi = "USD", KategoriYolu = "Otomasyon/Robotik", MusaitStok = 15, Yayinda = true }
        );
        dbContext.SaveChanges();
    }

    // 3. Stoklar Seeding
    if (!dbContext.Set<Stok>().Any())
    {
        Console.WriteLine("[INFO] Seeding Stoklar...");
        dbContext.Set<Stok>().AddRange(
            new Stok { Sku = "SKU-100", Miktar = 50, RezerveMiktar = 0 },
            new Stok { Sku = "SKU-200", Miktar = 120, RezerveMiktar = 0 },
            new Stok { Sku = "SKU-300", Miktar = 35, RezerveMiktar = 0 },
            new Stok { Sku = "SKU-990", Miktar = 3, RezerveMiktar = 0 },
            new Stok { Sku = "SKU-880", Miktar = 22, RezerveMiktar = 0 },
            new Stok { Sku = "SKU-770", Miktar = 15, RezerveMiktar = 0 }
        );
        dbContext.SaveChanges();
    }

    // 4. Kampanyalar Seeding
    if (!dbContext.Set<Kampanya>().Any())
    {
        Console.WriteLine("[INFO] Seeding Kampanyalar...");
        dbContext.Set<Kampanya>().AddRange(
            new Kampanya { Kod = "INDIRIM20", Baslik = "500$ Üzeri %20 Flaş İndirim", Deger = 0.20m, Aktif = true },
            new Kampanya { Kod = "SENSOR20", Baslik = "Optik Sensörlerde Özel %20 İndirim", Deger = 0.20m, Aktif = true },
            new Kampanya { Kod = "YAZ2026", Baslik = "Yaz Sezonu %15 İndirim Kampanyası", Deger = 0.15m, Aktif = true },
            new Kampanya { Kod = "HOSGELDIN50", Baslik = "$50 B2B Hoş Geldin İndirimi", Deger = 50m, Aktif = true }
        );
        dbContext.SaveChanges();
    }

    // 5. Kuponlar Seeding
    if (!dbContext.Set<Kupon>().Any())
    {
        Console.WriteLine("[INFO] Seeding Kuponlar...");
        dbContext.Set<Kupon>().AddRange(
            new Kupon { Kod = "INDIRIM20", IndirimTutari = 200m, MinimumSepetTutari = 500m, KullanimLimiti = 500, KullanilanAdet = 42, SonKullanmaTarihi = DateTime.UtcNow.AddMonths(6), Aktif = true },
            new Kupon { Kod = "SENSOR20", IndirimTutari = 200m, MinimumSepetTutari = 1000m, KullanimLimiti = 100, KullanilanAdet = 12, SonKullanmaTarihi = DateTime.UtcNow.AddMonths(6), Aktif = true },
            new Kupon { Kod = "YAZ2026", IndirimTutari = 150m, MinimumSepetTutari = 300m, KullanimLimiti = 200, KullanilanAdet = 34, SonKullanmaTarihi = DateTime.UtcNow.AddMonths(3), Aktif = true },
            new Kupon { Kod = "HOSGELDIN50", IndirimTutari = 50m, MinimumSepetTutari = 400m, KullanimLimiti = 300, KullanilanAdet = 88, SonKullanmaTarihi = DateTime.UtcNow.AddMonths(12), Aktif = true }
        );
        dbContext.SaveChanges();
    }

    // 6. Değerlendirmeler Seeding
    if (!dbContext.Set<Degerlendirme>().Any())
    {
        Console.WriteLine("[INFO] Seeding Degerlendirmeler...");
        dbContext.Set<Degerlendirme>().AddRange(
            new Degerlendirme { UrunId = 1, KullaniciId = "Ahmet Yılmaz (B2B Tedarikçi)", Puan = 5, Yorum = "Lazer sensörün ölçüm hassasiyeti fabrikadaki toleranslarımızı mükemmel karşıladı. Hızlı teslimat için teşekkürler.", Onaylandi = true },
            new Degerlendirme { UrunId = 1, KullaniciId = "Selin Kaya (Endüstriyel Otomasyon Mühendisi)", Puan = 5, Yorum = "Modbus ve OPC-UA bağlantısı sorunsuz çalıştı. Tavsiye ederim.", Onaylandi = true },
            new Degerlendirme { UrunId = 3, KullaniciId = "Mehmet Can (IoT Sistem Mimarı)", Puan = 5, Yorum = "Edge Gateway performansı harika, kesintisiz telemetri akışı sağlıyor.", Onaylandi = true }
        );
        dbContext.SaveChanges();
    }

    // 7. Siparişler & Kargo & Ödeme Seeding
    if (!dbContext.Set<Siparis>().Any())
    {
        Console.WriteLine("[INFO] Seeding Siparisler & Kargo & Odemeler...");
        dbContext.Set<Siparis>().AddRange(
            new Siparis { SiparisNo = "SIP-948201", KullaniciId = "alice", ToplamTutar = 5723m, Durum = "Kargoda", KargoTakipNo = "TR98421094" }
        );
        dbContext.SaveChanges();
    }

    if (!dbContext.Set<Kargo>().Any())
    {
        dbContext.Set<Kargo>().AddRange(
            new Kargo { SiparisId = 1, KargoFirmasi = "DHL Express", TakipNumarasi = "TR98421094", Durum = "Kargoda", TahminiTeslimTarihi = DateTime.UtcNow.AddDays(2) }
        );
        dbContext.SaveChanges();
    }

    if (!dbContext.Set<Odeme>().Any())
    {
        dbContext.Set<Odeme>().AddRange(
            new Odeme { SiparisId = 1, OdemeNumarasi = "OD-948201", Tutar = 5723m, ParaBirimi = "USD", OdemeYontemi = "KrediKarti", Durum = "Basarili", IslemKodu = "TXN-98421" }
        );
        dbContext.SaveChanges();
    }

    Console.ForegroundColor = ConsoleColor.Green;
    Console.WriteLine("[SUCCESS] All 10 modules successfully migrated and seeded in database!");
    Console.ResetColor();

    return 0;
}
catch (Exception ex)
{
    Console.ForegroundColor = ConsoleColor.Red;
    Console.WriteLine($"[ERROR] Migration/Seeding failed: {ex.Message}");
    Console.WriteLine(ex.StackTrace);
    Console.ResetColor();
    return 1;
}
