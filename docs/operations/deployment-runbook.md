---
Bu dosya Qervon projesi dokümantasyonu, `qervon-1.pdf` ve `qervon-2.pdf` içeriklerine dayalı olarak güncellenmiştir.

**PDF 1 özeti:**
— Bir kurye programı; Admin panel, Web Uygulama Müşteri tarafı, Kurye ve Müşteri için mobil uygulama. Çok kolay kullanılabilir ve çok gelişmiş olmalı.Bu tip bir sistemi sıradan bir "kurye uygulaması" olarak değil, lojistik işletim sistemi (Logistics Operating System - LOS) olarak tasarlarsak yıllarca geliştirilebilir ve farklı sektörlere de satılabilir (yemek, e-ticaret, kargo, teknik servis, eczane, market vb.).Ben olsam sistemi 5 ana parçaya ayırırım.1. Yönetim Paneli (Admin)Burası şirketin be...

**PDF 2 özeti:**
Qervon Architecture SpeciﬁcationVersion 0.1 Foundation Draft1. PurposeQervon, modern lojistik ve saha operasyonları için geliştirilen, yüksek performanslı, güvenli, modüler ve çok kiracılı (Multi-Tenant) bir Logistics Operating System'dir.Sistem;•küçük işletmeler,•kurumsal ﬁrmalar,•uluslararası lojistik şirketleriiçin tek platform sunacaktır.Qervon yalnızca bir kurye yazılımı değildir.Qervon;•Delivery•Fleet•Warehouse•Dispatch•Field Service•Retail•Medical Logisticsgibi modüllerin çalışacağı ortak...

*Bu kısım otomatik olarak oluşturulmuştur.*
---

## PDF Belgelerinden Gelen Özet

— Bir kurye programı; Admin panel, Web Uygulama Müşteri tarafı, Kurye ve Müşteri için mobil 
uygulama. Çok kolay kullanılabilir ve çok gelişmiş olmalı.
Bu tip bir sistemi sıradan bir "kurye uygulaması" olarak değil, lojistik işletim sistemi (Logistics 
Operating System - LOS) olarak tasarlarsak yıllarca geliştirilebilir ve farklı sektörlere de satılabilir 
(yemek, e-ticaret, kargo, teknik servis, eczane, market vb.).
Ben olsam sistemi 5 ana parçaya ayırırım.
1. Yönetim Paneli (Admin)
Burası şirketin beyni olacak.
Dashboard
•
Anlık sipariş sayısı
•
Online kurye sayısı
•
Bekleyen teslimatlar
•
Günlük gelir
•
Haftalık gelir
•
Aylık gelir
•
Aktif müşteriler
•
Isı haritası
•
Canlı harita
Sipariş Yönetimi
•
Yeni Sipariş
•
Bekleyen
•
Kurye Atanacak
•
Kurye Yolda
•
Teslim Edildi
•
İptal
•
İade
Filtreleme
•
Tarih
•
Kurye
•
Bölge
•
Firma
•
Durum
Kurye Yönetimi
Her kurye için
•
Profil
Sayfa  / 
1
67
•
Fotoğraf
•
Kimlik
•
Ehliyet
•
Araç Bilgisi
•
Motor/Bisiklet/Otomobil
•
Plaka
•
Sigorta
Canlı
•
Konumu
•
Hızı
•
Batarya
•
Son Görülme
•
Günlük teslimat
•
Performans
Firma Yönetimi
Şirketler
Market
Restoran
Eczane
Kargo
Mağaza
Kurye firmaları
Müşteri Yönetimi
•
Profil
•
Adresler
•
Favoriler
•
Geçmiş Siparişler
•
Puan
•
Şikayetler
Finans
•
Tahsilatlar
•
Kurye Hakedişleri
•
Komisyonlar
•
Primler
•
Cezalar
•
İadeler
Sayfa  / 
2
67
Kampanyalar
Kupon
İndirim
Promosyon
Referans
Sadakat
Bildirim Merkezi
SMS
Push
WhatsApp
Mail
Toplu Bildirim
Raporlar
Kurye Performansı
Gelir
Harita
Yoğunluk
Teslim Süresi
Yetkilendirme
Rol sistemi
Admin
Operator
Muhasebe
Çağrı Merkezi
Bölge Müdürü
Sayfa  / 
3
67
Super Admin
2. Web Müşteri Paneli
Kurumsal müşteriler için.
Sipariş oluşturma
Adres defteri
Toplu sipariş
Excel yükleme
Takip ekranı
Canlı harita
Teslim geçmişi
Fatura
Raporlar
API anahtarları
Webhook yönetimi
3. Mobil Müşteri Uygulaması
Android
iPhone
Ana ekran
Yeni Kurye Çağır
Haritada Konum
Yakındaki Kurye
Tahmini Süre
Sipariş
Alım Adresi
Sayfa  / 
4
67
Teslim Adresi
Fotoğraf
Not
Telefon
Teslimat tipi
Ödeme
Kart
Nakit
QR
Cüzdan
Canlı Takip
Harita
Kurye Nerede
ETA
Bildirimler
Geçmiş
Tekrar Sipariş
Favoriler
4. Kurye Mobil
Bu uygulama çok kritik.
Giriş
Telefon
OTP
Biometrik
Sayfa  / 
5
67
Online / Offline
Tek tuş
Yeni İş
Bildirim
Reddet
Kabul Et
Navigasyon
Google Maps
Apple Maps
Yandex
Teslim Alma
QR
Barkod
Fotoğraf
İmza
Teslim
Fotoğraf
İmza
PIN
QR
Kazanç
Bugün
Hafta
Ay
İstatistik
Sayfa  / 
6
67
Puan
Performans
Teslim
Mesafe
5. API
REST
GraphQL
WebSocket
Webhook
SDK
Canlı Harita
En önemli bölüm.
Gösterilecekler
Tüm kuryeler
Online
Offline
Siparişler
Yoğunluk
Isı Haritası
Gerçek Zamanlı
WebSocket
Socket.io
Redis Pub/Sub
Kafka (çok büyük sistemlerde)
Sayfa  / 
7
67
Bildirimler
Firebase Push
APNs
SMS
WhatsApp
Mail
Yapay Zeka
Bence en büyük fark burada olmalı.
AI Dispatcher
En uygun kuryeyi otomatik seçer.
AI ETA
Trafik
Yağmur
Yoğunluk
Motor tipi
Teslim süresini hesaplar.
AI Route
Bir kuryeye 5–10 teslimatı en verimli sırayla planlar.
AI Fraud Detection
Sahte teslimat
Konum sahtecili

