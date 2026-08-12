# Çoklu Sunucu Konum ve Browser Push

PostgreSQL kullanan her API örneği, kurye konumunu önce kalıcı takip deposuna
yazar ve ardından `qervon_location_updates` PostgreSQL bildirim kanalına
yayınlar. Her örnek bu kanalı dinler; uzak örnekten gelen olay kendi WebSocket
istemcilerine de iletilir. Böylece müşteri, admin ve kurye aynı API örneğine
bağlı olmak zorunda değildir.

Browser push aboneliği yalnız imzalı kullanıcı oturumundan alınır. Aboneliğin
endpoint'i, P-256DH anahtarı ve auth anahtarı PostgreSQL'de saklanır. Teslimat
tamamlandığında bildirim `queued` olarak kaydedilir; worker, VAPID ile şifreli
Web Push isteğini gönderir. Başarısız denemeler tekrar edilir, süresi geçmiş
abonelikler teslimat dead-letter kaydında görünür. VAPID özel anahtarı yalnız
worker ortamında bulunur; API yalnız genel anahtarı verir.

Gerekli ortam değerleri `.env.example` içinde listelenmiştir. Push izin isteği
müşteri ve kurye ekranlarındaki **BİLDİRİMLERİ AÇ** düğmesine basılmadan
başlatılmaz.
