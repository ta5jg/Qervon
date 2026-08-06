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
name: review-web-client
description: Apply the Web Client Review workflow for browser JS/DOM/Canvas client work (game.js, rules.js, cinematic.js, worldmap.js, seat.js); use it before proposing or validating a change.
---

# Web Client Review

This project's shipping client (as of Stage 1) is dependency-free static
HTML/CSS/JS: `index.html`, `game.js`, `rules.js`, `cinematic.js`,
`worldmap.js`, `seat.js`, `styles.css`. There is no build step, no
`package.json`, and no server. Treat the browser as the actual runtime and the
player's machine as hostile.

## Procedure

1. Identify which files are **rules** (must be pure: no DOM, no `localStorage`,
   no `Math.random`, no wall-clock reads) versus **presentation** (renders and
   dispatches, must not decide outcomes). `rules.js` must stay in the first
   category; a change that lets `game.js`/`cinematic.js`/`worldmap.js`/`seat.js`
   read or mutate rules state directly, or that lets rendering code branch on
   an outcome the rules engine hasn't produced, is a contract violation.
2. Grep every touched file for `innerHTML`, `outerHTML`, `document.write`,
   `eval`, `new Function`, and template-literal HTML construction. Any player-
   or content-authored string (champion names, region flavor text, content
   packs under `content/`) reaching the DOM through one of these without
   escaping is an XSS path.
3. Check every `localStorage`/`sessionStorage` read and write site. Until
   Phase 4 (server-authoritative state) lands, the RNG seed and season state
   live in client storage and are directly editable — this is a **documented,
   accepted exploit** for save-scumming, state editing, and outcome
   precomputation (see `PROJECT_CONTEXT.md`). Do not treat it as a bug to fix
   silently; treat it as a boundary to keep visible and not worsen (e.g., don't
   add new authoritative values to client storage without flagging it).
4. Confirm no hidden network calls, analytics, or third-party script tags are
   introduced — this is a fully offline static client today, and anything that
   reaches out to a network is a scope change requiring explicit sign-off.
5. Confirm keyboard operability and contrast are preserved for any new UI
   (per the Accessibility rule); a mouse-only interaction added to the core
   loop is a regression, not a style choice.
6. For anything touching `rules.js`, confirm determinism survives: same seed,
   same command sequence, same output. Non-deterministic iteration order
   (unordered object/Map iteration used as if ordered) or a newly introduced
   wall-clock/timer dependency breaks replay and the future C# port (Phase 5).

## Required Verification

```bash
node scripts/simulate.js
node scripts/check-ui.js
grep -rn "innerHTML\|outerHTML\|document.write\|eval(\|new Function" game.js rules.js cinematic.js worldmap.js seat.js
```

Re-run `scripts/simulate.js` with the same seed before and after a `rules.js`
change and diff the output; a change that alters results for an unrelated seed
is a regression.

## Completion Gate

Do not report a review as clean while an XSS-reachable content path, a rules/
presentation boundary violation, a new hidden network call, or a determinism
break remains unresolved. The `localStorage` client-trust gap is a known,
accepted-until-Phase-4 exception — flag it when a change touches it, but it is
not itself a blocking finding unless the change makes it worse.

## Trigger

Use **Web Client Review** for any change to `game.js`, `rules.js`,
`cinematic.js`, `worldmap.js`, `seat.js`, `index.html`, or `styles.css`.

## Scope Boundary

This skill covers the static browser client only. Server-authoritative state,
network reconciliation, and account security become relevant starting Phase 4
and are out of scope until that architecture exists.

## Deliverable

A Web Client Review finding set with scope, severity or priority, affected
contract (rules purity, XSS surface, storage trust boundary, determinism),
evidence, minimal remediation, and verification status.
