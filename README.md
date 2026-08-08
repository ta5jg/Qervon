# 🚀 QERVON — Logistics Operating System (LOS)

<p align="center">
  <b>Yüksek Performanslı, Modüler, Çok Kiracılı (Multi-Tenant) ve Yapay Zeka Destekli Lojistik İşletim Sistemi</b>
</p>

---

## 📌 Proje Hakkında (About Qervon)

**Qervon**, yalnızca sıradan bir kurye uygulaması değildir. Yemek, e-ticaret, kargo, eczane, market ve teknik servis gibi farklı sektörlerde faaliyet gösteren işletmeler, kurumsal firmalar ve uluslararası lojistik şirketleri için tasarlanmış **Lojistik İşletim Sistemi'dir (Logistics Operating System - LOS)**.

Sistem 5 ana bileşenden oluşmaktadır:
1. **Yönetim Paneli (Admin Dashboard):** Canlı GPS haritası, Isı Haritası (Heatmap), AI Dispatcher atamaları, Kurye & Müşteri yönetimi, Finansal Cüzdan raporları.
2. **Web Müşteri Portalı:** Kurumsal müşteriler için sipariş oluşturma, canlı harita takibi ve Excel/CSV toplu sipariş yükleme.
3. **Mobil Müşteri Uygulaması (iOS & Android):** Gerçek donanımsal GPS canlı takibi, sipariş oluşturma ve cüzdan.
4. **Kurye Mobil Terminali (iOS Swift & Android Kotlin Native):** Arka planda (`CoreLocation` & `Foreground Service`) kesintisiz çalışan donanımsal GPS yayıncısı, navigasyon ve görev terminali.
5. **Yapay Zeka Engine (AI Dispatcher, Dynamic ETA & Fraud Guard):** Araç tipi, trafik yoğunluğu, hava durumu ve mesafe tabanlı akıllı kurye atama & sahte GPS engelleme motoru.

---

## 🏗️ Mimari Yapı (Architecture Stack)

### 🦀 Backend Core (Rust Clean Architecture)
- **`qervon-domain`:** İş kuralları, domain varlıkları (`Courier`, `Order`, `Customer`, `CourierWallet`, `Tenant`).
- **`qervon-application`:** Kullanım senaryoları (`AiDispatcher`, `Dynamic ETA Engine`, `BulkOrderParser`, `NotificationHubManager`).
- **`qervon-infrastructure`:** PostgreSQL (sqlx) ve In-Memory kalıcılık katmanları.
- **`qervon-api-gateway`:** Axum Web Framework, WebSocket canlı akış sunucusu (`/ws/tracking`, `/ws/tracking/customer`) ve OpenAPI Swagger UI.

### 📱 Native Mobile Applications
- **iOS (Swift / SwiftUI / CoreLocation):** `mobile/ios/QervonCourierApp` & `mobile/ios/QervonCustomerApp`
- **Android (Kotlin / Jetpack Compose / FusedLocation):** `mobile/android/QervonCourierApp` & `mobile/android/QervonCustomerApp`

---

## 🌐 Canlı Web Ekranları (Live Web Interfaces)

Backend sunucusu çalıştığında (`http://localhost:8080`):
- **Yönetim Paneli (Admin):** `http://localhost:8080/index.html`
- **Web Müşteri Portalı:** `http://localhost:8080/customer.html`
- **Mobil Müşteri Simülatörü:** `http://localhost:8080/mobile-customer.html`
- **Mobil Kurye Terminal Simülatörü:** `http://localhost:8080/mobile-courier.html`

---

## ⚡ Hızlı Başlatma (Quick Start)

### 1. Backend Sunucusunu Çalıştırma (Rust API Gateway)
```bash
cd backend
cargo run -p qervon-api-gateway
```
Sunucu varsayılan olarak `0.0.0.0:8080` portunda dinlemeye başlayacaktır.

### 2. Testleri Çalıştırma
```bash
cd backend
cargo test
```

### 3. Docker Compose ile Kaldırma
```bash
docker-compose up -d
```

---

## 🔒 Güvenlik & Sunucu Taraflı İzolasyon (Server-Side Isolation)
Müşterilerin diğer kuryelerin GPS koordinatlarına erişmesini önlemek amacıyla `/ws/tracking/customer?courier_id=...` WebSocket rotasında sunucu katmanında (Rust backend) müşteri-kurye eşleşme güvenlik filtresi uygulanmıştır.

---
© 2026 USDTG GROUP TECHNOLOGY LLC / Irfan Gedik. All rights reserved.
