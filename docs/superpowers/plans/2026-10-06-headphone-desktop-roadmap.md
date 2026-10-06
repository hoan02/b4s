# B4S: kế hoạch hoàn thiện desktop companion dành cho tai nghe

Ngày: 2026-10-06. Trạng thái: roadmap đang được triển khai; tiến độ, kiểm chứng và các cổng phần cứng theo dõi tại [`docs/headphone-desktop-progress.md`](../../headphone-desktop-progress.md). Chỉ đạo cập nhật ngày 2026-10-06 yêu cầu xây kiến trúc mới sạch, không giữ đường runtime fallback/legacy. Ghi chú cũ về facade tương thích trong lúc chuyển đã bị thay thế; chỉ giữ phép chuyển dữ liệu một lần khi cần bảo toàn cài đặt người dùng.

## 1. Quyết định đã chốt và mục tiêu

- Windows là nền tảng phát hành hoàn chỉnh đầu tiên; macOS/Linux được chuẩn bị ở ranh giới transport và triển khai sau.
- Ưu tiên điều khiển tai nghe ngoại tuyến. Catalog/ảnh có thể được cập nhật khi người dùng chủ động yêu cầu.
- Cloud/AI, tài khoản Baseus và OTA được nghiên cứu trong inventory nhưng triển khai ở giai đoạn sau.
- Phần cứng hiện có đã xác nhận: **BP1 Ultra, có điện thoại Android chạy app Baseus/lấy Bluetooth log**. Firmware, Android version và Bluetooth adapter Windows sẽ ghi khi bắt đầu capture.
- Giữ SolidJS, Tauri, Rust và hệ thống locale hiện tại; không cần chuyển framework.
- Tái tạo hành vi phục vụ tương thích thiết bị bằng code B4S. APK/decompiled source/firmware không được đưa vào Git hay bundle.
- Phạm vi đã chốt: earbuds, over-ear, neckband và open-ear; loại speaker và thiết bị không thuộc nhóm headphone/audio companion.
- UI đã chốt: desktop utility gọn, dễ dùng, theo trải nghiệm Windows; không sao chép layout Baseus, giữ tương đương chức năng khi thiết bị hỗ trợ.
- Chưa mua/mượn family khác ngay. Chọn sau M0 theo khác biệt transport, protocol family, framing và codec; ưu tiên đại diện family mới thay vì tên gần BP1.
- Đóng cửa sổ mặc định thu nhỏ xuống system tray; Quit là thao tác riêng. Auto-start và auto-reconnect là opt-in và tắt mặc định.
- Có Experimental mode riêng, tắt mặc định. Chỉ expose feature đã có source evidence, serializer/decoder hoặc fixture/replay tương ứng, và đã xác định giới hạn an toàn. Mode không bypass backend capability/validation.

Mục tiêu sản phẩm là một ứng dụng có thể nhận diện, kết nối, đọc trạng thái,
điều khiển, khôi phục kết nối và lưu tùy chọn đúng theo **model + firmware +
transport + nền tảng**. Mỗi chức năng phải có trạng thái hỗ trợ và bằng chứng
riêng. Có tên trong catalog không đồng nghĩa với có thể điều khiển.

“Dịch ngược toàn bộ” nghĩa là thống kê và phân loại toàn bộ hành vi liên quan
tai nghe trong phiên bản APK này, kể cả hành vi chưa port được; không hứa mọi
dịch vụ cloud hoặc tính năng điện thoại sẽ hoạt động trên Windows.

## 2. Hiện trạng được kiểm tra trong repo

| Thành phần | Hiện trạng | Việc cần xử lý |
|---|---|---|
| `src-tauri/src/ble.rs` | BLE runtime vẫn gom scan/connect/write/notify/poll/state/mock; discovery và handshake mới chỉ tách một phần | Tiếp tục chuyển quyền sở hữu runtime sang transport/session/command/state modules; xóa entrypoint và state legacy theo lát cắt |
| `src/App.tsx` | Khoảng 701 dòng, chứa nhiều signal và handler nghiệp vụ | App shell gọn; state/feature controller riêng, có ràng buộc theo session |
| Catalog công khai | 174 model, 129 audio, đã lấy CN/US/EU không cần token | Danh mục audio chứa 5 model chỉ thuộc nhóm loa; scope tai nghe hiện có 124 ứng viên trước kiểm tra phân loại sâu |
| Profile đã review | BP1 Pro có BLE profile; BP1 Ultra là profile scan-only do transport chưa được capture | Hoàn thiện transport/framing/feature constraints bằng evidence theo firmware |
| `protocol/models.rs` | Registry tĩnh và substring identity resolver đã bị xóa; profile review cấp quyền điều khiển, metadata công khai chỉ scan-only | Hoàn thiện explicit ID migration và ma trận profile; không thêm model bằng heuristic |
| `protocol/router.rs` | Chỉ còn family BP1 và Unknown; generic Baseus AA/BA route đã bị xóa | Mỗi family tiếp theo cần codec/decoder riêng và evidence trước khi thêm |
| `protocol/wrap_v2.rs` | Lựa chọn framing còn dựa chuỗi tên | Framing nằm trong transport/protocol profile, kiểm chứng theo từng model |
| Kết nối | Chỉ dùng service/characteristic/framing khai báo trong profile; không dò characteristic generic hoặc thử sibling tự động | Hoàn thiện readiness/readback lifecycle theo reply; không dựa write success |
| `device/initialization.rs` | Startup query chưa bao phủ toàn bộ state; unknown vẫn có kế hoạch query pin | Kế hoạch init phải theo negotiated capability; unknown không gửi packet đoán |
| Pin | Có heuristic loại dữ liệu rất thấp và salvage ở nhiều lớp | Xác minh 0–4%, một bên vắng mặt, hộp sạc, sạc; không biến unknown thành 0% |
| EQ | FE còn constants chung; profile có curve cố định | Tách preset ID, wire index, filter data, preview curve và firmware constraints |
| Tauri command | Một số giá trị lạ mặc định thành preset/chế độ hợp lệ | Enum/validation có lỗi rõ ràng; không mặc định Off/Balanced khi dữ liệu sai |
| State UI | Một số handler cập nhật trước khi được device xác nhận | Phân biệt requested/pending/confirmed/failed; snapshot có revision và session ID |
| GATT | UUID/config do profile cấp; connect/subscribe/disconnect dùng UUID tường minh, scan entries cùng tên được giữ riêng | Audit UUID theo hardware trace; không dò alternate characteristic hoặc tự đổi sang entry khác |
| Kiểm thử | Baseline lượt trước: 65 Rust + 9 Python, typecheck/build/check qua | Giữ baseline và thêm replay, transport giả lập, integration/UI, hardware matrix |

