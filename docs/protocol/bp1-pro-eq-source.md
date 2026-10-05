# BP1 Pro preset EQ source dossier

Evidence: local Android 2.17.0.1 consumer source and the existing public BP1 Pro modelParams metadata. This is source evidence, not a new hardware acceptance report. Proprietary source is excluded from Git.

| Consumer | Responsibility | Result |
| --- | --- | --- |
| EarEqDefaultRegulationActivityV2.X1 / receiveData | Query and reconcile selection | BA30 queries; AA30 carries dictSort |
| EarEqDefaultRegulationActivityV2.d2 / DeviceManager.n0 | Select write path by model | BP1 Pro uses full preset filters; sort-only branch names Bowie30Max and BowieH1sPro |
| EarEqDefaultRegulationPresenter.w / o / s | Serialize model preset | BA31, dictSort, then eight bytes per filter |
| EarEqDefaultRegulationActivityV2.receiveData | Other replies | AA42 is spatial state; AA43 is success/error ACK, neither is EQ selection |

Filter layout: frequency LE16, truncated `(gain * 10 + 120)` LE16, truncated `(Q * 10)` LE16, filter type LE16. The description prefix is a display color, not packet data. Presets have six or seven filters; do not invent eight-band preview curves from these parametric filters.

Catalog contains the seven metadata selections: 0 Classic, 1 Deep Bass, 3 Hi-Fi Live, 7 Jazz, 8 Classical, 9 Treble Boost, 10 Acoustic. UI sends a catalog ID; backend resolves and authorizes its wire index and filters. Raw AA30 indexes remain observable even when not recognized by the catalog. Confirmation requires the matching AA30 index in the current session.

Custom EQ slot/ANC payload semantics and readback require their own source/capture verification. Existing custom implementation is not promoted by this preset correction. Firmware scope and device-side persistence after reconnect remain open hardware gates.


## Custom consumer trace

EarEqSelfDefinePresenter `A/B` uses the same LE16 truncation layout. `C/F` chooses default frequencies and Q; BP1 Pro is absent from DeviceManager.Y, so frequencies are 100/200/400/800/1000/3000/6000/10000, Q=1 and filter=1. `m/G` sends BA31 plus index and filters, adding a selector only for Storm 1. Activity `i1/p1` calls the two-selector `n` path only for Storm 1. `r` initializes index 101; `w` persists local entries of that index. A matching AA30 101 confirms selection only, not equality of transmitted filters. Local list capacity must not be described as device-side slots.
