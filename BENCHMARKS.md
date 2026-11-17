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

## 5. External comparison

No chat client publishes a controlled, like-for-like client benchmark, so this
report does not borrow marketing deltas as if they were measurements. The
numbers above are the reproducible ones: cache hydration, frame CPU time and
memory at fixed history sizes. Treat vendor "faster" claims as directional
until they publish their hardware, dataset and method.

### What the numbers mean

- On the metric Runecord controls - cache hydration - it is effectively
  instant (0.10 ms for a real cache, 2.43 ms for a 1.08 MB worst case).
- Frame cost is flat in history length because messages, members and channels
  are virtualized; the 500-message and 300-member windows are memory bounds,
  not rendering bounds.
- Memory scales with the rendered row window, not with the size of the archive.

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

## Caveats

- Frame numbers are CPU layout/shape/record time in Rune's offscreen renderer, not end-to-end GPU frame time.
- The synthetic snapshot is not a real account; 1.08 MB is a stress size, while the actual profile cache is 9 KB.
- The loopback API is not comparable to a remote internet startup path.
- The machine was otherwise idle during the final sequential runs; earlier parallel runs produced 30-60% higher frame times, which is why the report uses sequential medians.