Nguồn baseline: `docs/architecture.md`, `docs/model-catalog.md`, các file trên,
và `docs/re/findings-2.17.0.1.md`. Không dùng lại giả định hỗ trợ từ mô tả cũ
thay cho kiểm tra code và phần cứng.

## 3. Phạm vi chức năng phải điều tra

Các dòng dưới là backlog kiểm kê. Chức năng chưa có bằng chứng đầy đủ được
đánh dấu candidate trong ma trận; không tự bật capability vì tìm thấy tên lớp.

| Nhóm | Hành vi cần khảo sát | Desktop offline đầu tiên | Bằng chứng/điểm bắt đầu |
|---|---|---|---|
| Discovery | Name, manufacturer data, service UUID, serial, alias, color/model ID, entry audio/control | Có | `DeviceManager`, add/search-device flow, advertisement parser |
| Kết nối | GATT/SPP/vendor path, pairing, subscribe, MTU, queue, handshake, init, reconnect | Có theo transport được xác nhận | `BluetoothDataWriteManager`, connector/manager theo SDK |
| Identity | Model, firmware, hardware revision, serial, trạng thái hai tai | Có | Identity query/DTO/callback; không đoán từ tên |
| Pin | L/R/case, charging, single-bud, low battery, freshness | Có | `BA02`, `BA27`, consumer state theo family |
| ANC | Off/ANC/adaptive, môi trường, level/range, nhớ chế độ | Có theo model | `BleCommandUtil`, noise views/beans, firmware guards |
| Transparency | Full/voice, level và tương tác ANC | Có theo model | Noise builders và UI conditions |
| EQ preset | Preset server/dictionary/local fallback, ID/sort/command, interaction codec | Có | EQ activities, `getModelParams`, dictionary consumers |
| Custom EQ | Số band/filter, frequency/Q/gain/type, slot, CRC, reset/save/readback | Có khi codec đã xác minh | Custom EQ flow và serialization thực tế |
| Bass | Enable/level, dải giá trị, compact/extended payload | Có | `BA53/54`, `EarSoundSettingViewModel` |
| Spatial | Mode, query, capability negotiation, phụ thuộc EQ/codec | Có khi là xử lý trong tai nghe | `PanoramicSoundViewModel`, `SpatialSoundManger` |
| Game | Query/set, đồng bộ khi đổi ở app khác | Có | `BA23/24`, `GestureBleManager` |
| Codec | LDAC/LHDC/khác nếu có, polarity, restart/reconnect, multipoint conflict | Chỉ phần cấu hình tai nghe | `LdacSettingActivity`, codec guards; codec OS là vấn đề riêng |
| Gesture | Left/right, tap/double/triple/hold, action IDs, mode-dependent mapping | Có | `BA21/22`, gesture config/dictionary và consumer |
| In-ear | Query/set, auto-pause, single-bud dependency | Có | `BA25/26`, ear-check callbacks |
| Multipoint | Query/set, kết nối thứ hai, codec conflict, reboot | Có nếu command local | `SmartConnectActivity`, firmware guards |
| Find buds | L/R/both, stop, timeout, disconnect/cancel | Có | `BA10`, find-ear flow; cảnh báo âm thanh trước khi start |
| Hearing protection | Enable, giới hạn theo dB/level/FF, query/ACK/state | Có nếu setting local | `AA93/BA94`, `HearingProtectionPopWindow`; không đoán dải 0–3 |
| Settings thiết bị | Nickname, prompt language/volume, auto-off, indicator/light nếu có | Candidate | Ear settings, `QuickCallSettingActivity`, bean/command mapping |
| Firmware info | Đọc phiên bản, check metadata | Đọc phiên bản offline; check online chủ động | Version query; không gửi OTA trong discovery |
| Reset/binding | Restore settings, clear pairing, local/cloud bind, ownership state | Tách riêng, cần xác minh và xác nhận thao tác | Bind/init consumers; không coi `#InitState:` là universal |
| SoundFit/Mimi | Hearing test, calibration, algorithm, profile import/apply, dữ liệu riêng tư | Nghiên cứu; chỉ enable khi có cơ sở kỹ thuật | Mimi SDK và personalization flow, local/native/cloud dependency |
| Cleaning | Rung/âm thanh vệ sinh, guide-only hay command thực | Candidate; default tắt khi chưa xác minh | Ear clean activities, data callbacks |
| Remote camera/call | Gesture triggers, điện thoại hay tai nghe xử lý | Candidate hoặc không áp dụng trên Windows | `ControlPhotographyActivity`, quick-call flow |
| AI tuning/translation/record | Audio capture, session/token/backend, entitlement | Sau offline release | AI tuning/translation/record classes và API |
| OTA | Vendor DFU, dual-bud coordination, signature, resume, recovery | Sau offline release; discovery vẫn nghiên cứu | OTA/BES/SDK branches; có SDK không chứng minh model dùng SDK đó |

Inventory phải thêm mọi chức năng mới phát hiện ngoài bảng này. Luôn ghi một
kết luận: implemented / planned / phone-only / cloud-dependent / unresolved /
unsupported on platform, cùng lý do và bằng chứng.

## 4. Phương pháp dịch ngược: từ chức năng đến bằng chứng

### 4.1. Hồ sơ nguồn và công cụ

1. Ghi hash XAPK/APK/split, package/version, tool version và lỗi từng lượt chạy;
   inventory native/resource lưu paths, sizes, hashes trong `.tmp`, không commit
   binaries/source.
2. Giữ riêng decompile, resources, strings, native library inventory và server snapshots trong thư mục bị ignore.
3. Phân tích APK chính và split ARM64: `.so`, asset JSON, bundle/script, config; không chỉ DEX strings.
4. Lập danh sách 508 lỗi JADX; ưu tiên lỗi trong call path tai nghe, không mất thời gian sửa toàn bộ SDK không liên quan.
5. Với method lỗi: kiểm tra DEX/smali hoặc engine thứ hai, rồi so lại control flow; không tin pseudocode thiếu branch.
6. Kiểm tra reflection, obfuscated class, JNI, callbacks, byte builders và dynamic dictionary resolution.
7. Chỉ ghi tài liệu tự viết, packet fixture tối thiểu đã scrub và implementation B4S vào Git.

### 4.2. Một “hồ sơ chức năng” bắt buộc có gì

Mỗi feature được điều tra bằng chuỗi:
UI entry → điều kiện model/firmware → source cấu hình → request builder →
transport/framing → callback/ACK → state decoder → UI effect → persistence.

