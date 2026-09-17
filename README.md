# 🚀 QERVON — Logistics Operating System (LOS)

Yüksek performanslı, modüler, çok kiracılı (multi-tenant) ve yapay zeka destekli lojistik işletim sistemi.

---

## 📌 Proje Hakkında (About Qervon)

**Qervon**, yalnızca sıradan bir kurye uygulaması değildir. Yemek, e-ticaret, kargo, eczane, market ve teknik servis gibi farklı sektörlerde faaliyet gösteren işletmeler, kurumsal firmalar ve uluslararası lojistik şirketleri için tasarlanmış **Lojistik İşletim Sistemi'dir (Logistics Operating System - LOS)**.

Sistem 5 ana bileşenden oluşmaktadır:

1. **Yönetim Paneli (Admin Dashboard):** Canlı GPS haritası, Isı Haritası (Heatmap), AI Dispatcher atamaları, Kurye & Müşteri yönetimi, Finansal Cüzdan raporları.
2. **Web Müşteri Portalı:** Kurumsal müşteriler için sipariş oluşturma, canlı harita takibi ve sunucu fiyatlandırmalı CSV toplu sipariş yükleme.
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

- **iOS Kurye + Müşteri (Swift / SwiftUI / CoreLocation / MapKit):** `mobile/ios/` — tek Xcode projesinde iki gerçek, derlenebilir app target'ı (`QervonCourierApp`, `QervonCustomerApp`), bkz. [mobile/ios/README.md](mobile/ios/README.md).
- **Android Kurye + Müşteri (Kotlin / Jetpack Compose / FusedLocation / osmdroid):** `mobile/android/` — çok-modüllü, `./gradlew assembleDebug` ile gerçek APK üreten iki uygulama (`app-courier`, `app-customer`), bkz. [mobile/android/README.md](mobile/android/README.md).

### 🌐 Web Platform

- **Admin paneli, müşteri portalı, mobil simülatörler (vanilla HTML/CSS/JS):** `backend/apps/api-gateway/static/` — Axum tarafından `include_str!` ile doğrudan sunulan, gerçek `/v1/...` API'lerine bağlı çalışan sayfalar (ayrı bir derleme adımı, `node_modules` veya build aracı yok). Bu, projenin resmi web platformudur — docs'ta tarif edilen ayrı bir React/Vite platformu (`web/`) inşa edilmedi; boş bir iskelet olduğundan kaldırıldı. Ayrıntılar için alttaki "Canlı Web Ekranları" ve `BACKEND_BACKLOG.md`.

---

## 🌐 Canlı Web Ekranları (Live Web Interfaces)

Backend sunucusu çalıştığında (`http://localhost:8080`), bunlar gerçek, çalışan sayfalardır (statik dosya değil, canlı API entegrasyonu vardır):

- **Kurumsal Tanıtım Sitesi:** `http://localhost:8080/` — dış dünyaya açık, Qervon'un ne olduğunu anlatan pazarlama sayfası (`home.html`). Herhangi bir API çağrısı yapmaz, tamamen statiktir.
- **Yönetim Paneli (Admin):** `http://localhost:8080/admin` (eski `/index.html` yolu da geriye dönük uyumluluk için çalışır)
- **Web Müşteri Portalı:** `http://localhost:8080/customer.html`
- **Mobil Müşteri Simülatörü:** `http://localhost:8080/mobile-customer.html`
- **Mobil Kurye Terminal Simülatörü:** `http://localhost:8080/mobile-courier.html`
- **Giriş / Kayıt:** `http://localhost:8080/login`
- **İlk Kurulum:** `http://localhost:8080/setup`