---

<!-- =============================================================================
 File:           docs/operations/deployment-runbook.md
 Project:        Qervon
 Author:         USDTG GROUP TECHNOLOGY LLC
 Developer:      Irfan Gedik
 Created Date:   2026-08-05
 Version:        0.1.0

 Description:
   Defines the Qervon runbook for deployment runbook.

 Specification:
   QMI-000000 and QAS-000014 deployment architecture

 License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Deployment Runbook

# Purpose

This runbook defines how to execute and verify deployment runbook safely in the Qervon platform.

# Steps

## Release preparation

1. Run `make check` from the repository root.
2. Build the release binaries with `scripts/build-release.sh`; this includes the API, migration runner and webhook outbox worker.
3. Back up PostgreSQL before applying a migration.
4. Copy the binaries to `/opt/qervon/bin/` on the VPS and keep the previous API binary as `.previous` for rollback.

## VPS configuration

1. Create a non-login `qervon` user and `/etc/qervon/qervon.env` readable only by that user.
2. Set `QERVON_STORAGE=postgres`, `DATABASE_URL`, a 32+ character random `QERVON_TOKEN_SIGNING_SECRET`, a base64-encoded 32-byte `QERVON_WEBHOOK_ENCRYPTION_KEY`, `QERVON_API_ACCESS_TOKEN`, `QERVON_LISTEN=127.0.0.1:8080`, and `RUST_LOG` in that environment file. These secrets are mandatory in every production runtime.
3. Install `infrastructure/systemd/qervon-api.service` and `infrastructure/systemd/qervon-worker.service` under `/etc/systemd/system/`.
4. Apply migrations with the migration runner before restarting the API.
5. Run `systemctl daemon-reload && systemctl enable --now qervon-api qervon-worker`.
6. Put Caddy or Nginx in front of `127.0.0.1:8080` to terminate TLS; do not expose PostgreSQL or Redis publicly.

## First tenant owner bootstrap

Run this once, directly on the VPS after migrations and before enabling normal user access. Do not put these values in the API environment file or shell history; use a protected one-time environment file and remove it immediately after success.

```bash
sudo -u qervon env \
  DATABASE_URL='postgres://...' \
  QERVON_BOOTSTRAP_ALLOW=confirm \
  QERVON_BOOTSTRAP_TENANT_NAME='Example Logistics' \
  QERVON_BOOTSTRAP_TENANT_SLUG='example-logistics' \
  QERVON_BOOTSTRAP_EMAIL='owner@example.com' \
  QERVON_BOOTSTRAP_PASSWORD='use-a-long-unique-password' \
  /opt/qervon/bin/qervon-bootstrap-admin
```

The command refuses to run without explicit confirmation and refuses to overwrite an existing email or tenant slug. It creates a global super-admin identity plus an `owner` membership for the requested tenant. Public registration can only create customer identities and cannot select a tenant or an elevated role.

## Verification and rollback

1. Check `systemctl status qervon-api` and `journalctl -u qervon-api -n 100`.
2. Verify `curl http://127.0.0.1:8080/health` locally on the VPS before switching proxy traffic.
3. On failure, stop the service, restore the `.previous` binary, restart it, and investigate before reattempting the release.

# Operational Baseline

- Use clear owners and approval gates.
- Verify the result before closing the task.
- Record any exception or rollback path.

# References

- [qervon-2.pdf](/Users/irfangedik/Qervon_Platform/qervon/docs/sources/qervon-2.pdf)
- [docs/operations/README.md](/Users/irfangedik/Qervon_Platform/qervon/docs/operations/README.md)

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Aligned deployment runbook to the source PDFs. |
| 0.2.0 | 2026-08-10 | Added direct binary and systemd VPS deployment procedure. |
