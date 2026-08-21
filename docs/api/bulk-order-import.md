# Toplu müşteri siparişi içe aktarma

`POST /v1/customer/orders/bulk` yalnız müşteri oturumu ile çalışır. İstek
gövdesi `text/csv;charset=utf-8` biçimindedir; en fazla 1 MB ve 100 sipariş
kabul edilir.

## CSV sütunları

| Sütun | Zorunlu | Açıklama |
|---|---:|---|
| `reference` | Evet | Dosya içinde benzersiz, 1-64 karakterlik müşteri referansı |
| `pickup_label` | Evet | Alım adresi açıklaması |
| `pickup_latitude` | Evet | -90 ile 90 arasında enlem |
| `pickup_longitude` | Evet | -180 ile 180 arasında boylam |
| `dropoff_label` | Evet | Teslimat adresi açıklaması |
| `dropoff_latitude` | Evet | -90 ile 90 arasında enlem |
| `dropoff_longitude` | Evet | -180 ile 180 arasında boylam |
| `contact_phone` | Evet | En az 10 rakam içeren iletişim numarası |
| `payment_method` | Hayır | `cash` (varsayılan), `card` veya `wallet` |
| `delivery_note` | Hayır | Kurye için teslimat notu |

Müşteri kimliği, tenant, ücret ve para birimi dosyada bulunmaz. Bunlar güvenli
oturumdan ve tenant fiyat tarifesinden sunucuda belirlenir; bu alanları ekleyen
dosyalar reddedilir. QR ödeme geçici olarak kapalıdır.

## Örnek

```csv
reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone,payment_method,delivery_note
SIP-001,"Yıldıztabya, Gaziosmanpaşa",41.0638,28.9351,"Maslak, Sarıyer",41.1082,29.0198,05550000000,cash,"Alıcıyı arayın"
```

Başarılı yanıt `201 Created` döndürür. `orders` dizisinde her `reference` için
oluşan sipariş, sunucunun hesapladığı ücret ve ilk durum yer alır. Dosyadaki
tek bir doğrulama hatasında `422 Unprocessable Entity` döner ve hiçbir satır
oluşturulmaz.
