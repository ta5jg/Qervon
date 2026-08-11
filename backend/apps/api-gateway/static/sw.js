const CACHE = 'qervon-shell-v1';
const SHELL = ['/mobile-customer', '/mobile-courier', '/login'];
self.addEventListener('install', event => event.waitUntil(caches.open(CACHE).then(cache => cache.addAll(SHELL))));
self.addEventListener('activate', event => event.waitUntil(self.clients.claim()));
self.addEventListener('fetch', event => {
  if (event.request.method !== 'GET' || new URL(event.request.url).pathname.startsWith('/v1/')) return;
  event.respondWith(fetch(event.request).then(response => response).catch(() => caches.match(event.request)));
});
self.addEventListener('push', event => {
  const payload = event.data ? event.data.json() : {};
  const title = payload.title || 'Qervon';
  const options = {
    body: payload.body || 'Yeni operasyon güncellemeniz var.',
    icon: '/manifest.webmanifest',
    badge: '/manifest.webmanifest',
    data: { url: payload.url || '/mobile-customer' }
  };
  event.waitUntil(self.registration.showNotification(title, options));
});
self.addEventListener('notificationclick', event => {
  event.notification.close();
  event.waitUntil(clients.openWindow(event.notification.data?.url || '/mobile-customer'));
});
