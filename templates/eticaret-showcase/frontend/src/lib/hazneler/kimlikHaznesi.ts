import { writable, get } from 'svelte/store';
import type { KullaniciProfili } from '../types';

export const tumKullanicilar = writable<KullaniciProfili[]>([]);
export const aktifKullanici = writable<KullaniciProfili>({
  id: 'alice',
  ad: 'Alice Cooper',
  eposta: 'alice@clrinf.dev',
  rol: 'Admin',
  departman: 'Sistem & Operasyon',
  avatar: 'AC',
  rozetRengi: 'badge-primary',
  siparisLimiti: 10000000,
});
export const aktifToken = writable<string>('');
export const oturumAcik = writable<boolean>(true);
export const cedarDenetimGunlugu = writable<any[]>([]);

export const kullaniciDegistir = (k: any) => profilDegistir(typeof k === 'string' ? k : k.id);

// Rust IAM Auth Servisi (/auth/kullanicilar - Port 8081) üzerinden canlı kullanıcıları yükleme
export async function kullanicilariYukle(): Promise<KullaniciProfili[]> {
  try {
    const authHost = import.meta.env.VITE_AUTH_URL || 'http://localhost:8081';
    const res = await fetch(`${authHost}/auth/kullanicilar`);
    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data) && data.length > 0) {
        tumKullanicilar.set(data);
        const mevcutAktif = get(aktifKullanici);
        const eslesen = data.find((u: KullaniciProfili) => u.id === mevcutAktif.id) || data[0];
        aktifKullanici.set(eslesen);
        return data;
      }
    }
  } catch (err) {
    console.warn('Rust Auth IAM servisine erişilemedi:', err);
  }
  return get(tumKullanicilar);
}

// Rust Auth Servisinden Gerçek JWT Token Alımı ve Profil Değişimi
export async function profilDegistir(kullaniciId: string): Promise<void> {
  const authHost = import.meta.env.VITE_AUTH_URL || 'http://localhost:8081';
  try {
    const res = await fetch(`${authHost}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username: kullaniciId,
        password: 'password123',
      }),
    });

    if (res.ok) {
      const data = await res.json();
      if (data.token) {
        aktifToken.set(data.token);
      }
      if (data.user) {
        aktifKullanici.set(data.user);
        return;
      }
    }
  } catch {
    // Rust auth servisi offline ise yerel listeden güncelle
  }

  const kullanicilar = get(tumKullanicilar);
  const k = kullanicilar.find((u) => u.id === kullaniciId);
  if (k) {
    aktifKullanici.set(k);
  }
}

export function getAuthHeaders(): HeadersInit {
  const token = get(aktifToken);
  return {
    'Content-Type': 'application/json',
    ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
  };
}

export function oturumAc(kullaniciId: string) {
  profilDegistir(kullaniciId);
  oturumAcik.set(true);
}

export function oturumuKapat() {
  oturumAcik.set(false);
}
