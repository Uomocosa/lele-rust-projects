# Candidate 7 — Active-window G-counter with base fold O(active)

Status: implemented — branch `candidate/c7-windowed`
Contract: `WindowedState{base, slots, tombstones}` `ACTIVE_CAP=32` `window=1` + sigs

## Nomenclature
- **TTL / active window**: only `ACTIVE_CAP` newest slots stay live; older evicted into `base`. TTL via count cap not wall clock (pure merge).
- **Tombstone**: evicted pubkey remembered forever so its old value not re-added (Delta resurrection prevention). Grow-only.
- **Base**: sum of evicted values, keeps total exact. `total = base + sum(slots)`.

## Why strictly better
`10k users` idle: `560KB → 32*40B + 8B ~1.3KB` (400×) while total exact. Active users keep per-user detail. Trade: rejoin with same pubkey blocked (needs new key via new tag).

## Vulnerability
Tombstone blow-up `O(evicted)` forever; resurrection if pruned. Mitigate epoch rotation (new WASM forks key). TTL pinning not used (cap only).

## Harness
Window 1 + fold deterministic (pop_first lexicographic). Divergent equal total still structural.