| Trường hồ sơ | Nội dung bắt buộc |
|---|---|
| Identity | Feature ID, APK version, model/family, firmware scope, platform |
| Entry | Activity/Fragment/ViewModel hoặc server feature descriptor |
| Guards | Tất cả điều kiện model, firmware, single/dual ear, codec, ANC, account |
| Dependencies | Local/SDK/native/cloud; cần mic/audio stream/OS integration hay không |
| Request | Opcode, byte offsets, endianness, checksum, length, enum/range, target side |
| Response | State response riêng với write ACK; success/error/busy, readback query |
| Execution | Timing, request correlation, serialize, retry/idempotency, reconnect, cancellation |
| UX | Disabled/hidden/pending/error, interaction conflict, persistent/local-only state |
| Evidence | File/class/method/line ở dump local, capture ID, sample TX/RX và độ tin cậy |
| Result | Khả năng port, B4S gap, test vector, chưa biết gì, điều kiện để enable |

Ví dụ hồ sơ đầu tiên: hearing protection đã biết `AA93 enabled level`, ACK ngắn
`AA94 01`; tiếp tục kiểm dải level, init query, firmware guards và device readback.
Không sử dụng bộ decoder đó cho family khác trước khi có evidence.

BP1 Ultra được dùng làm thiết bị tham chiếu đầu tiên. Cần ưu tiên capture
startup/transport: nguồn đã phân tích cho thấy đường Classic BT/SPP và
789C+CRC có vai trò trên Ultra, trong khi B4S hiện chủ yếu là BLE GATT.
Đây là giả thiết phải đối chiếu với firmware thực tế của thiết bị, không phải
lý do tự ép mọi Ultra hoặc mọi chức năng qua một transport chưa được test.

### 4.3. Ma trận model/family và trạng thái bằng chứng

- Ma trận tách: discovery identity, transport, framing, handshake, firmware, từng feature, từng platform.
- Các mức bằng chứng: string candidate → traced source → replay-tested → hardware-tested.
- Mức runtime feature: unavailable / experimental / verified; kèm read/write support và constraints.
- Không nâng toàn bộ model thành verified sau khi xác nhận được một packet hoặc một feature.
- Match alias phải có provenance; edition/Plus/Ultra không tự dùng profile của tên ngắn hơn.
- Gộp family khi packet table và lifecycle thực sự giống; tách variant khi ACK/enum/framing khác.
- Chọn thiết bị đại diện bằng transport/codec khác nhau, không theo độ giống tên thương mại.
- Không thể xác nhận family ngoài BP1 bằng phần cứng BP1; replay chỉ xác minh implementation của giả thiết.

### 4.4. Kiểm chứng động bằng thiết bị

1. Dùng app Baseus trên điện thoại của người dùng nếu có; ghi model/firmware/OS/app version.
2. Lấy Bluetooth HCI log hoặc capture phù hợp với transport; lựa chọn phương pháp theo nền tảng sau xác nhận Android.
3. Mỗi capture chỉ đổi một biến; đọc state trước/sau, lưu timeline thao tác và reply.
4. Kiểm tra bật/tắt, biên enum/range hợp lệ, reconnect, một tai trong hộp, codec conflict.
5. So sánh trên B4S với cùng state đầu; xác nhận bằng readback và hành vi thiết bị.
6. Đổi setting trên điện thoại/tai nghe để kiểm tra B4S nhận notify ngoài lệnh của mình.
7. Mọi thử nghiệm có âm thanh lớn/reset/OTA phải là thao tác chủ động, có điều kiện dừng tương ứng.
8. Capture chứa địa chỉ/serial/audio/token giữ riêng; export hỗ trợ phải review và redact.

### 4.5. Bộ capture đầu tiên cho BP1 Ultra + Android

| Capture | Thao tác | Mục tiêu |
|---|---|---|
| U01 | Mở app khi tai nghe đã pair, kết nối mới, không đổi setting | Xác định transport, service/channel, subscribe, handshake, init và startup queries |
| U02 | Ngắt/kết nối lại từ app và tắt/bật tai nghe | Xác định state phục hồi, identifier ổn định, timing và query nào thực sự cần |
| U03 | Normal→ANC→Transparency; thay từng level app cho phép | Mode/level encoding, ACK/readback, adaptive khác custom level |
| U04 | Chọn từng preset app thật sự hiển thị; thử custom nếu app hỗ trợ | Preset source/wire index, payload filters, tránh dựa vào server array rỗng |
| U05 | Game/spatial/bass từng chức năng một nếu app hiển thị | Polarity/range/conflicts và state notify |
| U06 | Đổi gesture mỗi bên; in-ear nếu app hỗ trợ | Mapping query/set, event side, action enum và persistence |
| U07 | Multipoint/codec theo lựa chọn app; ghi restart/reconnect | Transport có đổi không, enum/ACK, dependency và ảnh hưởng connection |
| U08 | Find left/right/both và stop, chỉ sau khi tháo tai nghe | Target bytes, timeout/stop semantics, reply availability |
| U09 | Một tai trong hộp, hai tai ra ngoài, hộp sạc đang sạc | State topology, pin missing/charging và notifications |
| U10 | Setting đổi ở điện thoại trong khi B4S quan sát bằng phương án khả thi | Xác định push notifications và synchronization; không giả định cho phép hai control clients đồng thời |

Mỗi lượt ghi firmware/app/Android/transport, trạng thái đầu, thao tác có
timestamp, TX/RX, kết quả nhìn thấy, unknowns. Không gửi các giá trị ngoài
những lựa chọn hợp lệ của app chính thức. Chưa biết firmware không cản chuẩn
bị lab nhưng phải có trước khi kết luận profile được xác nhận.

## 5. Kiến trúc đích và trách nhiệm

### 5.1. Backend Rust

```text
src-tauri/src/
  api/                 Tauri commands, typed DTOs, error mapping, versioned events
  discovery/           advertisement parser, identity resolver, scan deduplication
  catalog/             metadata, reviewed profiles, validation, capability resolver
  transport/
    mod.rs             transport interfaces; no UI/state business rules
    ble_gatt/          adapter discovery, connect, subscribe, write, disconnect
    classic_spp/       Windows implementation only when established by evidence
    mock/              deterministic scripted transport
  protocol/
    framing/           bare AA/BA, 789C and later verified formats
    families/          codec implementations; encode/decode/query plans
    types/             domain commands/events; independent of Tauri
  session/
    manager.rs         session ownership, generation, task lifetimes
    lifecycle.rs       connect/init/ready/reconnect state machine
    executor.rs        request queue, deadlines, correlation, retry policies
    state.rs           confirmed device snapshot, revision, availability/freshness
  features/            noise, eq, gestures, connectivity, find, hearing, settings
  persistence/         device records, user preferences, migrations, optional cache
  diagnostics/         bounded logs, scrubbed replay/export, error explanations
```

