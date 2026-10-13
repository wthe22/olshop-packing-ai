# Phase 3 — Android camera spike

Answers one question before the app is built: does phone-camera scanning of the printed labels
work fast and reliably enough?

Read: [05-app-architecture](../../design/05-app-architecture.md) (stack, scan pipeline).

- [ ] **3.1** Install Android Studio and SDK; log in [01-setup](../../development/01-setup.md).
- [ ] **3.2** Create `android/` (Kotlin, Compose, `minSdk 26`, application id
  `com.wthe22.packingchecker`), modules `app` + `core` (pure Kotlin).
- [ ] **3.3** One screen: CameraX preview + ML Kit (bundled), QR + Code 128; shows each decoded
  value with time; 2 s rule; lens switch button; beep on new code.
- [ ] **3.4** Install on the owner's phone (model and Android version → [01-setup](../../development/01-setup.md) log).

**Owner review:** [review-1.md](review-1.md).
