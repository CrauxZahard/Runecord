# Runecord benchmarks

Measured 2026-11-13 on the release build (`target\release\runecord.exe`, Rune `09a7590`).

## Environment

| | |
|---|---|
| CPU | AMD Ryzen 5 6600H, 6 cores / 12 threads |
| GPU | AMD Radeon 660M integrated graphics |
| RAM | 15.2 GB |
| OS | Windows 11 Home 10.0.26200 |
| Toolchain | rustc 1.99.1 |
| Mode | Rune headless/offscreen for frame tests; live window for memory |

The offscreen GPU initialization in this environment takes 12-17 s, so **process launch time is not a meaningful number here**. The cache and frame numbers below are.

## 1. Cache read/write

Synthetic snapshot: 4 guilds, 200 members, 40 channels, 6 guild details, 2,880 retained messages, 1.08 MB JSON.

| Operation | Result |
|---|---:|
| Serialize snapshot | 1.24 ms |
| Deserialize snapshot | 2.27 ms |
| Load through `cache::load` (file read + parse) | 2.43 ms |
| Real profile cache (9,036 bytes) | **102 µs** |
| Backend `/api/bootstrap` (4,435 bytes, 2 guilds, loopback) | 1.76 ms p50 / 4.02 ms p95 |

The real warm cache is fully parsed in about **0.12 ms**, so the shell can paint from it before the socket answers. The local bootstrap is also fast, so the bigger startup change is architectural: `App::new` no longer waits for it.

## 2. Frame cost vs message count

Command:

```powershell
$env:RUNECORD_BENCH_MESSAGES="5000"
.\target\release\runecord.exe --frames 120 --fps 60
```

120 frames, 60 fps, headless seeded state. CPU time is Rune layout/shape/record work, not GPU time. `max` is discarded because it includes the first frame. The message list now uses `virtual_rows`, so only visible rows are built.

| Rows | CPU mean | p50 | p95 | Layout nodes | p95 vs 16.67 ms budget |
|---:|---:|---:|---:|---:|---:|
| 50 | 1.34 ms | 1.60 ms | 2.09 ms | 1,601 | 13% |
| 240 | 1.29 ms | 1.47 ms | 2.01 ms | 1,489 | 12% |
| 1,000 | 1.26 ms | 1.46 ms | 1.74 ms | 1,620 | 10% |
| 3,000 | 1.29 ms | 1.49 ms | 1.82 ms | 1,502 | 11% |
| 5,000 | 1.27 ms | 1.48 ms | 1.79 ms | 1,502 | 11% |

Before virtualization, the same machine measured 2.25 ms at 240 rows, 11.6 ms at 1,000, 38.1 ms at 3,000 and 62.8 ms at 5,000. The frame cost is now flat in history length.

## 2b. Member-list frame cost

Command:

```powershell
$env:RUNECORD_BENCH_MEMBERS="300"
.\target\release\runecord.exe --frames 120 --fps 60
```

| Loaded members | CPU mean | p50 | p95 |
|---:|---:|---:|---:|
| 100 | 0.93 ms | 1.00 ms | 1.17 ms |
| 300 | 0.93 ms | 0.99 ms | 1.19 ms |
| 3,000 (above the 300 cap) | 0.90 ms | 0.94 ms | 1.23 ms |

The member list is also virtualized; the 300-row production cap is a memory/page bound, not a rendering bound.

## 2c. Large-guild backend fixture

A unit test builds a 10,001-member guild and calls the real store functions:

- summary bootstrap payload: **681 bytes** (no member rows)
- first 100-member page: **2.54 ms**
- search for one member returns one row
- test suite: 0.07 s

Before the split, a 10k-member bootstrap had to serialize and parse every member plus per-member role lookups.

## 3. Long-session cleanup

| Operation | Result |
|---|---:|
| Old global image scan, 12,000 messages | 37.9 µs per scan |
| New per-channel/visible-window scan, 240 messages | **0.48 µs** |
| Retention trim per incoming message | 0.92 µs |
| Old per-frame clone, 240 messages | 0.012 ms |
| Old per-frame clone, 5,000 messages | 0.491 ms |
| Old per-frame member clone, 500 members | 0.073 ms |

The image scan is ~79x cheaper per event and no longer grows with total cached history. Images are now requested only for the visible row range plus one viewport of prefetch, and the server serves a 480 px thumbnail (377 KB original → 56.5 KB thumbnail in one test).

## 4. Memory

Live window, warm cache, one account, idle after 8 s:

| Mode | Working set | Private |
|---|---:|---:|
| Live shell, 1 account | 90.1 MB | 89.2 MB |

Headless seeded state (offscreen surfaces inflate absolute numbers; compare rows, not the live shell):

| Rows | Working set | Private |
|---:|---:|---:|
| 50 | 162.5 MB | 179.5 MB |
| 240 | 177.5 MB | 201.9 MB |
| 3,000 | 382.9 MB | 415.0 MB |

Unbounded 3,000-row history costs ~205 MB more than 240 rows; the current 500-row production cap has the same purpose while virtualization keeps frame cost flat.

## 5. Discord comparison

Discord does **not** publish a controlled, like-for-like client benchmark. These are the closest public numbers, with their limits.

