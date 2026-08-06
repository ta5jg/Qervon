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
name: perform-supply-chain-assessment
description: Apply the Perform Supply Chain Assessment workflow for relevant perform-supply-chain-assessment work; use it before proposing or validating a change.
---

# Perform Supply Chain Assessment

## Trigger

Use **Perform Supply Chain Assessment** when the task requires an explicitly authorized operational assessment and the affected artifact, acceptance criteria, and authorization boundary are known.

## Inputs

- Target files, affected runtime path, and explicit acceptance criteria
- Existing architecture, tests, deployment/rollback boundary, and applicable profile
- Authorization and stop conditions when the task has security, production, financial, or external effects

## Specialist Procedure

Verify lockfiles, provenance, maintainer/release signals, vulnerable and malicious packages, scripts, transitive reachability, licenses, integrity hashes, CI permissions, and a tested update/removal path. Never silently rewrite dependency locks.

1. Map the path from source to deployed artifact: who can commit, who can release, and what runs in between.
2. Assess build-system permissions, third-party actions, artifact signing, and whether provenance is verified at deploy time.
3. Evaluate dependency integrity: pinned versions, hashes, install scripts, and the maintainer trust behind critical packages.
4. Deliver the smallest set of changes that removes an unreviewed path to production, with verification for each.

## Required Evidence

- A source-level explanation of the affected contract and invariant
- At least one focused automated or reproducible verification appropriate to the capability
- Explicit treatment of boundary, failure, and unauthorized/invalid paths where applicable
- A measurement rather than intuition for every material performance claim

## Scope Boundary

This skill is limited to its named capability and must not absorb adjacent work that has a separate contract, owner, or verification path.

## Guardrails

Never expose secrets, claim unrun checks passed, broaden authorized scope, hide a breaking change, or perform destructive/production actions without explicit approval.

## Deliverable

A Perform Supply Chain Assessment result with scope, inputs, outputs, evidence, remaining risks, and the next owner/action.
