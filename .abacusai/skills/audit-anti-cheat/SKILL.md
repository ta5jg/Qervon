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
name: audit-anti-cheat
description: Apply the Audit Anti Cheat workflow for relevant audit-anti-cheat work; use it before proposing or validating a change.
---

# Audit Anti Cheat

## Trigger

Use **Audit Anti Cheat** when the task requires an adversarial assessment and the affected artifact, acceptance criteria, and authorization boundary are known.

## Inputs

- Target files, affected runtime path, and explicit acceptance criteria
- Existing architecture, tests, deployment/rollback boundary, and applicable profile
- Authorization and stop conditions when the task has security, production, financial, or external effects

## Specialist Procedure

Define threat model, trusted authority, telemetry minimization, tamper detection limits, server-side validation, appeals, false-positive safeguards, version rollout, and monitoring. Do not collect intrusive data without necessity and consent analysis.

1. Model the cheat classes that matter for this game: client memory edits, injected input, network manipulation, and automation, each with its economic motive.
2. Verify that every competitive outcome is decided by trusted authority, and that detection is a supplement rather than the enforcement mechanism.
3. Assess telemetry against necessity and consent: what is collected, how long it is kept, who can read it, and what a false positive costs a player.
4. Deliver detection changes with rollout, appeal, and false-positive handling, plus the measurement that shows the change worked.

## Required Evidence

- A source-level explanation of the affected contract and invariant
- At least one focused automated or reproducible verification appropriate to the capability
- Explicit treatment of boundary, failure, and unauthorized/invalid paths where applicable
- A measurement rather than intuition for every material performance claim

## Scope Boundary

This skill evaluates game trust, economy, and anti-cheat boundaries; it does not justify invasive telemetry, bypassing client protections, or testing players without authorization.

## Guardrails

Never expose secrets, claim unrun checks passed, broaden authorized scope, hide a breaking change, or perform destructive/production actions without explicit approval.

## Deliverable

A Audit Anti Cheat finding set with scope, severity or priority, affected contract, evidence, minimal remediation, and verification status.