Kimlik doğrulama `HttpOnly`+`SameSite` çerezleri ve çift-gönderim (double-submit) CSRF token'ıyla korunur; tüm kullanıcı girdisi render edilmeden önce HTML-escape edilir. Sipariş ücretleri her zaman sunucuda (`GET /v1/customer/fare-quote` önizleme + `POST /v1/customer/orders` kesin hesaplama) belirlenir, istemci tarafından asla belirlenmez.

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
## .env.local içinde QERVON_POSTGRES_PASSWORD değerini ayarlayın.
make dev-services-up
```

Docker, Qervon API'sini çalıştırmaz ve VPS dağıtım yolu değildir. Yalnızca yerel PostgreSQL ve Redis ihtiyacı olduğunda kullanılır.

### 4. VPS dağıtımı

VPS'te API doğrudan release binary olarak `systemd` altında çalışır; PostgreSQL ve Redis yerel servis veya yönetilen servis olabilir. Kurulum, geri alma ve sağlık kontrolü adımları için [Deployment Runbook](docs/operations/deployment-runbook.md) belgesini izleyin.

---

## 🔒 Kimlik, tenant ve canlı konum güvenliği

Kullanıcı oturumu, tenant slug'ını tek başına yetki olarak kabul etmez: girişte kullanıcı-tenant üyeliği doğrulanır. Erişim belirteçleri 15 dakika ömürlü ve imzalıdır; yenileme belirteçleri veritabanında yalnızca özetlenmiş biçimde tutulur, her yenilemede döndürülür ve çıkışta geçersizleştirilir. İlk tenant sahibi, public API üzerinden değil VPS'te tek kullanımlık bootstrap komutuyla oluşturulur; ayrıntılar [Deployment Runbook](docs/operations/deployment-runbook.md) içindedir.

Kurye ve sipariş kaynakları tenant sahipliğiyle bağlanır. Kurye konumu yalnız kendi tenant’ına ait kayıt için yayımlanabilir; admin kendi tenant’ının son konumlarını, müşteri ise yalnız kendi siparişine atanmış kuryeyi görebilir. Kurye ekranı tarayıcı GPS izniyle konumu yollar. Tarayıcı erişimi HttpOnly oturum çerezleriyle yapılır; hassas değerler URL’ye veya sayfa içi depolamaya yazılmaz.

### Kurye hesabı ve oturum akışı

1. Boş bir yerel geliştirme ortamında ilk platform yöneticisi ve ilk tenant, `/setup` ekranından birlikte oluşturulur. PostgreSQL/VPS ortamında bu ekran yalnız `QERVON_INITIAL_SETUP_TOKEN` ile çalışır; alternatif olarak güvenli VPS bootstrap komutu kullanılabilir. Ayrıntılar [Deployment Runbook](docs/operations/deployment-runbook.md) içindedir.
2. Yönetici `/login` üzerinden kendi tenant kodu, e-posta ve parolasıyla giriş yapar.
3. Yönetim ekranındaki **Firma ve Operasyon Ekibi** bölümünde kurye adı, e-posta adresi, ilk parolası ve aracı girilerek **Kurye hesabı oluştur** seçilir. Bu işlem kullanıcı hesabını, tenant üyeliğini ve kurye kaydını birlikte oluşturur.
4. Kurye aynı HTTPS alan adındaki `/mobile-courier.html` ekranında **Kurye girişi** ile tenant kodu, e-posta ve parolasını girer. Başarılı girişten sonra cihaz konum izni istenir; kabul edilirse konum yalnız kendi tenant’ına canlı olarak yayınlanır.

`localhost` geliştirmesinde tarayıcı oturum çerezleri güvenli yerel bağlantıya uyarlanır. iPhone’dan gerçek GPS alınması için sayfa VPS üzerindeki HTTPS alan adından açılmalıdır; `localhost` iPhone’un kendisini ifade eder, Mac’i değil.

---

## 📱 Mobil Faz-2.1: Destekleyici Backend API'leri

Native iOS/Android uygulamalarının ihtiyaç duyacağı backend API'leri hazır: telefonla OTP girişi (`/v1/auth/otp/request`, `/v1/auth/otp/verify`), kurye cüzdanı (`/v1/courier/me/wallet`, teslimatta otomatik kazanç kredisi), müşteri değerlendirmesi ve destek biletleri (`/v1/customer/orders/{id}/rating`, `/v1/customer/support-tickets`), promosyon kuponları (`/v1/coupons`, sipariş oluştururken `coupon_code`), sipariş ödeme yöntemi (`payment_method`, kurye teslimde `payment_collected` onayı) ve native push cihaz kaydı (`/v1/push/devices`). Bunların hepsi gerçek, kalıcı (memory + PostgreSQL) ve testlidir.

Üç noktada, üçüncü taraf sağlayıcı kimlik bilgisi gerektiren ve bu ortamda bulunmayan bir sınır bilinçli olarak bırakılmıştır (ayrıntılar [BACKEND_BACKLOG.md](BACKEND_BACKLOG.md) içinde):

- **OTP kodu üretilip doğrulanıyor, ancak SMS ile gönderilmiyor.** Bellek (yerel/geliştirme) modunda kod test kolaylığı için `dev_code` alanında döner; PostgreSQL modunda yalnız sunucu loguna yazılır, hiçbir zaman HTTP yanıtına konmaz.
- **Kart/QR/Cüzdan ödeme yöntemleri gerçek bir ödeme geçidine bağlı değil.** Yalnızca seçilen yöntem kaydedilir, tahsilat yapılmaz (ve PCI kapsamına girilmez). Yalnızca nakit için kurye tarafı "tahsil edildi" onayı gerçektir.
- **Native push token'ları kaydediliyor ama hiçbir APNs/FCM gönderimi yok.** Mevcut tarayıcı web-push işçisiyle (`backend/apps/worker`) aynı "kayıt / gönderim" ayrımını izler; gönderim tarafı gerçek Apple/Google kimlik bilgisi gerektirir.

## 📱 Mobil Faz-2.2: iOS Kurye Uygulaması (native, tam kapsam)

`mobile/ios/` artık gerçek, `xcodegen generate` ile üretilen ve `xcodebuild` ile
doğrulanmış, derlenebilir bir Xcode projesidir (ayrıntılar için
[mobile/ios/README.md](mobile/ios/README.md)). Kurye uygulaması giriş
(parola + telefon/OTP + biyometrik kilit), online/offline, iş teklifi
(geri sayımlı kabul/red — bkz. altta), harici navigasyon (Apple/Google/Yandex),
teslim alma/verme (gerçek QR/barkod tarama, dijital imza, yerel foto kaydı),
kazanç/cüzdan, puanlar ve profil ekranlarının tamamını PDF vizyonuna uygun
şekilde gerçek backend endpoint'leriyle çalışır durumda içerir.

Bu fazın ön koşulu olarak backend'e küçük ama gerçek bir **"teklif → kabul/red"**
atama akışı eklendi: müşteri sipariş oluşturduğunda (`auto_assign_for_tenant`)
artık kurye anında ve kesin olarak atanmıyor; en yakın uygun kuryeye **teklif**
ediliyor (`Assignment.status = Offered`, sipariş `Pending`, kurye `Available`
kalıyor). Kurye `GET /v1/courier/me/offer` ile teklifi görür, 45 saniye içinde
`POST /v1/courier/orders/{id}/accept` ya da `.../reject` ile yanıtlar; süre
dolarsa teklif sunucu tarafında lazy olarak `Cancelled`'a döner. Reddedilen/
süresi dolan teklif artık otomatik olarak aynı tenant içindeki bir sonraki en
uygun müsait kuryeye yeniden teklif ediliyor (`Assignment.excluded_courier_ids`
ile aynı kurye iki kez denenmiyor); tüm müsait kuryeler tükenirse sipariş
`Pending` kalır ve operasyon mevcut anında-atama endpoint'iyle
(`POST /v1/orders/{id}/assign`) devam ettirebilir (2026-08-13'te eklendi,
bkz. `BACKEND_BACKLOG.md`). Kuryenin kendi
puanlarını görebildiği `GET /v1/courier/me/ratings` da bu fazda eklendi.

## 📱 Mobil Faz-2.3: iOS Müşteri Uygulaması (native, tam kapsam)

`mobile/ios/QervonCustomerApp` artık Courier app ile aynı Xcode projesini ve
`Packages/QervonKit` altyapısını paylaşan, gerçek ve derlenebilir ikinci bir
app target'ıdır (ayrıntılar için [mobile/ios/README.md](mobile/ios/README.md)).
Kayıt/giriş (OTP dahil), adres defteri (MapKit arama + pin ile gerçek adres
seçimi), canlı ücret teklifi ile sipariş oluşturma, sipariş geçmişi, canlı
harita + ETA ile takip, iptal, değerlendirme, destek talebi, bildirimler ve
profil ekranlarının tamamını PDF vizyonuna uygun şekilde gerçek backend
endpoint'leriyle çalışır durumda içerir.

Bu fazın ön koşulu olarak backend'e gerçek bir **mesafe-bazlı fiyatlandırma
motoru** eklendi: `POST /v1/customer/orders` artık istemciden ücret kabul
etmiyor — ücreti her zaman sunucu, `qervon_domain::DeliveryPricing` ile
pickup/dropoff mesafesinden kendisi hesaplıyor (tenant başına konfigüre
edilebilir, `GET`/`PUT /v1/pricing`; konfigüre edilmemiş tenant'lar için
gerçek bir varsayılan uygulanıyor: ₺10 taban + ₺2.50/km, ₺15 minimum). Ayrıca
müşterinin kendi bekleyen/atanmış siparişini iptal edebildiği
`POST /v1/customer/orders/{id}/cancel`, tahmini teslim süresi gösteren
`GET /v1/customer/orders/{id}/eta` (mevcut AI ETA motorunu kullanır) ve
sipariş oluştururken teslimat notu/iletişim telefonu alanları eklendi.

## 📱 Mobil Faz-2.4: Android Kurye + Müşteri Uygulamaları (native, tam kapsam)

`mobile/android/` artık gerçek, `./gradlew assembleDebug` ile hem
`app-courier` hem `app-customer` için imzasız `.apk` üreten, çok-modüllü bir
Kotlin/Jetpack Compose projesidir (ayrıntılar için
[mobile/android/README.md](mobile/android/README.md)). iOS'un Faz-2.2/2.3'te
inşa ettiği kapsamın aynısı — her iki uygulamanın tüm ekranları — aynı
`backend/apps/api-gateway` sözleşmesine karşı native Android'de
gerçekleştirildi: giriş (parola + telefon/OTP + biyometrik kilit,
`androidx.biometric`), kurye online/offline + iş teklifi (geri sayımlı
kabul/red), harici navigasyon (`Intent` ile `google.navigation:`/`geo:`),
teslim alma/verme (ML Kit ile gerçek QR/barkod tarama, Compose `Canvas`
imza pedi, CameraX ile yerel foto kaydı), kazanç/cüzdan, puanlar; müşteri
tarafında kayıt/giriş, adres defteri (osmdroid harita + `Geocoder`), canlı
ücret teklifiyle sipariş oluşturma, geçmiş, canlı harita + ETA ile takip,
iptal, değerlendirme, destek talebi, bildirimler ve profil. Bu tamamen
istemci-tarafı bir faz oldu — backend'de hiçbir değişiklik gerekmedi, tüm
endpoint'ler iOS çalışmasından tam olarak biliniyordu. Google Maps SDK
(API anahtarı/faturalandırma gerektirdiğinden) ve native push/FCM
(derleme zamanı `google-services.json` kimlik bilgisi gerektirdiğinden,
eksikliği iOS'takinin aksine derlemeyi kıracağından) bilinçli olarak bu
fazda entegre edilmedi; ayrıntılar `BACKEND_BACKLOG.md` içinde.

## 🌐 Web Platformu Kararı

`web/` altındaki React/Vite iskeleti tamamen boştu — 43 dosya, hepsi yalnızca
lisans header'ı içeren, hiçbir gerçek bileşen/route/API çağrısı olmayan bir
klasör rezervasyonuydu; `node_modules`/lockfile yoktu ve derlenemezdi. Bu
iskelet **silindi**. Bunun yerine, README'nin en başından beri gerçek ve
çalışır durumda olan `backend/apps/api-gateway/static/` altındaki vanilla
HTML/CSS/JS sayfaları (`index.html`, `customer.html`, mobil simülatörler,
`login.html`, `setup.html`) **resmi web platformu** olarak kabul edildi.

Bu karardan önce bu sayfalar üzerinde bir güvenlik + fonksiyonellik denetimi
yapıldı ve şu düzeltmeler uygulandı:

- **Güvenlik**: `mobile-customer.html`'de bir XSS boşluğu (sipariş adres
  etiketleri `escapeHtml()` uygulanmadan render ediliyordu) düzeltildi;
  `lucide` CDN script'i (4 dosyada) `@latest`'ten sabit bir versiyona
  (`1.31.0`) pinlendi (tedarik zinciri riski). Auth çerezleri
  (`HttpOnly`+`SameSite`) ve çift-gönderim CSRF token'ı zaten sağlamdı,
  değişiklik gerekmedi.
- **Fonksiyonellik/dürüstlük**: `customer.html` ve `mobile-customer.html`,
  Faz-2.3'te backend'den kaldırılan eski `fare_amount_minor`/`fare_currency`
  alanlarını göndermeyi bırakıp gerçek `GET /v1/customer/fare-quote` önizlemesi
  - güncel `coupon_code`/`payment_method`/`delivery_note`/`contact_phone`
  alanlarını kullanacak şekilde güncellendi. Sabit kodlanmış sahte veriler
  kaldırıldı: `mobile-customer.html`'deki sahte cüzdan bakiyesi/sadakat puanı/
  adresler artık gerçek `GET /v1/customer/profile` verisine bağlı;
  `mobile-courier.html`'deki sahte kazanç/puan kartları artık gerçek
  `GET /v1/courier/me/wallet` ve `/ratings`'e bağlı. İşlevsiz dekoratif
  kamera tabanlı QR/foto tarama butonları kaldırılıp dürüst "henüz
  uygulanmadı" notlarıyla değiştirildi. Toplu sipariş yükleme ise artık
  `POST /v1/customer/orders/bulk` üzerinden en fazla 100 satırlık UTF-8 CSV
  dosyalarını doğrular; müşteri/tenant bilgisini oturumdan alır ve her ücreti
  sunucuda hesaplar. Portal indirilebilir şablon ve satır bazlı sonuç sunar.

**Kapsam dışı bırakılan (gerçek bir React platformu inşa etmek)**: docs'ta
tarif edilen tam React/Vite platformu (12 feature modülü + 7 paylaşılan paket)
inşa edilmedi — bu, kapsam olarak mobil fazın (iOS+Android) tamamından daha
büyük bir iş olurdu ve zaten çalışan bir arayüzü sıfırdan yeniden yazmak
anlamına gelirdi. Mevcut vanilla HTML/JS yaklaşımı gerçek ve bakımı yapılabilir
durumda; React'e geçiş gerekirse ayrı bir faz olarak ele alınabilir.

---
© 2026 USDTG GROUP TECHNOLOGY LLC / Irfan Gedik. All rights reserved.
