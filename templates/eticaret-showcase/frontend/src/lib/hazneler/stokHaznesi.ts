import { writable, get } from 'svelte/store';

export interface StokKaydi {
  id?: number;
  sku: string;
  miktar: number;
  rezerveMiktar: number;
  musaitStok: number;
}

// Global Stok Haznesi (Başlangıçta boştur, veritabanından /api/Stok ile beslenir)
export const stoklar = writable<Record<string, StokKaydi>>({});
export const stoklarYukleniyor = writable<boolean>(false);

// Backend WebAPI (/api/Stok - PostgreSQL) üzerinden canlı stokları yükleme
export async function stoklariYukle(): Promise<Record<string, StokKaydi>> {
  stoklarYukleniyor.set(true);
  try {
    const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';
    const res = await fetch(`${apiHost}/api/Stok?PageIndex=0&PageSize=100`);
    if (res.ok) {
      const data = await res.json();
      if (data && data.items && Array.isArray(data.items)) {
        const yeniHarita: Record<string, StokKaydi> = {};
        for (const item of data.items) {
          const miktar = Number(item.miktar) || 0;
          const rezerve = Number(item.rezerveMiktar) || 0;
          yeniHarita[item.sku] = {
            id: item.id,
            sku: item.sku,
            miktar,
            rezerveMiktar: rezerve,
            musaitStok: Math.max(0, miktar - rezerve),
          };
        }
        stoklar.set(yeniHarita);
        return yeniHarita;
      }
    }
  } catch (err) {
    console.warn('Stok haznesi API/DB senkronizasyon hatası:', err);
  } finally {
    stoklarYukleniyor.set(false);
  }
  return get(stoklar);
}

// Belirli bir SKU için güncel müsait stok adedini doğrudan veritabanı haznesinden döner
export function stokGetir(sku: string): number {
  const harita = get(stoklar);
  if (harita[sku] !== undefined) {
    return harita[sku].musaitStok;
  }
  return 0; // Veritabanında kayıt yoksa veya henüz yüklenmediyse 0
}

// Belirli bir SKU için talep edilen miktarın veritabanı stoğuna uygunluğunu denetler
export function stokUygunMu(sku: string, talepEdilenMiktar: number): boolean {
  const musait = stokGetir(sku);
  return musait >= talepEdilenMiktar && musait > 0;
}

// Sipariş gerçekleştiğinde hem hazneyi günceller hem de veritabanına (PUT /api/Stok) kalıcı olarak yansıtır
export async function stokDusur(kalemler: { sku: string; miktar: number }[]): Promise<void> {
  const apiHost = import.meta.env.VITE_API_URL || 'http://localhost:5000';
  const harita = get(stoklar);

  stoklar.update((mevcut) => {
    const guncel = { ...mevcut };
    for (const k of kalemler) {
      if (guncel[k.sku]) {
        const yeniMiktar = Math.max(0, guncel[k.sku].miktar - k.miktar);
        const yeniMusait = Math.max(0, yeniMiktar - guncel[k.sku].rezerveMiktar);
        guncel[k.sku] = {
          ...guncel[k.sku],
          miktar: yeniMiktar,
          musaitStok: yeniMusait,
        };

        // Backend WebAPI & PostgreSQL Stok Tablosu Güncelleme (PUT /api/Stok)
        try {
          if (guncel[k.sku].id) {
            fetch(`${apiHost}/api/Stok`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                id: guncel[k.sku].id,
                sku: k.sku,
                miktar: yeniMiktar,
                rezerveMiktar: guncel[k.sku].rezerveMiktar,
              }),
            }).catch(() => {});
          }
        } catch {}
      }
    }
    return guncel;
  });
}