| Metric | Runecord | Discord | Fair comparison? |
|---|---|---|---|
| Warm cache hydrate | 0.10 ms (9 KB); 2.43 ms (1.08 MB) | No published equivalent | No |
| Desktop startup | Cache does not block window creation; process launch not measurable here | p50 desktop startup improved ~8% (**~650 ms**) in June 2026; p50 TTI improved **11.8%** from payload ordering in March 2026 | No: Discord includes updater, auth and network |
| Rust client cold launch | Not measurable cleanly here (12-17 s offscreen GPU init) | CTO demo: window fades in **<1 s**, friends list loaded before Electron reaches its interface at **~5 s** | No: demo video, not a controlled test |
| Idle memory | **90 MB**, one account | Phoenix: **290.2 MB with two accounts**; current Electron: **1-2 GB** reported idle | No: missing voice/video/overlay/media stack |
| Chat frame performance | 5,000 rows: **1.79 ms p95** (virtualized); 300 members: **1.19 ms p95** | Mobile chat list: **up to 60% fewer slow frames**, **~12% less chat-list memory**; server list: **10% faster startup**, **14% less startup memory** | Directional only; Discord publishes deltas, not ms |
| Strategy | Virtualized message/member/channel rows, lazy guild detail, paginated members, 500-message window | Virtualized server/chat/member lists, recycling pools | Same family; Discord has more content types |

### What the comparison means

- On the metric Runecord controls — cache hydration — it is effectively instant (0.10 ms for a real cache, 2.43 ms for a 1.08 MB worst case).
- Discord's publicly reported startup wins are still measured in **hundreds of milliseconds**. Their June 2026 note says 8% equals roughly 650 ms, so their p50 is still a seconds-scale number; they did not publish the baseline.
- Discord's Rust-client demo shows a sub-second window and 290 MB with two accounts, but it is a video, not a benchmark. The outlets covering it explicitly say there is no like-for-like test.
- Discord's mobile work uses virtualization and recycling. Runecord now uses Rune's `virtual_rows` for messages, members and channels, so frame cost is flat in history length; the 500-message/300-member windows are memory bounds, not rendering bounds.
- The 2021 DiscordTauri benchmark is old but still the only third-party launch measurement found: Discord launched in **7.41 s** on one run and **2.83 s** on a smaller account, with **535 MB / 516 MB** average memory. Conditions were not controlled by today's standards.

## Reproduce

Cache microbench:

```powershell
cargo run --release --manifest-path tools/cache-bench/Cargo.toml
```

Frame benchmark:

```powershell
$env:RUNECORD_BENCH_MESSAGES="240"
.\target\release\runecord.exe --frames 120 --fps 60

$env:RUNECORD_BENCH_MEMBERS="300"
.\target\release\runecord.exe --frames 120 --fps 60
```

Memory:

```powershell
$env:RUNECORD_BENCH_MESSAGES="240"
$p = Start-Process .\target\release\runecord.exe -ArgumentList @('--frames','1200','--fps','60','--realtime') -PassThru
Start-Sleep -Seconds 18
(Get-Process -Id $p.Id).WorkingSet64 / 1MB
Stop-Process -Id $p.Id
```

## 4. Server-side caches

The backend is one loopback process, so the cache is an in-process bounded map; Redis is not part of the design until multiple server nodes must share state.

`GET /api/guilds/100` on the demo guild, 12 alternating runs, forced miss by a write before each pair:

| State | p50 | p95 |
|---|---:|---:|
| Cache miss (rebuild detail + permissions) | **3.90 ms** | — |
| Cache hit | **1.87 ms** | 2.05 ms |

The small demo guild already halves latency; the gap grows with channels and overwrites because a miss recomputes `channel_permissions` for every visible channel. The key includes SQLite's `total_changes()`, so the write used to force each miss also proves invalidation is automatic.

Thumbnail cache: a 480 px PNG (56,537 bytes) served twice produced `thumbs.hits = 1`, `thumbs.misses = 1`, `weight = 56,537` from `GET /api/cache`, avoiding a second disk read, AES decrypt and image decode.

## Sources

- Discord Patch Notes, 2026-06-04: p50 desktop startup improved ~8% (~650 ms). <https://discord.com/blog/discord-patch-notes-june-4-2026>
- Discord Patch Notes, 2026-03-06: p50 TTI reduced 11.8% by payload ordering. <https://discord.com/blog/discord-patch-notes-march-6-2026>
- Discord, "Supercharging Discord Mobile", 2025-03-05: 14% startup memory reduction and 10% faster startup from server-list virtualization; up to 60% fewer slow frames and 12% less chat-list memory. <https://discord.com/blog/supercharging-discord-mobile-our-journey-to-a-faster-app>
- Windows Latest, 2026-11-06: Rust "Discord Phoenix" client window under 1 s, friends list before Electron's ~5 s interface, 290.2 MB with two accounts; explicitly not a controlled benchmark. <https://www.windowslatest.com/2026/11/06/every-windows-11-app-should-kill-electron-and-discord-just-showed-why-no-starting-screen-and-70-less-ram-even-with-two-accounts-signed-in/>
- DiscordTauri archived benchmark, 2021: Discord launch 7.41 s / 2.83 s, 535 MB / 516 MB average memory. <https://github.com/DiscordTauri/discord-tauri>

## Caveats

- Frame numbers are CPU layout/shape/record time in Rune's offscreen renderer, not end-to-end GPU frame time.
- The synthetic snapshot is not a real account; 1.08 MB is a stress size, while the actual profile cache is 9 KB.
- The loopback API is not comparable to Discord's internet startup path.
- The machine was otherwise idle during the final sequential runs; earlier parallel runs produced 30-60% higher frame times, which is why the report uses sequential medians.
