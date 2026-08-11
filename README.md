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

### 1. Yerelde doğrudan çalıştırma (varsayılan)
```bash
make api
```
API doğrudan bilgisayarınızda çalışır. Varsayılan adres `127.0.0.1:8080`, varsayılan depolama ise hızlı geliştirme için bellektir.

PostgreSQL ile çalışmak için `.env.example` dosyasını `.env.local` olarak kopyalayın, kendi yerel veritabanı bağlantınızı girin ve `QERVON_STORAGE=postgres` yapın. Ardından migration'ları çalıştırın:

```bash
set -a && source .env.local && set +a
make migrate
make api
```

### 2. Testleri Çalıştırma
```bash
cd backend
cargo test
```

### 3. İsteğe bağlı yerel servisler
```bash
cp .env.example .env.local
# .env.local içinde QERVON_POSTGRES_PASSWORD değerini ayarlayın.
make dev-services-up
```

Docker, Qervon API'sini çalıştırmaz ve VPS dağıtım yolu değildir. Yalnızca yerel PostgreSQL ve Redis ihtiyacı olduğunda kullanılır.

### 4. VPS dağıtımı

VPS'te API doğrudan release binary olarak `systemd` altında çalışır; PostgreSQL ve Redis yerel servis veya yönetilen servis olabilir. Kurulum, geri alma ve sağlık kontrolü adımları için [Deployment Runbook](docs/operations/deployment-runbook.md) belgesini izleyin.

---

## 🔒 Kimlik, tenant ve canlı konum güvenliği

Kullanıcı oturumu, tenant slug'ını tek başına yetki olarak kabul etmez: girişte kullanıcı-tenant üyeliği doğrulanır. Erişim belirteçleri 15 dakika ömürlü ve imzalıdır; yenileme belirteçleri veritabanında yalnızca özetlenmiş biçimde tutulur, her yenilemede döndürülür ve çıkışta geçersizleştirilir. İlk tenant sahibi, public API üzerinden değil VPS'te tek kullanımlık bootstrap komutuyla oluşturulur; ayrıntılar [Deployment Runbook](docs/operations/deployment-runbook.md) içindedir.

Kurye ve sipariş kaynakları tenant sahipliğiyle bağlanır. Kurye konumu yalnız kendi tenant’ına ait kayıt için yayımlanabilir; admin kendi tenant’ının son konumlarını, müşteri ise yalnız kendi siparişine atanmış kuryeyi görebilir. Kurye ekranı tarayıcı GPS izniyle konumu yollar; ekranlar oturum sonrası `qervon_access_token`, kurye ekranı ayrıca `qervon_courier_id`, müşteri ekranı `qervon_tracking_order_id` yerel oturum değerlerini kullanır. Bu değerler kullanıcıya gösterilmez veya URL’ye yazılmaz.

---
© 2026 USDTG GROUP TECHNOLOGY LLC / Irfan Gedik. All rights reserved.
