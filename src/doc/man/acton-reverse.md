# acton-reverse(1)

## Name

acton-reverse --- Build TON state-flow reverse-engineering artifacts

## Synopsis

`acton reverse` [_options_] _command_

## Description

`acton reverse` is the Acton-native TON state-machine reverse-engineering
surface. It emits reusable JSON and Markdown artifacts instead of one-off logs,
so CLI commands, the Test UI, localnet UI, fork replay checks, and audit
handoffs can consume the same evidence model.

The command group is organized around the state-flow loop:

- `retrace` exports one `StateFlowTx` evidence JSON for a known transaction
  hash
- `collect` turns recent account history into a corpus of retraced
  transactions
- `infer` clusters the corpus by opcode and proposes message, storage, replay,
  outbound effect, state-machine, confidence, and unknown-field surfaces
- `replay` replays or mutates a `StateFlowTx` on the same pre-state and emits a
  diff
- `report` renders corpus, schema, and replay artifacts into a Markdown audit
  report
- `analyze` runs collect, infer, replay, and report for one target address
- `smoke` runs checked-in real-chain targets through the full loop and writes
  a manifest bundle
- `verify-artifacts` validates that a bundle still contains the evidence needed
  for the short-, medium-, and long-term reverse workflow

## Artifact Model

The primary single-transaction artifact is `StateFlowTx`. It includes:

- transaction identity and replay context
- pre/post shard account state, including code/data hashes and cell shapes
- inbound message body/opcode evidence
- outbound messages and parsed out actions
- VM trace and executor trace logs
- c5/out-action cell evidence

The higher-level artifacts reuse the same model:

- corpus JSON contains retraced `StateFlowTx` entries and collection failures
- schema JSON contains opcode candidates, message/storage/effect surfaces,
  replay probes, confidence, unknown fields, and audit signals
- replay diff JSON compares baseline and mutated observations from the same
  pre-state
- report Markdown renders the evidence tables used by review and audit
- artifact manifests preserve target context, paths, and validation capability
  checks

## Subcommands

### acton reverse retrace

Replay one transaction and emit `StateFlowTx` JSON.

#### Synopsis

`acton reverse retrace` [_options_] _hash_

#### Options