Đây là cấu trúc mục tiêu. Chuyển toàn bộ luồng BLE và command sang ranh giới mới
theo lát cắt có kiểm chứng; mỗi lát cắt phải xóa đường cũ khi đường mới thay
thế xong. Không dual-run, silent fallback, hoặc giữ facade legacy sau migration.
Quy trình chuyển dữ liệu người dùng phải versioned, explicit và chạy một lần;
model/firmware/transport không có profile tường minh thì unavailable.

| Ranh giới | Quy tắc |
|---|---|
| Transport | Mang byte và connection events; không biết ANC/EQ hay model label |
| Framing | Length/checksum/chunk/reassembly; không tự chọn mode theo substring tên |
| Family adapter | Biến domain command thành packet, packet thành typed event; có variant explicit |
| Session executor | Serialize khi transport yêu cầu; theo dõi ACK/state/deadline/cancel |
| Feature service | Kiểm availability/constraint/conflict trước gửi; query readback khi cần |
| Catalog | Metadata khác reviewed control profile; remote data không tự bật writable opcode |
| Persistence | User preference khác confirmed device state; scope theo device identity |
| API | DTO camelCase, typed enums, error codes; invalid input không thành lệnh mặc định |

### 5.2. Model và capability schema

Schema v2 tách tối thiểu:

- `identity`: canonical ID, server model aliases, advertisement rules, product category, evidence refs.
- `presentation`: product label, variant/color, local/cache/remote/fallback image provenance.
- `transport`: BLE/SPP/vendor adapter, UUID/channel, write mode, MTU/chunking và platform support.
- `protocol`: family, explicit framing, handshake/init sequence, opcode/ACK variant.
- `firmwareRules`: dải firmware/known exclusions, version comparison strategy đã xác minh.
- `features`: per-feature read/write, enum/range, constraints, dependencies, confirmation mode, evidence/support.
- `eq`: supported presets với stable IDs + wire IDs; custom filter schema riêng; visualization riêng.
- `stateQueries`: init/refresh/query cadence và dependency; unknown không có mặc định gửi BA.

Capability hiệu lực là phần giao của profile đã review, firmware rules,
transport/platform khả dụng, capability device báo và dependency hiện tại.
Metadata server chỉ enrich UI. Firmware chưa xác định thì chỉ dùng feature
được xác nhận không phụ thuộc firmware; phần còn lại hiện chưa xác minh.

Profile loader phải reject duplicate identity, family/variant lạ, enum/range
sai, UUID không hợp lệ, feature thiếu serializer/query confirmation cần thiết,
unsupported combination và profile tự nhận verified nhưng thiếu evidence refs.

### 5.3. Session và command lifecycle

Connection lifecycle dự kiến: disconnected → scanning → connecting →
discovering → subscribing → initializing → ready; có degraded/reconnecting/
failed khi từng điều kiện đã được phân biệt. Không đưa raw opcode vào luồng UI.

- Một session actor/task sở hữu peripheral, queue và state của một device.
- Mỗi session có generation ID; reply/event từ connection cũ bị loại.
- ACK write, confirmed state và successful OS write là ba tín hiệu khác nhau.
- Nếu wire không có request ID, correlation dùng opcode/target/sequence và thời gian trong một queue serialized.
- ACK trễ của lệnh đã timeout không được xác nhận lệnh mới cùng opcode; dùng drain/readback/quarantine tùy family.
- Chỉ retry lệnh idempotent có policy đã xác minh; reset/bind/find-start/OTA không được retry mù.
- Coalesce slider requests theo semantics; lệnh stop tìm tai nghe có ưu tiên nhưng không phá queue correlation.
- Khi đổi thiết bị/disconnect/sleep: cancel queue/timer/subscription, kết thúc các tác vụ stream theo thứ tự.
- Reconnect có backoff hữu hạn, người dùng có thể cancel; tránh liên tục giành tai nghe với app điện thoại.
- Snapshot gồm sessionId/revision, model/firmware, availability, value/unknown, freshness và pending operation.
- Pin thật 0% khác chưa có dữ liệu; không dựa thay đổi giá trị để biết query đã được trả lời.
- Notification batching/out-of-order phải được framing xử lý trước feature decoder.

### 5.4. Frontend SolidJS

```text
src/
  app/                 shell, navigation, providers, global preferences
  bridge/              typed Tauri commands/events, subscription cleanup
  stores/              session/device snapshot; pending command reconciliation
  features/
    devices/           scan, pairing, saved devices, model support
    overview/          battery, connection, quick controls
    listening/         ANC/transparency/spatial/game/bass
    equalizer/         presets, custom filters, preview, saved profiles
    gestures/          per-side/action mapping, in-ear detection
    connectivity/      multipoint, earbud codec settings, constraints
    device-settings/   prompt/language/power/identity/reset
    diagnostics/       status, guided recovery, export
  components/ui/       shared controls, dialogs, pending/error states
  lib/                 small utilities; no parallel source of device truth
  locales/             retain the five bundled locales
  styles/              semantic tokens, common UI and feature styles
```

- FE không gửi arbitrary opcode hoặc tự tính capability từ model name.
- Device snapshot từ backend là source of truth; desired value có pending UI riêng.
- Store keyed theo device/session, không mang EQ/ANC của thiết bị A sang B.
- Listener registration/cleanup có một owner; reconnect không nhân đôi subscriptions.
- Thay constants `EQ_BANDS/EQ_LABEL/presetSort` bằng resolved model schema.
- Dùng preset ID cho intent, backend resolve wire index; preview curve không được dùng nhầm làm payload.
- Không có number gain grid nào dùng chung cho mọi family nếu cấu trúc filter khác nhau.
- Preferences như theme/locale tồn tại riêng; EQ cá nhân migrate có version và device/profile key.

## 6. Kế hoạch UI/UX

### 6.1. Information architecture

