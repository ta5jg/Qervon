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

---
name: review-content-pipeline
description: Apply the Content Pipeline Review workflow for content packs, CONTENT_PIPELINE.md, and scripts/check-content.js; use it before proposing or validating a change.
---

# Content Pipeline Review

The project authors game content (champions, regions, relics, LiveOps event
text, alliance-facing strings) as data under `content/`, validated by
`scripts/check-content.js` against the contract in `CONTENT_PIPELINE.md`. This
pipeline is the actual mechanism for the "large, non-repeating LiveOps
calendar" ambition in `ROADMAP.md`, so its correctness gates content volume the
same way the rules engine gates gameplay correctness.

## Procedure

1. Confirm every content file conforms to the schema/closed vocabulary defined
   in `CONTENT_PIPELINE.md` — reject free-form fields that `check-content.js`
   cannot validate, since an unvalidated field is a silent way to break the
   client or introduce unescaped strings later (see **Web Client Review** for
   the XSS angle).
2. Confirm content data carries no executable payload and no field that a
   downstream consumer would `eval`, inject via `innerHTML`, or otherwise treat
   as code rather than data.
3. Confirm every player-facing string is structured so it can move into a
   locale file without a schema change — `ROADMAP.md` Phase 4 requires every
   player-facing string to be localizable, and retrofitting that later is more
   expensive than designing for it now.
4. Confirm new content is versioned or additive rather than silently
   overwriting or renaming existing IDs referenced elsewhere (save data, rules
   engine lookups, other content files) — a renamed champion or region ID can
   silently break a save or a cross-reference.
5. Confirm tone consistency against `GAME_CONCEPT.md` ("dark, elegant and
   strategic... not horror for horror's sake") and against the "avoid clone
   perception" constraint — flag content that reads as generic Dark War/Last
   War reskinning rather than the project's own voice.

## Required Verification

```bash
node scripts/check-content.js
```

Run against every changed or added file under `content/`. A clean run is
necessary evidence that the schema validates; it is not evidence that the
content is balanced (see **SLG Economy and LiveOps Review** for balance) or
that strings are final copy.

## Completion Gate

Do not report a review as clean while an unvalidated field, an executable
payload in content data, a silently renamed/removed ID with live references, or
a non-localizable player-facing string remains unresolved.

## Trigger

Use **Content Pipeline Review** for any change under `content/`,
`CONTENT_PIPELINE.md`, or `scripts/check-content.js`.

## Scope Boundary

This skill validates content structure, safety, and localization-readiness. It
does not judge gameplay balance, monetization fairness, or retention design —
route those to **SLG Economy and LiveOps Review**.

## Deliverable

A Content Pipeline Review finding set with scope, severity or priority,
affected contract (schema, ID stability, localization-readiness), evidence,
minimal remediation, and verification status.