{{#options command="acton reverse retrace"}}

{{#option "_hash_" }}
Transaction hash in hex format to retrace.
{{/option}}

{{#option "`--net` _network_" }}
Network to use.
{{/option}}

{{#option "`-o`, `--output` _path_" }}
Write state-flow JSON to a file. `--out` is also accepted.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output.
{{/option}}

{{/options}}

### acton reverse collect

Collect recent account history into a state-flow corpus.

#### Synopsis

`acton reverse collect` [_options_] _address_

#### Options

{{#options command="acton reverse collect"}}

{{#option "_address_" }}
Account address in friendly or raw format.
{{/option}}

{{#option "`--net` _network_" }}
Network to use.
{{/option}}

{{#option "`--limit` _count_" }}
Maximum number of recent account transactions to collect.
{{/option}}

{{#option "`-o`, `--output` _path_" }}
Write state-flow corpus JSON to a file. `--out` is also accepted.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output.
{{/option}}

{{/options}}

### acton reverse infer

Infer opcode and effect schema candidates from a state-flow corpus or artifact
manifest.

#### Synopsis

`acton reverse infer` [_options_] [_corpus_]

#### Options

{{#options command="acton reverse infer"}}

{{#option "_corpus_" }}
State-flow corpus JSON produced by `acton reverse collect`.
{{/option}}

{{#option "`--artifact-manifest` _path_" }}
State-flow artifact manifest produced by `acton reverse smoke` or
`acton reverse analyze`.
{{/option}}

{{#option "`--target-id` _id_" }}
Target id to select from an artifact manifest.
{{/option}}

{{#option "`-o`, `--output` _path_" }}
Write schema candidate JSON to a file. `--out` is also accepted.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output.
{{/option}}

{{/options}}

### acton reverse replay

Replay or mutate a `StateFlowTx` or corpus artifact and emit a diff.

#### Synopsis

`acton reverse replay` [_options_] [_state-flow_]

#### Options

{{#options command="acton reverse replay"}}

{{#option "_state-flow_" }}
`StateFlowTx` JSON from `acton reverse retrace` or corpus JSON from
`acton reverse collect`.
{{/option}}

{{#option "`--artifact-manifest` _path_" }}
State-flow artifact manifest produced by `acton reverse smoke` or
`acton reverse analyze`.
{{/option}}

{{#option "`--target-id` _id_" }}
Target id to select from an artifact manifest.
{{/option}}

{{#option "`--tx-index` _index_" }}
Transaction index to replay when the input is a corpus artifact.
{{/option}}

{{#option "`--tx-hash` _hash_" }}
Transaction hash to replay when the input is a corpus artifact.
{{/option}}

{{#option "`--replay-probe` _field_" }}
Replay an inferred schema probe from an artifact manifest.
{{/option}}

{{#option "`--flip-body-bit` _bit_" }}
Flip one inbound message body bit before replay.
{{/option}}

{{#option "`--body-boc64` _boc64_" }}
Replace inbound message body with this base64 BoC before replay.
{{/option}}

{{#option "`--set-body-uint` _offset:bits:value_" }}
Set an unsigned integer field in the inbound body before replay.
{{/option}}

{{#option "`--ignore-chksig`" }}
Ignore TVM signature checks during local replay.
{{/option}}

{{#option "`-o`, `--output` _path_" }}
Write replay diff JSON to a file. `--out` is also accepted.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output.
{{/option}}

{{/options}}

### acton reverse report

Generate a state-flow reverse-engineering report.

#### Synopsis

`acton reverse report` [_options_] [_corpus_]

#### Options

{{#options command="acton reverse report"}}

{{#option "_corpus_" }}
State-flow corpus JSON produced by `acton reverse collect`.
{{/option}}

{{#option "`--schema` _path_" }}
Schema candidate JSON produced by `acton reverse infer`.
{{/option}}

{{#option "`--replay` _path_" }}
Replay diff JSON produced by `acton reverse replay`. May be repeated.
{{/option}}

{{#option "`--artifact-manifest` _path_" }}
State-flow artifact manifest produced by `acton reverse smoke` or
`acton reverse analyze`.
{{/option}}

{{#option "`--target-id` _id_" }}
Target id to select from an artifact manifest.
{{/option}}

{{#option "`-o`, `--output` _path_" }}
Write Markdown report to a file. `--out` is also accepted.
{{/option}}

{{/options}}

### acton reverse verify-artifacts

Validate a state-flow artifact manifest bundle.

#### Synopsis

`acton reverse verify-artifacts` [_options_] _artifacts_

#### Options

{{#options command="acton reverse verify-artifacts"}}

{{#option "_artifacts_" }}
State-flow artifact manifest produced by `acton reverse smoke` or
`acton reverse analyze`.
{{/option}}

{{#option "`--target-id` _id_" }}
Only validate artifacts for this target id.
{{/option}}

{{#option "`-o`, `--output` _path_" }}
Write artifact validation JSON to a file. `--out` is also accepted.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output.
{{/option}}

{{/options}}

### acton reverse analyze

Run collect, infer, replay, and report for one target address.

#### Synopsis

`acton reverse analyze` [_options_] _address_

#### Options

{{#options command="acton reverse analyze"}}

{{#option "_address_" }}
Account address in friendly or raw format.
{{/option}}

{{#option "`--net` _network_" }}
Network to use.
{{/option}}

{{#option "`--target-id` _id_" }}
Target id to write into summary, manifest, validation, and output paths.
{{/option}}

{{#option "`--source-url` _url_" }}
Source URL for the analyzed target.
{{/option}}

{{#option "`--notes` _text_" }}
Human notes to preserve in the generated target artifacts.
{{/option}}

{{#option "`--limit` _count_" }}
Maximum number of recent account transactions to collect.
{{/option}}

{{#option "`--retrace-tx-hash` _hash_" }}
Transaction hash to retrace into `retrace.json` for this analysis target.
{{/option}}

{{#option "`--replay-tx-index` _index_" }}
Corpus transaction index to replay.
{{/option}}

{{#option "`--replay-tx-hash` _hash_" }}
Corpus transaction hash to replay.
{{/option}}

{{#option "`--flip-body-bit` _bit_" }}
Flip one inbound message body bit before replay.
{{/option}}

{{#option "`--body-boc64` _boc64_" }}
Replace inbound message body with this base64 BoC before replay.
{{/option}}

{{#option "`--set-body-uint` _offset:bits:value_" }}
Set an unsigned integer field in the inbound body before replay.
{{/option}}

{{#option "`--ignore-chksig`" }}
Ignore TVM signature checks during local replay.
{{/option}}

{{#option "`--out-dir` _path_" }}
Directory for analysis output artifacts.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output artifacts.
{{/option}}

{{/options}}

### acton reverse smoke

Run state-flow smoke targets through collect, infer, replay, and report.

#### Synopsis

`acton reverse smoke` [_options_]

#### Options

{{#options command="acton reverse smoke"}}

{{#option "`--targets` _path_" }}
Smoke target manifest JSON.
{{/option}}

{{#option "`--target-id` _id_" }}
Only run the smoke target with this id.
{{/option}}

{{#option "`--out-dir` _path_" }}
Directory for smoke output artifacts.
{{/option}}

{{#option "`--pretty`" }}
Pretty-print JSON output artifacts.
{{/option}}

{{/options}}

### Display Options

{{> options-display }}

### Project Options

{{> options-project-resolved }}

## Environment

Built-in `mainnet`/`testnet` requests read `TONCENTER_MAINNET_API_KEY` or
`TONCENTER_TESTNET_API_KEY`, depending on the selected network.

Both built-in networks also fall back to `TON_CENTER_API_KEY`, which is useful
when one shared key is kept in a local `.env` file.

Acton loads `.env` automatically, so the simplest setup during project work is
usually to keep these keys there and use shell environment variables only for
one-off overrides or CI.

## Output Layout

`acton reverse analyze` and `acton reverse smoke` write a bundle directory with
these stable paths:

- `summary.json`: run-level target status and output paths
- `artifacts.json`: portable artifact manifest
- `validation.json`: bundle validation result and capability checks
- `<target>/corpus.json`: collected retraced transactions
- `<target>/schema.json`: inferred schema and state-machine surfaces
- `<target>/transaction-<n>.json`: selected replay source transaction
- `<target>/retrace.json`: explicit retrace target when configured
- `<target>/replay*.json`: replay diffs, including schema probe replays
- `<target>/report.md`: rendered state-flow report

Use `acton reverse verify-artifacts artifacts.json` after moving or archiving a
bundle to prove that all required evidence is still present and internally
consistent.

## Exit Status

- `0`: The requested artifact was generated or validated successfully.
- `1`: Network lookup, retrace, replay, parsing, artifact validation, or output
  writing failed.

## Examples

1. Export a single transaction evidence JSON:

   ```bash
   acton reverse retrace bd4352bc4c89b3a5ea8af3667baf67b6a73d3b4873b1ef604c746831b3a14566 \
       --net mainnet --out tx.json --pretty
   ```

2. Run one address through the full collect-infer-replay-report loop:

   ```bash
   acton reverse analyze EQAgvOlWk7C0Pz3YgSaX-MA7UDDhE9n6eQgQRwJahOBm4VKr \
       --net mainnet \
       --target-id tonviewer-requested-target \
       --source-url https://tonviewer.com/EQAgvOlWk7C0Pz3YgSaX-MA7UDDhE9n6eQgQRwJahOBm4VKr \
       --retrace-tx-hash bd4352bc4c89b3a5ea8af3667baf67b6a73d3b4873b1ef604c746831b3a14566 \
       --flip-body-bit 0 \
       --ignore-chksig \
       --out-dir target/stateflow-analysis \
       --pretty
   ```

3. Verify a generated bundle:

   ```bash
   acton reverse verify-artifacts target/stateflow-analysis/artifacts.json --pretty
   ```

4. Load an inferred replay probe from a manifest:

   ```bash
   acton reverse replay --artifact-manifest target/stateflow-analysis/artifacts.json \
       --target-id tonviewer-requested-target \
       --replay-probe 0xd5e3832f:query_id \
       --ignore-chksig \
       --out replay-query-id.json \
       --pretty
   ```

5. Render a report from a manifest:

   ```bash
   acton reverse report --artifact-manifest target/stateflow-analysis/artifacts.json \
       --target-id tonviewer-requested-target \
       --out report.md
   ```

## See Also

- `acton retrace`
- `acton test --ui`
- `acton localnet`
- [Commands overview](https://ton-blockchain.github.io/acton/docs/commands/overview)