| Màn hình | Nội dung chính | Điều kiện và trạng thái |
|---|---|---|
| Thiết bị | Saved devices, scan results, tín hiệu, model/support, kết nối | Phân biệt audio entry/control entry; unknown có hướng dẫn, không nút điều khiển giả |
| Tổng quan | Tên/ảnh, L/R/case, charging, connection, mode hiện tại, quick actions | Unknown pin dùng dấu chưa có dữ liệu; stale có timestamp/trạng thái |
| Âm thanh | ANC/transparency, bass, spatial, game | Chỉ feature phù hợp; conflict có lý do và thao tác giải quyết |
| Equalizer | Preset đúng model, custom editor, slots, apply/reset/readback | Số band/filter theo schema; đang áp dụng khác đã xác nhận |
| Điều khiển | Gesture L/R, in-ear, các action thật sự hỗ trợ | Không giả định mọi tai có mọi gesture hoặc mọi action |
| Kết nối | Multipoint, cấu hình codec tai nghe, trạng thái transport | Phân biệt codec tai nghe với codec audio của Windows |
| Cài đặt tai nghe | Device identity, firmware, prompt, auto-off, rename/reset | Những mục chưa có bằng chứng được ẩn hoặc giải thích rõ |
| Cài đặt app | Theme/locale, startup/tray/reconnect policy, catalog update, privacy | Catalog refresh là thao tác chủ động; offline vẫn dùng được |
| Chẩn đoán | Bước kết nối đang lỗi, support matrix, guided recovery, export | Thông tin kỹ thuật sâu ở khu vực riêng, không lẫn vào controls nghe nhạc |

Navigation tối đa 5–6 mục cấp đầu; gộp các mục phụ vào cài đặt/section để
không tạo một sidebar dài theo mọi feature trong APK. Thiết kế cho một thiết bị
đang điều khiển trước, nhưng identity/store không khóa kiến trúc vào một device.

### 6.2. Quy tắc control và thông báo

- Feature không có trên model: không render control hoạt động.
- Feature có nhưng chưa được xác minh: mục hỗ trợ/experimental rõ ràng, không gắn nhãn verified.
- Experimental mode tắt: không expose control experimental; support matrix vẫn có thể giải thích trạng thái nghiên cứu. Bật mode chỉ expose feature đã đạt điều kiện evidence/implementation/fixture và safety review.
- Experimental mode không mở raw opcode console, không biến scanOnly/unknown thành writable và không bỏ qua giới hạn model/firmware/transport/range/dependency phía backend.
- Feature tạm unavailable do một tai/codec/firmware/connection: disable với lý do dễ hiểu.
- Lệnh đang gửi: giữ confirmed value, hiện desired/pending; timeout có retry theo policy/readback.
- Error sát control; toast chỉ bổ sung, không là nơi duy nhất thông báo lỗi.
- Đổi codec cần restart: dialog giải thích mất kết nối, progress reconnect, trạng thái kết quả.
- Find: xác nhận tháo tai nghe, chọn bên, timer/stop luôn truy cập được, không tự resume sau reconnect.
- Reset/clear pairing: nói rõ phạm vi bị xóa, yêu cầu thao tác xác nhận riêng.
- App không ready: các lệnh bị khóa; không suy ra ready từ một write thành công.

### 6.3. Visual system và accessibility

Giữ hướng utility desktop rõ ràng, sáng/tối, typography hệ thống đọc được
tiếng Việt. Skill UI/UX đã được dùng để khảo sát; các gợi ý hero/CTA/font sci-fi
không phù hợp companion này nên không đưa vào thiết kế.

- Tokens cho background/surface/border/text/accent/status/focus; không rải màu trực tiếp.
- Spacing thống nhất 4/8 px, hierarchy rõ; mức hỗ trợ phải có chữ, không chỉ màu.
- Keyboard đầy đủ, focus visible, Escape cho modal và focus return về trigger.
- Slider có numeric input và keyboard increment; không bắt buộc kéo chuột.
- Test dark/light, Windows 100/125/150/200% scaling, cửa sổ nhỏ và zoom/text scaling.
- Animation ngắn có ý nghĩa; reduced-motion; không block input vì animation.
- Image fallback có đúng provenance, giữ kích thước để tránh layout nhảy.
- NVDA/Windows screen reader cho core flows; tiếng Anh/Việt trước, giữ locale parity cho mọi string.
- Không copy layout điện thoại nguyên xi; chuyển control theo density và input desktop.
- Theo hành vi Windows: window controls, keyboard shortcuts, context menus, system dialogs và notification dùng cơ chế platform phù hợp. “Native experience” là tiêu chí UX, không yêu cầu đổi SolidJS/Tauri hay dùng hiệu ứng trang trí Windows ở mọi nơi.

### 6.4. Tray, startup và reconnect: chính sách đã chốt

- Close/window X ẩn cửa sổ vào tray; không gọi Quit hoặc phá session đang hoạt động chỉ vì cửa sổ bị ẩn.
- Tray có Open B4S, trạng thái thiết bị và Quit; mở lại cửa sổ không tạo session/instance/subscription mới.
- Quit là explicit shutdown: cancel queue/reconnect/timer, cố gắng dừng find nếu đang chạy với timeout hữu hạn, cleanup transport và thoát process. Không báo đã dừng âm thanh khi chưa có confirmation.
- Startup và auto-reconnect tắt trên clean install; chỉ bật khi người dùng chọn. Migration không tự chuyển preference false thành true.
- Auto-reconnect off: mất kết nối thì hiện trạng thái và chờ Connect chủ động. Khôi phục UI/state của link vẫn còn sau sleep không được dùng để lén bật policy reconnect.
- Nếu bật auto-reconnect: dùng session lifecycle/backoff/cancel đã định nghĩa; Quit, disconnect chủ động và đổi thiết bị phải dừng reconnect tương ứng.
- Thông báo lần đầu Close→tray giải thích app vẫn chạy và chỉ cách Quit; không lặp modal mỗi lần đóng.
- Không có tray khả dụng: giữ cửa sổ và cho lỗi/hướng thoát rõ ràng, không ẩn app thành process không thể mở lại.

Nghiệm thu: Close→Open lặp lại không tăng listener/task; Quit thoát process và
giải phóng connection; clean install không đăng ký auto-start; mất kết nối khi
auto-reconnect off không có connection attempt nền. Kiểm tra lại sau app restart,
preference migration và Windows sleep/resume.

### 6.5. Experimental mode: điều kiện expose và nghiệm thu

Feature experimental phải có hồ sơ source, model/firmware/transport áp dụng,
serializer/decoder hoặc fixtures/replay đủ cho hành vi tương ứng, giới hạn
giá trị/dependency/rủi ro đã biết và confirmation policy. Trường chưa biết
ảnh hưởng an toàn thì feature vẫn unavailable dù người dùng bật mode.

