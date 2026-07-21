# APK-Style Model Profile Refactor Implementation Plan

> **For agentic workers:** Execute this plan task-by-task with verification checkpoints.

**Goal:** Tách engine protocol dùng chung khỏi profile/data theo model, giống cách APK nạp cấu hình model rồi chọn protocol family tương ứng.

**Architecture:** Rust giữ các protocol family và router chung. Catalog model, capability, EQ preset, môi trường ANC và ảnh được gom thành profile data theo model; BP1 Pro chỉ còn adapter cho family đã xác minh. Frontend đọc catalog/profile thay vì hard-code BP1 và EQ toàn cục.

**Tech Stack:** Rust/Tauri, serde JSON, TypeScript/SolidJS, Vite.

---

### Task 1: Chuẩn hóa schema profile model

**Files:**
- Create: `src-tauri/src/catalog/mod.rs`
- Create: `src-tauri/src/catalog/types.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/catalog/mod.rs`

- [ ] Tạo schema `ModelProfile`, `EqProfile`, `EqPresetProfile`, `NoiseProfile`, `ImageProfile` bằng serde.
- [ ] Đưa các field đang nằm trong `ModelInfo`/`DeviceProfile` về profile data, vẫn giữ `DeviceProfile` làm runtime projection.
- [ ] Thêm loader catalog tĩnh bằng `include_str!` và validate duplicate model id/dictSort.
- [ ] Test profile BP1 Pro có 12 preset, 8 band custom, môi trường `[101,102,103,108]`.

### Task 2: Chuyển data BP1 Pro ra khỏi code protocol

**Files:**
- Create: `src-tauri/catalog/models/bass-bp1-pro.json`
- Create: `src-tauri/catalog/eq/bass-bp1-pro.json`
- Create: `src-tauri/catalog/images/README.md`
- Modify: `src-tauri/src/protocol/models.rs`
- Modify: `src-tauri/src/protocol/router.rs`

- [ ] Đưa model id, capability, transport, noise environment và support level vào JSON.
- [ ] Đưa 12 preset EQ, dictSort, band frequencies và custom constraints vào JSON.
- [ ] `models.rs` đọc catalog thay cho `all_models()` hard-code.
- [ ] `router.rs` chỉ nhận `ProtocolFamily`, không đọc tên model hay hard-code data EQ.

### Task 3: Giữ BP1 adapter là protocol family

**Files:**
- Rename/Modify: `src-tauri/src/protocol/bp1_pro.rs` → `src-tauri/src/protocol/families/bp1.rs`
- Create: `src-tauri/src/protocol/families/mod.rs`
- Modify: `src-tauri/src/protocol/mod.rs`
- Test: `src-tauri/src/protocol/families/bp1.rs`

- [ ] Đổi tên adapter thành `bp1` để thể hiện đây là family dùng cho BP1 Pro/Ultra và model tương thích.
- [ ] Giữ frame builder/decode đã xác minh, không copy data model vào adapter.
- [ ] Router map `protocolFamily = bp1` vào adapter.
- [ ] Giữ test BA43, BA31, BA10 và AA responses.

### Task 4: Nối frontend với profile runtime

**Files:**
- Modify: `src/lib/ble.ts`
- Modify: `src/lib/device.ts`
- Modify: `src/lib/eq.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/EqPanel.tsx`

- [ ] Expose `modelProfile` và `eqProfile` từ runtime bridge.
- [ ] EQ UI dùng preset/band data của profile; fallback chỉ dùng khi profile không có.
- [ ] Custom EQ dùng band/frequency/max custom từ profile, không hard-code 8 trong component.
- [ ] Giữ popup spatial/EQ và lưu custom theo device.

### Task 5: Quy trình bổ sung model mới

**Files:**
- Create: `docs/model-catalog.md`
- Modify: `docs/superpowers/specs/2026-07-21-apk-protocol-integration-design.md`

- [ ] Ghi rõ thêm model chỉ cần thêm JSON profile/EQ/image reference.
- [ ] Nếu protocol family mới, thêm một adapter và đăng ký family trong router.
- [ ] Đánh dấu `verified`, `experimental`, `scanOnly`; không tự bật command nguy hiểm cho model chưa xác minh.

### Task 6: Verification

- [ ] Chạy `npm.cmd run build`.
- [ ] Chạy `cargo test --lib` nếu Rust toolchain có sẵn; nếu không, ghi nhận blocker rõ ràng.
- [ ] Kiểm tra catalog không duplicate id/dictSort và BP1 Pro vẫn tạo đúng frame.

