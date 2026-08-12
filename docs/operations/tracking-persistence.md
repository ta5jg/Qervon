# Tracking Persistence

With `QERVON_STORAGE=postgres`, courier location samples and tracking-session
lifecycle are persisted through `PgTrackingRepository`.

`tracking.location_points` stores validated latitude, longitude, optional speed,
battery percentage, and timestamp. `tracking.sessions` stores active and ended
courier sessions. PostgreSQL enforces at most one active session per courier.

The live WebSocket channel and in-process latest-location cache remain optimized
for real-time delivery. Persistent tracking data is the durable history; the
live cache is deliberately not a source of record and is rebuilt as couriers
report new locations after an API restart.