- Experimental preference được quản lý riêng và tắt mặc định; UI hiển thị nhãn experimental trên từng control được expose.
- Backend tự resolve capability/evidence eligibility và validate intent. FE không được gửi cờ `force` để nâng support hoặc bỏ qua constraints.
- Tắt mode không rollback tùy tiện setting đã áp dụng trên tai nghe; chặn lệnh experimental mới và giữ đường stop/cancel cần thiết cho operation đang hoạt động.
- Test gọi trực tiếp Tauri API khi mode off, feature unknown, firmware không phù hợp, range sai và dependency chưa đủ: không có packet write.
- Test khi mode on với feature đủ điều kiện: request vẫn qua validator/queue/confirmation như feature thường; failed ACK không thành success.
- Test scanOnly chỉ có metadata: bật mode vẫn không gửi lệnh hoặc kết nối control bằng transport phỏng đoán.

## 7. Catalog/metadata pipeline dành riêng tai nghe

1. Giữ raw responses local, snapshot allowlist public và provenance trong repo.
2. Tách “audio” khỏi “headphone”: phân loại theo categoryPath/type và allowlist schema đã review.
3. Catalog hiện có 5 speaker-only entries: AeQur 30 Air, DS10, N10, VO20, Sleep SK1; đưa ra khỏi headphone discovery mới sau khi review ảnh hưởng legacy.
4. Số 124 headphone candidates hiện tại không có nghĩa trùng 124 legacy entries; so sánh bằng full identity/aliases.
5. Đọc có chọn lọc `getModelParams`, dictionary và SKU/resource consumers để tìm feature configuration, không crawl mọi endpoint account/devicebind.
6. Cache response theo region/language/model/app/schema, timestamp/content hash/ETag nếu server cung cấp.
7. Rate limit, timeout và fail-preserve-snapshot; không refresh tất cả model mỗi lần app mở.
8. Enrichment ảnh là opt-in; validate URL/content type/size, bounded cache, fallback offline, không tải executable.
9. Feature config server không được thay serializer/protocol hay nâng quyền hỗ trợ runtime.
10. Chỉ import EQ khi đã hiểu từng trường và consumer: BP1 Pro server có 7 preset, Ultra query hiện rỗng; không tạo fallback capabilities từ response rỗng.
11. Catalog update có diff: additions/removals/aliases/category/metadata, source/version; review trước sửa profile đã xác nhận.
12. Model bị gỡ khỏi server vẫn giữ profile đã review và device user đã lưu.

## 8. Các gói công việc và điều kiện hoàn thành

| ID | Công việc cụ thể | Đầu ra | Phụ thuộc | Điều kiện đóng |
|---|---|---|---|---|
| P0.1 | Chụp baseline Git/build/test; ghi thay đổi đang có, không reset | Baseline report | Không | Repo hiện tại và 65+9 tests được bảo toàn hoặc giải thích thay đổi |
| P0.2 | Ghi firmware BP1 Ultra, Android/Windows version và Bluetooth adapter | Hardware manifest local | BP1 Ultra + Android đã xác nhận | Không chọn nhầm transport/profile |
| P0.3 | Ghi phạm vi product và policy đã được người dùng chốt | ADR scope | Đã chốt | Earbuds/over-ear/neckband/open-ear; exclude speaker; tray mặc định; startup/reconnect/experimental opt-in |
| P1.1 | Index entrypoints, model guards, SDK/native dependencies | Earphone feature inventory | P0 | In progress: transport plus gesture config loaders/guards indexed; [BP1 Pro feature matrix](../../protocol/bp1-feature-evidence-matrix.md) records current source/replay status and hardware limits. Broader feature/native call graph is incomplete; each screen/action still needs status and source pointer. |
| P1.2 | Triage JADX errors liên quan; extract resource/native inventory | Extraction coverage report | P1.1 | In progress: local hashes cover 7,916 resource and 106 native APK entries; gesture config schemas have targeted consumer notes. Priority failed call paths still require DEX/second-engine validation and remaining resource contents need review. |
| P1.3 | Lần theo family/transport/framing/firmware rules | Protocol-family matrix | P1.1 | Mỗi family candidate có evidence và unknowns |
| P1.4 | Trace server dictionary/model-param consumers | Configuration contract notes ([current dossier](../../protocol/model-parameter-consumers.md)) | P1.1 | Partial: consumer fields are classified for EQ, gesture imagery, guides, cleaning and shared sleep UI; profile dictionary is traced to account/avatar presentation. Dynamic headphone dictionary names, full field/guard coverage and configuration contract remain open. |
| P2.1 | Chuẩn hóa capture plan, local trace format, redaction | Capture guide + manifest schema | P0.2 | Có thể lặp lại cùng thao tác và so TX/RX |
| P2.2 | BP1 capture core features/init/reconnect | BP1 golden traces local | P2.1, Android/log path | Model+firmware+state đầu+timeline rõ |
| P2.3 | Replay harness và scripted fake transport | Offline test harness | P2.1 | Mô phỏng split/batched packet, timeout, late ACK, disconnect |
| P3.1 | Thay BLE discovery/GATT facade bằng transport/session mới | Transport interface + BLE adapter | P2.3 | In progress: demo behavior is isolated in `src-tauri/src/ble/mock.rs`; connection publication rejects vanished scan entries instead of synthesizing an experimental device; adapter initialization, scan start/stop and central-listener lifecycle have dedicated operation locks, with BLE state locks released during adapter awaits; failed scan stop preserves active state. Scan/connect still share the global runtime and need a session owner plus injected transport. |
| P3.2 | Tách framing/reassembly khỏi feature decoder | Frame codecs | P1.3, P2.3 | Captured/replay vectors đúng; malformed/CRC sai không đổi state |
| P3.3 | Session lifecycle/generation/cancel/reconnect | Session actor/state machine | P3.1 | In progress: `SessionRuntime` groups epoch, notification/battery worker ownership, and the per-session command executor; every production link-reset path joins workers outside the BLE lock and replaces the executor; central-listener start/Quit is serialized. A peripheral-owning session actor and remaining process-exit ordering remain open. |
| P3.4 | Queue/correlation/deadline/readback | Command executor | P3.2/P3.3 | In progress: bounded serialization/deadline execution is scoped to the active session and replaced on reset; request correlation and confirmed-state behavior still need completion. Write success must never be reported as device-confirmed. |
| P3.5 | Ưu tiên spike Windows SPP/vendor transport khi U01 xác nhận Ultra cần đường đó | ADR + nhỏ gọn prototype | P0.2/P1.3/U01 | Xác minh API Windows, RFCOMM/channel/pairing thật; BLE/SPP cùng profile có quy tắc explicit |
| P4.1 | Profile v2, validator, migrate BP1 Pro/Ultra explicit | Typed profiles + migration | P1.3/P1.4 | Không substring framing; firmware/UUID có provenance |
| P4.2 | Capability resolver/readiness/query planner | Resolved device schema | P4.1/P3.4 | Unknown không gửi queries đoán; backend reject unsupported intent |
| P4.3 | Device snapshot/error/event contract thay thế API cũ | DTOs/API v2 | P3.3/P4.2 | In progress: BLE scan/device/link DTOs are grouped in `src-tauri/src/ble/contracts.rs`; existing wire shapes are preserved. BLE error typing and event ownership remain open. |
| P4.4 | Scoped persistence, migrations, bounded diagnostic cache | Device/preferences storage | P4.3 | A/B không lẫn state; corrupt/old prefs có recovery. In progress: session-scoped BLE diagnostics retain at most 64 bytes per RX/TX preview; desktop preferences migrate v1 to v2 once and recover corrupt v2 to safe defaults; broader device-scoped storage recovery remains open. |
| P5.1 | App shell/navigation/session store | Solid feature structure | P4.3 | Mount/unmount/reconnect không nhân đôi listener |
| P5.2 | Devices/overview + accurate battery/connect feedback | First complete vertical slice | P5.1 | Scan→connect→identity→state→disconnect đúng trên BP1 |
| P5.3 | Shared controls pending/error/availability/a11y và Experimental policy | UI primitives + backend eligibility | P5.1/P4.2 | Light/dark, keyboard, scaling, locale; mode không bypass validation |
| P6.1 | ANC/transparency/game, constraints/readback | Listening feature slice | P4/P5, matching traces | Partial: snapshot v2 preserves AA34 mode/parameter/time; UI only selects profile-valid values from accepted observations. Matching device captures and firmware/hardware acceptance remain open. |
| P6.2 | EQ preset/custom/slot with model schema | Equalizer slice | P1.4/P6.1 | Wire index đúng; no invented filter curves; validate before write |
| P6.3 | Bass/spatial/codec/hearing constraints | Advanced sound slice | P1 traces/P3.4 | ACK khác state; codec restart UX/recovery verified |
| P6.4 | Gestures/in-ear, per-side mapping | Controls slice | Gesture evidence/P4 | Read current map, save, re-query, single-ear restrictions đúng |
| P6.5 | Multipoint/find/device settings | Connectivity/settings slice | Matching evidence | Stop/cancel/find safeguards; settings chỉ xuất hiện khi supported |
| P7.1 | Classify headphone-only catalog và one-time identity migration | Catalog + support matrix | P1/P4 | Speaker exclusions reviewed; old IDs map explicit một lần rồi legacy resolver bị xóa |
| P7.2 | Adapter của family kế tiếp | New-family replay + experimental profile | P1.3/P2.3/P3 | Codec/transport có tests; không đánh verified khi chưa có tai nghe |
| P7.3 | Hardware validation cho family kế tiếp | Per-feature support report | Có thiết bị/capture | Discover/init/read/write/reconnect thật trên firmware ghi nhận |
| P8.1 | Windows robustness và accessibility acceptance | Acceptance report | P5/P6 | Sleep/resume, BT off, app restart, OS scaling, cancellation qua |
| P8.2 | Signed installer/update + tray/startup/reconnect theo mục 6.4 | Windows release candidate | P8.1 | Close→tray, Open không duplicate session; Quit cleanup; startup/reconnect off mặc định; migration giữ opt-in |
| P8.3 | README/model matrix/diagnostics guide | Release documentation | P8.1/P7 | In progress: public candidate matrix and BP1 Pro feature evidence/limit matrix exist; each model/feature/platform still needs its acceptance evidence and limitations. |
| P9 | Cloud/AI/SoundFit/OTA và macOS/Linux | ADRs/backlog riêng | Offline release + scope review | Không chặn release offline; rollout riêng có hardware/recovery gates |

