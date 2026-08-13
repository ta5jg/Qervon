# iOS Acceptance Artifacts (v3)

This folder contains the final iOS acceptance screenshot set for customer and courier apps, captured from simulator builds after parity updates.

## Integrity

- File checksums: `SHA256SUMS.txt`
- Archive package: `../ios-acceptance-v3.zip`

## Customer App Tabs

- `ios-customer-live-v3.png` -> `Canli Takip`
- `ios-customer-order-v3.png` -> `Siparis Ver`
- `ios-customer-history-v3.png` -> `Gecmis`
- `ios-customer-wallet-v3.png` -> `Cuzdan`
- `ios-customer-support-v3.png` -> `Destek`

## Courier App Tabs

- `ios-courier-navigation-v3.png` -> `Navigasyon`
- `ios-courier-pod-v3.png` -> `POD / Imza`
- `ios-courier-earnings-v3.png` -> `Kazanclar`
- `ios-courier-profile-v3.png` -> `Profil`

## Capture Notes

- Source simulator: `iPhone 17 Pro`
- Bundle IDs:
  - `com.qervon.ios.customer`
  - `com.qervon.ios.courier`
- Launch arguments used for acceptance mode:
  - `--qervon-acceptance-mode`
  - `--qervon-customer-tab=<tab>`
  - `--qervon-courier-tab=<tab>`
