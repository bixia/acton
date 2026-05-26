# TON reverse-validation target candidates

This directory keeps real mainnet targets worth using as a validation range for
`acton reverse smoke`, `collect`, `infer`, `replay`, `retrace`, and report work.

The set is intentionally split into two files:

- `registry.json`: broad research registry with protocol grouping, source links,
  coverage notes, sample transactions, and confidence.
- `smoke-targets.high-confidence.json`: smoke-ready subset using the same
  `schemaVersion` / `targets[]` shape as `crates/ton-stateflow/smoke-targets.json`.

## How to run

Run the full high-confidence subset:

```bash
target/debug/acton reverse smoke \
  --targets crates/ton-stateflow/validation-targets/smoke-targets.high-confidence.json \
  --out-dir target/stateflow-validation-targets \
  --pretty
```

Run one target:

```bash
target/debug/acton reverse smoke \
  --targets crates/ton-stateflow/validation-targets/smoke-targets.high-confidence.json \
  --target-id stonfi-v2-ton-usdt-pool \
  --out-dir target/stateflow-validation-targets/stonfi-v2-ton-usdt-pool \
  --pretty
```

Validate the produced artifact bundle:

```bash
target/debug/acton reverse verify-artifacts \
  target/stateflow-validation-targets/artifacts.json \
  --pretty
```

## Priority model

- `p0`: first validation range. High activity, clear source lineage, and useful
  ABI/state coverage.
- `p1`: keep in the broader registry. Useful for protocol breadth, edge cases,
  or negative tests, but not necessarily first-pass smoke targets.
- `p2`: reference or low-liquidity targets. Keep for later manual exploration.

## Coverage map

- STON.fi: v2 constant-product, v1 legacy, stable-swap, weighted stable, router,
  pTON path, and pool op differences.
- DeDust: factory, native vault, jetton vault, volatile pool, stable pool, and
  factory-derived vault addressing. Keep classic `PoolType` pools separate from
  DeDust CPMM v2 UI pools, which use a different layout/opcode family.
- TONCO: PoolV3/concentrated-liquidity-style pool swaps, stable pairs, and
  explicit `tx hash` versus `root hash` trace handling.
- swap.coffee: Coffee DEX factory/vault/pool surfaces, constant-product pools,
  route-builder/API surfaced pools, and lower-liquidity protocol-owned pools.
- USDt: canonical jetton master, high-volume jetton wallets, wallet derivation
  checks, and spoof/metadata-negative cases.

## Suggested order

1. Validate USDt identity first: canonical master, wallet derivation,
   `transfer`, `transfer_notification`, and `excesses`.
2. Run high-volume swap flows: STON.fi v2, DeDust native/USDt vault and pool,
   TONCO PoolV3, and Coffee DEX USDt pools.
3. Add compatibility breadth: STON.fi v1 legacy, STON.fi stable/weighted stable,
   DeDust stable pool, TONCO stable-to-stable PoolV3, and Coffee stable AMM.
4. Add negative cases: pools/routes where symbol or metadata resembles USDt but
   the jetton master is not the canonical USDt master.

Future expansion targets outside this first requested set: Tonstakers/tsTON for
liquid-staking accounting, EVAA for lending and liquidation paths, Storm Trade
for derivatives/perps, Torch/tgUSD for yield-bearing stablecoin flows, and FIVA
for yield-tokenization/composability stress cases.

## Address form

Acton accepts raw and friendly TON addresses through the retrace address parser,
but checked-in smoke targets should use mainnet bounceable URL-safe `EQ...`
addresses. Keep raw `0:<hash>` forms in the registry when protocol indexers
return raw addresses, especially for TONCO.

## Source policy

Prefer official docs and protocol APIs first, then protocol indexers, then
Tonviewer/TonAPI explorer evidence. Never identify USDt by symbol or image
alone; use the canonical master address:

```text
EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs
```

For TONCO samples, the registry records both the pool transaction hash and the
root trace hash when available. Use the pool transaction hash for the pool-local
transaction and the root hash when the retrace backend expects a full trace root.

Native TON is protocol-specific. STON.fi and TONCO use protocol-specific
pTON/wtPTon wrappers, while DeDust and Coffee DEX use native vault contracts.
Do not collapse these into one global wrapper rule.

For DeDust, prefer the newer Hub docs over older `docs.dedust.io` pages when
they disagree. For TONCO, treat indexer data as target discovery and check live
get-method/source behavior before concluding pool math or router semantics.
Explorer labels are useful hints, not primary proof.
