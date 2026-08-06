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
 File:           docs/qfs/QFS-000002-kernel-architecture.md
 Project:        Qervon
 Author:         USDTG GROUP TECHNOLOGY LLC
 Developer:      Irfan Gedik
 Created Date:   2026-08-05
 Version:        0.1.0

 Description:
   Defines a Qervon Foundation Specification document.

 Specification:
   QMI-000000 and the applicable Qervon Foundation Specification.

 License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000002 — Kernel Architecture

**Document ID:** QFS-000002  
**Title:** Kernel Architecture  
**Version:** 0.1.0  
**Status:** Foundation Draft  
**Classification:** Internal / Canonical  
**Language:** English (Canonical)  
**Owner:** Qervon Architecture Board

---

# 1. Purpose

This specification defines the Kernel architecture of the Qervon Foundation.

The Kernel is the central runtime component responsible for hosting, coordinating, supervising, and managing all platform modules.

Business domains are explicitly outside the scope of this specification.

---

# 2. Scope

This specification governs:

- Kernel Runtime
- Module Registration
- Module Lifecycle
- Service Registry
- Dependency Resolution
- Event Dispatch
- Runtime Context
- Startup
- Shutdown
- Health Supervision

It does not define:

- Business Rules
- Domain Models
- User Interfaces
- Persistence Models

---

# 3. Normative Language

The key words **MUST**, **MUST NOT**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as described in RFC 2119.

---

# 4. Kernel Definition

The Kernel SHALL be the single runtime authority within a Qervon process.

The Kernel SHALL provide common runtime services to all Foundation and Domain modules.

The Kernel SHALL remain domain-independent.

The Kernel MUST NOT contain business logic.

---

# 5. Architectural Goals

The Kernel SHALL:

- remain deterministic,
- remain observable,
- remain extensible,
- remain modular,
- remain secure,
- remain technology-independent at the architectural level.

---

# 6. Runtime Ownership

The Kernel SHALL own:

- process initialization,
- dependency graph creation,
- service registration,
- module activation,
- module deactivation,
- graceful shutdown,
- runtime diagnostics.

No module SHALL bypass Kernel ownership.

---

# 7. Kernel Responsibilities

The Kernel SHALL provide:

- lifecycle management,
- dependency injection,
- configuration access,
- event routing,
- scheduling interface,
- service discovery,
- diagnostics,
- runtime metadata,
- health reporting.

The Kernel MUST NOT implement business workflows.

---

# 8. Module Registration

Every module SHALL register itself through the Kernel.

Module registration SHALL include:

- Module Identifier
- Version
- Dependencies
- Public Capabilities
- Lifecycle Hooks

Duplicate module identifiers SHALL be rejected.

---

# 9. Module Lifecycle

Each module SHALL support the following lifecycle phases:

1. Registration
2. Initialization
3. Validation
4. Activation
5. Running
6. Suspension (optional)
7. Deactivation
8. Shutdown

The Kernel SHALL guarantee lifecycle ordering according to declared dependencies.

---

# 10. Dependency Resolution

Dependencies SHALL be declared explicitly.

The Kernel SHALL construct an acyclic dependency graph.

Circular dependencies MUST NOT be permitted.

If dependency resolution fails, affected modules SHALL NOT be activated.

---

# 11. Service Registry

The Kernel SHALL expose a Service Registry.

Services SHALL be resolved through stable interfaces.

Modules MUST NOT depend on implementation classes directly.

---

# 12. Runtime Context

The Kernel SHALL expose a Runtime Context.

The Runtime Context MAY contain:

- Configuration
- Tenant Context
- Correlation Information
- Request Metadata
- Security Context
- Trace Context

Business state MUST NOT be stored in the Runtime Context.

---

# 13. Event Routing

The Kernel SHALL route platform events.

Routing SHALL be contract-based.

Events SHALL remain immutable after publication.

The Kernel MUST NOT modify event payloads.

---

# 14. Error Handling

Kernel failures SHALL be classified.

Minimum classifications:

- Configuration Error
- Dependency Error
- Startup Error
- Runtime Error
- Security Error
- Resource Error

Each failure SHALL produce structured diagnostics.

---

# 15. Observability

The Kernel SHALL expose telemetry for:

- startup duration,
- shutdown duration,
- module activation,
- module failures,
- dependency resolution,
- runtime health,
- resource utilization.

All telemetry SHALL be machine-readable.

---

# 16. Security

Every Kernel operation SHALL execute within an authenticated runtime context when applicable.

Privilege escalation SHALL NOT occur implicitly.

Sensitive configuration SHALL NOT be exposed through diagnostic interfaces.

---

# 17. Performance Requirements

Kernel initialization SHOULD be optimized for predictable startup.

The Kernel SHALL minimize synchronization overhead.

Blocking operations SHOULD NOT occur on critical execution paths unless explicitly required.

---

# 18. Extensibility

The Kernel SHALL support future extensions without requiring changes to existing module contracts.

New capabilities SHALL be introduced through additive interfaces whenever possible.

---

# 19. Compatibility

Kernel public interfaces SHALL be versioned.

Breaking changes SHALL require:

- a new major version,
- migration guidance,
- an approved ADR,
- updated specifications.

---

# 20. Compliance

A Kernel implementation SHALL be considered compliant only if it satisfies every mandatory requirement defined by this specification.

Partial compliance SHALL be documented explicitly.

---

# 21. Out of Scope

The following are intentionally excluded:

- Logistics
- Dispatch Algorithms
- Fleet Management
- Customer Management
- Billing
- Warehouse Operations
- Mobile Applications
- Web Applications

These capabilities are defined in higher-level specifications.

---

# 22. References

- QMI-000000 — Master Architecture Index
- QFS-000001 — Foundation Overview
- QFS-000003 — Runtime Lifecycle
- QFS-000004 — Module System
- QFS-000012 — Observability Runtime
- QES-000002 — Rust Engineering Standard

---

# 23. Revision History

| Version | Date | Description |
|----------|------|-------------|
| 0.1.0 | 2026-08-05 | Initial canonical draft |