Không đổi toàn bộ code trước khi có lát cắt kiểm chứng. Mỗi PR thay một ranh
giới hoàn chỉnh và xóa implementation cũ trong cùng thay đổi khi đường mới qua
regression BP1. Không giữ legacy facade, generic fallback, hoặc hai runtime song
song để che thiếu profile; giữ user data bằng migration có chủ đích.

## 9. Thứ tự triển khai đề xuất

1. P0 + P1: chốt baseline, inventory toàn phần tai nghe, chuẩn bị hồ sơ BP1 Ultra và firmware.
2. P2: chuẩn hóa capture/replay; dùng Android thu U01/U02 trước để chốt transport/framing, rồi thu core feature traces.
3. P3 + P4: thay transport/session/queue/profile/DTO theo contract mới; cập nhật UI consumer trực tiếp và xóa bridge cũ trong lát cắt hoàn chỉnh.
4. P5: một luồng hoàn chỉnh devices→overview trên BP1; không bật mọi control sớm.
5. P6: âm thanh cơ bản → EQ → advanced sound → gestures/connectivity/settings; mỗi nhóm có trace/tests/UI.
6. P7: mỗi family mới có adapter/profile riêng và hardware validation khi có điều kiện.
7. P8: ổn định Windows và phát hành offline; P9 là dự án mở rộng sau đó.

Nếu không có trace cho một feature, không để cả dự án đứng chờ: hoàn tất
source dossier, unit/replay tests và UI descriptor ở experimental/unavailable,
rồi chuyển nhóm có bằng chứng đầy đủ. Các bước cần xác nhận hành vi phần cứng
vẫn để mở; không dùng test pass để đóng hardware gate.

## 10. Bộ kiểm thử và tiêu chí nghiệm thu

### 10.1. Offline automated

| Lớp | Cases bắt buộc |
|---|---|
| Discovery | Name alias exact/edition, manufacturer layout, unknown category, multiple control/audio entries |
| Catalog | Duplicate IDs/alias ambiguity, schema version, missing firmware, invalid UUID, malformed constraints |
| Framing | CRC, short/overlong frame, chunk boundaries, concatenation, wrong family, unknown opcode |
| Command | Boundary enums/ranges, unsupported feature, ordering, coalescing, timeout, late ACK, cancel |
| State | ACK-only không giả state, unchanged-value readback completion, stale battery, 0%, absent bud/case |
| Session | Drop before/after subscribe/init, disconnect while sending, generation change, stale reply/event |
| Persistence | Upgrade profile/ID mappings, old EQ shapes, missing/corrupt config, device A/B isolation |
| FE controller | Pending→confirmed/failed, external change, reconnect availability, listener cleanup |
| UI | Core flows with mock DTOs, keyboard/focus, missing value, long labels, five locale parity |
| Catalog fetch | Nullable fields, region merge, fail-preserve-cache, size limits, public metadata allowlist |

Giữ Rust unit/integration tests. Thêm frontend controller tests bằng framework
phù hợp Solid/Vite và UI smoke/E2E bằng mock bridge; lựa chọn dependency cụ thể
được kiểm tra compatibility khi triển khai, không cài trong lượt lập kế hoạch.

### 10.2. Windows hardware acceptance

- Fresh scan, saved reconnect, alternate Windows audio/control entries.
- Bluetooth off/on, device out of range, earbuds power off/on, one bud in case.
- PC sleep/resume, app restart, close window/tray/exit theo policy đã chốt.
- Apply mỗi feature/valid boundary, query readback, đổi setting từ điện thoại.
- Lệnh liên tiếp/slider kéo nhanh; không order sai hoặc timeout biến thành success.
- Codec/multipoint conflict, reconnect sau restart, không tự claim codec Windows đã đổi.
- Find start/stop từng bên; cancel/timeout/disconnect được ghi nhận đúng, không báo chắc “đã dừng” khi chưa có reply.
- Low battery, 0%, missing case; UI không tự giả 100% hoặc 0%.
- Firmware/model chưa hỗ trợ có lỗi hữu ích, không gửi generic fallback.

### 10.3. Definition of done cho một feature/model

1. Source dossier đủ TX/RX/guards/dependencies và unknowns được giải quyết hoặc ghi rõ.
2. Backend codec/validation/query/confirmation có tests và fixtures.
3. Capability resolver và init plan đúng model/firmware/platform.
4. UI hiển thị đúng supported/unavailable/pending/error; locale và a11y qua.
5. Persistence không biến requested state thành confirmed state sau restart.
6. Hardware test có manifest model/firmware/platform và evidence cho chức năng cụ thể.
7. Documentation/matrix cập nhật; verified chỉ cho phạm vi đã test.
8. `npx tsc --noEmit`, `npm run build`, `npm run check:i18n`, `cargo check`, `cargo test --lib` qua.

Model-level “hoàn chỉnh” yêu cầu tất cả feature **áp dụng được cho model đó**
đạt gate; feature phone-only/cloud-dependent có quyết định phạm vi rõ.
Model chưa có hardware không được gắn nhãn hoàn chỉnh.

## 11. Mốc review và đầu ra hữu hình

| Mốc | Người dùng sẽ review gì | Quyết định |
|---|---|---|
| M0 — inventory | Bảng mọi chức năng APK, family/transport/framing/codec matrix, phần chưa biết | Đề xuất thiết bị đại diện family mới; chưa mua/mượn trước M0 |
| M1 — nền tảng | BP1 discovery/init/readback/reconnect qua session mới, trace và tests | Cho phép chuyển UI features sang API mới |
| M2 — UI skeleton | Mock của từng trang với BP1 và một model khác capability | Chốt navigation/density/control states |
| M3 — BP1 feature parity | Controls offline áp dụng được cho BP1, per-feature evidence | Chốt release scope và remaining gaps |
| M4 — family thứ hai | Adapter/profile + hardware report hoặc experimental replay | Promote support mức phù hợp |
| M5 — Windows release | Installer, manual matrix, recovery guide, diagnostics | Phát hành candidate |

Chưa gán lịch hoàn thành “mọi model”: phần cứng hiện có một dòng BP1 và còn
unknown transport/native/cloud dependencies. Sau M0 có thể ước lượng từng
work package, ghi software effort riêng với thời gian chờ thiết bị/capture.
Không lấy số model catalog nhân một thời lượng giả định để đưa ra deadline.

## 12. Rủi ro cụ thể và cách xử lý

| Rủi ro | Hệ quả | Cách xử lý |
|---|---|---|
| Chỉ có BP1 hardware | Không xác nhận family khác | Source/replay trước; mua/mượn hoặc capture người dùng, per-feature experimental |
| Model gần giống khác firmware/transport | Gửi sai lệnh | Explicit variant/profile, identity query, deny unknown capability |
| JADX method lỗi/native SDK | Hành vi bị hiểu sai | Triage smali/secondary engine/native boundary; dynamic trace khi cần |
| Windows transport khác Android | Decompiled logic không chạy được trực tiếp | Spike OS API và packet flow; separate transport adapter |
| ACK/polarity theo model khác nhau | UI báo sai trạng thái | Family variant, query confirmation, tests đối chiếu source+capture |
| Catalog server thay đổi | Mất model/ảnh/config | Snapshot version/hash/cache + diff/review; offline fallback |
| Thay legacy làm hỏng BP1 đang dùng | Regression người dùng | Thay theo vertical slice; regression gate trước khi xóa implementation cũ, không duy trì runtime fallback |
| Capability chỉ check FE | Backend vẫn gửi packet | Backend kiểm ở feature service/executor, FE dùng cùng resolved descriptor |
| EQ curves tự suy diễn | UI/payload sai dù command đúng | Tách filter payload và visualization; trace consumer, không invent data |
| SoundFit phụ thuộc SDK/calibration | Không thể port nguyên hành vi | Nghiên cứu riêng; không tạo kết quả đo/thuật toán giả |
| OTA thiếu recovery | Hỏng hoặc không boot device | Giai đoạn riêng, thiết bị thử/recovery và firmware integrity trước writes |

## 13. Quyết định đã chốt và thông tin cần cho capture

Đã chốt: Windows trước; offline trước; phần cứng hiện là BP1 Ultra, có Android.

Cần bổ sung trước hardware work:

- Firmware BP1 Ultra, Android version, Windows version/adapter; tên Windows quảng bá có thể khác model chính thức.
- Khả năng xuất HCI log thực tế trên máy Android đó; chọn cách lấy log dựa trên OS/vendor, không giả định chỉ bật một setting là có file dùng được.

Các quyết định sản phẩm dưới đây đã được người dùng chốt; không hỏi lại khi triển khai trong phạm vi này:

- Hỗ trợ earbuds, over-ear, neckband, open-ear; loại speaker và thiết bị ngoài nhóm headphone/audio companion.
- Desktop utility gọn, dễ dùng, native với trải nghiệm Windows; không sao chép layout Baseus, giữ chức năng tương đương khi supported.
- Family khác chưa mua/mượn ngay; chọn sau M0 theo transport/protocol family/framing/codec, ưu tiên đại diện family mới.
- Close mặc định vào tray; Quit riêng. Auto-start và auto-reconnect opt-in, tắt mặc định.
- Experimental mode riêng, tắt mặc định; chỉ expose feature đủ evidence/implementation hoặc replay và giới hạn an toàn; backend kiểm capability/validation bắt buộc.

Quyết định kiến trúc: **metadata riêng, reviewed profile riêng; family codec
riêng, transport riêng; backend state làm nguồn chính; UI theo capability;
hardware verification theo feature/model/firmware/platform; thay runtime cũ bằng
đường mới theo lát cắt và xóa fallback/legacy sau migration.**
