# TON Reverse StateFlow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first Acton-native TON reverse-engineering toolchain slice: `acton reverse retrace <tx-hash> --net <network> --out <file>` emits stable state-flow JSON that captures transaction identity, inbound message, pre/post account state, VM trace, executor logs, c5/out actions, and reserved ABI/schema sections.

**Architecture:** Fork Acton and add reverse tooling inside the workspace instead of treating Acton as an external dependency. Keep `ton-retrace` changes small by exposing replay artifacts, put the canonical JSON model in a new `ton-stateflow` crate, add the CLI under `src/commands/reverse`, and defer Test UI changes until the artifact format is stable.

**Tech Stack:** Rust 2024, Acton workspace crates, `ton-retrace`, `tycho-types`, `tvm-logs`, `serde`, `serde_json`, Clap, existing Acton integration tests.

---

## Scope

This plan implements Phase 1 only:

- Add `acton reverse retrace <tx-hash> --net mainnet --out tx.json`.
- Define the `StateFlowTx` JSON artifact and keep it stable enough for later `collect`, `infer`, `replay`, `report`, and Test UI work.
- Reserve first-class JSON sections for the state-machine evidence we need to recover:
  - how `recv_internal` parsed the inbound message body;
  - per-opcode TL-B/ABI sketches;
  - pre/post storage changes;
  - VM trace evidence for which body/storage fields were actually read;
  - c5/action/out-message effects;
  - same-pre-state replay results for constructed or mutated messages.
- Reuse existing `ton-retrace`, VM log parsing, executor action parsing ideas, and trace bundle conventions.
- Do not modify `acton-test-ui` in this phase.

Future phases are intentionally left as separate plans:

- `acton reverse collect <address> --limit 1000 --out corpus/`
- `acton reverse infer corpus/ --out schemas/`
- `acton reverse replay <address> --schema schemas/inbound.yaml --out replay/`
- `acton reverse report corpus/ schemas/ --out report.md`
- Test UI State Flow / Schema / Replay views.

## File Map

- Create `crates/ton-stateflow/Cargo.toml`: workspace crate manifest.
- Create `crates/ton-stateflow/src/lib.rs`: public module exports.
- Create `crates/ton-stateflow/src/model.rs`: serializable `StateFlowTx` data model.
- Create `crates/ton-stateflow/src/convert.rs`: conversion from `ton_retrace::StateReplayResult` into `StateFlowTx`.
- Create `src/commands/reverse/mod.rs`: `ReverseCommand` enum and command dispatch.
- Modify `Cargo.toml`: add `ton-stateflow` workspace dependency and root dependency.
- Modify `src/commands/mod.rs`: expose `reverse`.
- Modify `src/bin/acton.rs`: add the `Reverse` top-level command and dispatch.
- Modify `crates/ton-retrace/src/types.rs`: add replay artifact structs.
- Modify `crates/ton-retrace/src/runner.rs`: add `retrace_with_state_flow`.
- Modify `crates/ton-retrace/src/lib.rs`: export `retrace_with_state_flow` and replay artifact types.
- Add tests near changed code:
  - `crates/ton-stateflow/src/model.rs` unit test for JSON field names.
  - `crates/ton-stateflow/src/convert.rs` unit test for minimal conversion.
  - `tests/integration_test.rs` CLI help snapshot/update for `acton reverse --help`.

---

### Task 1: Create the `ton-stateflow` Crate Skeleton

**Files:**
- Create: `crates/ton-stateflow/Cargo.toml`
- Create: `crates/ton-stateflow/src/lib.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the crate manifest**

Create `crates/ton-stateflow/Cargo.toml`:

```toml
[package]
name = "ton-stateflow"
version = { workspace = true }
publish = false
authors = { workspace = true }
edition = { workspace = true }
rust-version = { workspace = true }
homepage = { workspace = true }
documentation = { workspace = true }
repository = { workspace = true }
license = { workspace = true }

[dependencies]
anyhow = { workspace = true }
hex = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
ton-retrace = { workspace = true }
tvm-logs = { workspace = true }
tycho-types = { workspace = true }

[dev-dependencies]
expect-test = { workspace = true }

[lints]
workspace = true
```

- [ ] **Step 2: Add the crate root**

Create `crates/ton-stateflow/src/lib.rs`:

```rust
pub mod convert;
pub mod model;

pub use convert::{StateFlowConvertError, state_flow_from_retrace};
pub use model::{
    AbiDecodeInfo, AccountStateSnapshot, ActionSummary, BodySchemaCandidate, CellRef,
    FieldReadEvidence, InboundMessageFlow, MessageBodyInfo, OpcodeFieldSketch,
    OpcodeSchemaSketch, RecvInternalEvidence, ReplayValidation, StateFlowTx, StateFlowVersion,
    StorageDiff, StorageFieldChange, TxIdentity, VmTraceInfo,
};
```

- [ ] **Step 3: Wire the crate into the workspace**

In the root `Cargo.toml`, add this workspace dependency near `ton-retrace`:

```toml
ton-stateflow = { path = "crates/ton-stateflow" }
```

Also add this root dependency near the other local dependencies:

```toml
ton-stateflow = { workspace = true }
```

- [ ] **Step 4: Verify the crate is detected**

Run:

```bash
cargo metadata --no-deps -q | jq -r '.packages[].name' | rg '^ton-stateflow$'
```

Expected:

```text
ton-stateflow
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/ton-stateflow
git commit -m "feat(reverse): add ton-stateflow crate"
```

---

### Task 2: Define the Stable `StateFlowTx` JSON Model

**Files:**
- Create: `crates/ton-stateflow/src/model.rs`
- Test: `crates/ton-stateflow/src/model.rs`

- [ ] **Step 1: Add the model**

Create `crates/ton-stateflow/src/model.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StateFlowVersion {
    V1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateFlowTx {
    pub schema_version: StateFlowVersion,
    pub identity: TxIdentity,
    pub inbound: InboundMessageFlow,
    pub pre: AccountStateSnapshot,
    pub post: AccountStateSnapshot,
    pub storage_diff: StorageDiff,
    pub vm_trace: VmTraceInfo,
    pub recv_internal: RecvInternalEvidence,
    pub executor_logs: String,
    pub c5: Option<CellRef>,
    pub installed_actions: Vec<ActionSummary>,
    pub executed_actions: Vec<ActionSummary>,
    pub out_messages: Vec<CellRef>,
    pub abi_decode: Option<AbiDecodeInfo>,
    pub schema_candidates: Vec<BodySchemaCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxIdentity {
    pub network: String,
    pub hash: String,
    pub lt: String,
    pub account: String,
    pub utime: u64,
    pub state_update_hash_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InboundMessageFlow {
    pub sender: Option<String>,
    pub destination: String,
    pub value: Option<String>,
    pub bounced: bool,
    pub raw_message: Option<CellRef>,
    pub body: MessageBodyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageBodyInfo {
    pub opcode: Option<String>,
    pub body: Option<CellRef>,
    pub body_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountStateSnapshot {
    pub shard_account: CellRef,
    pub code: Option<CellRef>,
    pub data: Option<CellRef>,
    pub balance: String,
    pub data_hash: Option<String>,
    pub code_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageDiff {
    pub data_hash_changed: bool,
    pub balance_delta: String,
    pub changed_fields: Vec<StorageFieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageFieldChange {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VmTraceInfo {
    pub raw_log: String,
    pub steps: usize,
    pub final_c5: Option<CellRef>,
    pub field_reads: Vec<FieldReadEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldReadEvidence {
    pub source: String,
    pub op: String,
    pub bit_offset: Option<u32>,
    pub ref_index: Option<u32>,
    pub instruction: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecvInternalEvidence {
    pub entrypoint: String,
    pub opcode: Option<String>,
    pub body_parse: Vec<FieldReadEvidence>,
    pub schema_sketch: Option<OpcodeSchemaSketch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpcodeSchemaSketch {
    pub opcode: Option<String>,
    pub name: Option<String>,
    pub tlb: String,
    pub fields: Vec<OpcodeFieldSketch>,
    pub constraints: Vec<String>,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpcodeFieldSketch {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionSummary {
    pub kind: String,
    pub index: usize,
    pub hash: Option<String>,
    pub body: Option<CellRef>,
    pub effect: String,
    pub failure_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AbiDecodeInfo {
    pub contract: String,
    pub inbound_body_type: Option<String>,
    pub storage_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BodySchemaCandidate {
    pub opcode: Option<String>,
    pub confidence: String,
    pub source: String,
    pub tlb: String,
    pub fields: Vec<OpcodeFieldSketch>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CellRef {
    pub boc64: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayValidation {
    pub same_pre_state: bool,
    pub message_mutation: String,
    pub exit_code: Option<i32>,
    pub storage_diff: StorageDiff,
    pub action_effects: Vec<ActionSummary>,
    pub verdict: String,
}
```

- [ ] **Step 2: Add a serialization test**

Append this test module to `crates/ton-stateflow/src/model.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn state_flow_tx_uses_stable_json_names() {
        let tx = StateFlowTx {
            schema_version: StateFlowVersion::V1,
            identity: TxIdentity {
                network: "mainnet".to_owned(),
                hash: "0xabc".to_owned(),
                lt: "123".to_owned(),
                account: "0:abc".to_owned(),
                utime: 1,
                state_update_hash_ok: true,
            },
            inbound: InboundMessageFlow {
                sender: None,
                destination: "0:abc".to_owned(),
                value: None,
                bounced: false,
                raw_message: None,
                body: MessageBodyInfo {
                    opcode: Some("0x00000000".to_owned()),
                    body: None,
                    body_hash: None,
                },
            },
            pre: AccountStateSnapshot {
                shard_account: CellRef {
                    boc64: "te6ccgEBAQEAAgAAAA==".to_owned(),
                    hash: "0xpre".to_owned(),
                },
                code: None,
                data: None,
                balance: "0".to_owned(),
                data_hash: None,
                code_hash: None,
            },
            post: AccountStateSnapshot {
                shard_account: CellRef {
                    boc64: "te6ccgEBAQEAAgAAAA==".to_owned(),
                    hash: "0xpost".to_owned(),
                },
                code: None,
                data: None,
                balance: "0".to_owned(),
                data_hash: None,
                code_hash: None,
            },
            storage_diff: StorageDiff {
                data_hash_changed: false,
                balance_delta: "0".to_owned(),
                changed_fields: vec![],
            },
            vm_trace: VmTraceInfo {
                raw_log: String::new(),
                steps: 0,
                final_c5: None,
                field_reads: vec![],
            },
            recv_internal: RecvInternalEvidence {
                entrypoint: "recv_internal".to_owned(),
                opcode: Some("0x00000000".to_owned()),
                body_parse: vec![],
                schema_sketch: None,
            },
            executor_logs: String::new(),
            c5: None,
            installed_actions: vec![],
            executed_actions: vec![],
            out_messages: vec![],
            abi_decode: None,
            schema_candidates: vec![],
        };

        let json = serde_json::to_string_pretty(&tx).expect("serialize state flow");
        expect![[r#"
{
  "schema_version": "v1",
  "identity": {
    "network": "mainnet",
    "hash": "0xabc",
    "lt": "123",
    "account": "0:abc",
    "utime": 1,
    "state_update_hash_ok": true
  },
  "inbound": {
    "sender": null,
    "destination": "0:abc",
    "value": null,
    "bounced": false,
    "raw_message": null,
    "body": {
      "opcode": "0x00000000",
      "body": null,
      "body_hash": null
    }
  },
  "pre": {
    "shard_account": {
      "boc64": "te6ccgEBAQEAAgAAAA==",
      "hash": "0xpre"
    },
    "code": null,
    "data": null,
    "balance": "0",
    "data_hash": null,
    "code_hash": null
  },
  "post": {
    "shard_account": {
      "boc64": "te6ccgEBAQEAAgAAAA==",
      "hash": "0xpost"
    },
    "code": null,
    "data": null,
    "balance": "0",
    "data_hash": null,
    "code_hash": null
  },
  "storage_diff": {
    "data_hash_changed": false,
    "balance_delta": "0",
    "changed_fields": []
  },
  "vm_trace": {
    "raw_log": "",
    "steps": 0,
    "final_c5": null,
    "field_reads": []
  },
  "recv_internal": {
    "entrypoint": "recv_internal",
    "opcode": "0x00000000",
    "body_parse": [],
    "schema_sketch": null
  },
  "executor_logs": "",
  "c5": null,
  "installed_actions": [],
  "executed_actions": [],
  "out_messages": [],
  "abi_decode": null,
  "schema_candidates": []
}
"#]]
        .assert_eq(&json);
    }
}
```

- [ ] **Step 3: Run the model test**

Run:

```bash
cargo test -p ton-stateflow state_flow_tx_uses_stable_json_names
```

Expected:

```text
test model::tests::state_flow_tx_uses_stable_json_names ... ok
```

- [ ] **Step 4: Commit**

```bash
git add crates/ton-stateflow/src/model.rs
git commit -m "feat(reverse): define state flow artifact model"
```

---

### Task 3: Expose Replay Artifacts from `ton-retrace`

**Files:**
- Modify: `crates/ton-retrace/src/types.rs`
- Modify: `crates/ton-retrace/src/runner.rs`
- Modify: `crates/ton-retrace/src/lib.rs`

- [ ] **Step 1: Add replay artifact types**

In `crates/ton-retrace/src/types.rs`, add these structs after `TraceResult`:

```rust
/// Full replay data needed by reverse-engineering tooling.
#[derive(Debug, Clone)]
pub struct StateReplayResult {
    pub trace: TraceResult,
    pub network: String,
    pub tx_hash_hex: String,
    pub pre_shard_account: Cell,
    pub post_shard_account: Cell,
    pub inbound_message: Option<Cell>,
}
```

- [ ] **Step 2: Refactor `retrace_base_tx` internals**

In `crates/ton-retrace/src/runner.rs`, introduce:

```rust
pub async fn retrace_with_state_flow(
    net: Network,
    link: &str,
    additional_libs: HashMap<HashBytes, Cell>,
) -> anyhow::Result<StateReplayResult> {
    let base_tx = find_base_tx_by_hash(net.clone(), link).await?;
    retrace_base_tx_with_state_flow(net, base_tx, additional_libs).await
}
```

Then create `retrace_base_tx_with_state_flow` by moving the current `retrace_base_tx` body into a shared internal implementation that captures the target transaction state immediately before and after the final emulation:

```rust
async fn retrace_base_tx_impl(
    net: Network,
    base_tx: BaseTxInfo,
    additional_libs: HashMap<HashBytes, Cell>,
) -> anyhow::Result<(TraceResult, StateReplayArtifacts)> {
    // Keep the existing retrace_base_tx workflow, including:
    // - find_raw_tx_by_hash
    // - find_shard_block_for_tx
    // - find_full_block_for_seqno
    // - get_block_config
    // - get_block_account
    // - collect_used_libraries
    // - emulate_previous_transactions
    //
    // The only added capture points are:
    let pre_target_shard_account = shard_account.clone();

    let (tx_res, executor_logs) = emulate(
        &our_tx,
        &block_config,
        &shard_account,
        libs.as_ref(),
        rand_seed,
    )?;

    let res = match tx_res {
        EmulationResult::Success(res) => res,
        EmulationResult::Error(err) => {
            anyhow::bail!("Emulated transaction failed: {:?}", err.error);
        }
    };

    let post_target_shard_account = Boc::decode_base64(res.shard_account.as_ref())?;

    // Build TraceResult exactly as retrace_base_tx does today.
    let trace_result = TraceResult {
        state_update_hash_ok,
        code_cell: loaded_code.or_else(|| code_cell.clone()),
        original_code_cell: code_cell,
        in_msg: TraceInMessage {
            sender,
            contract,
            amount: amount.map(|a| u128::from(a) as u64),
            opcode,
        },
        money,
        emulated_tx: TraceEmulatedTx {
            raw: our_tx.clone(),
            utime: u64::from(emulated_tx.now),
            lt: emulated_tx.lt,
            compute_info,
            executor_logs,
            actions: final_actions,
            c5,
            vm_logs: res.vm_log,
        },
    };

    Ok((
        trace_result,
        StateReplayArtifacts {
            pre_shard_account: to_cell(&pre_target_shard_account),
            post_shard_account: post_target_shard_account,
            inbound_message: our_tx.in_msg.clone(),
        },
    ))
}
```

Add the helper artifact type near the new implementation:

```rust
struct StateReplayArtifacts {
    pre_shard_account: Cell,
    post_shard_account: Cell,
    inbound_message: Option<Cell>,
}
```

Then make the public functions call the shared implementation:

```rust
pub async fn retrace_base_tx(
    net: Network,
    base_tx: BaseTxInfo,
    additional_libs: HashMap<HashBytes, Cell>,
) -> anyhow::Result<TraceResult> {
    retrace_base_tx_impl(net, base_tx, additional_libs)
        .await
        .map(|(result, _)| result)
}

pub async fn retrace_base_tx_with_state_flow(
    net: Network,
    base_tx: BaseTxInfo,
    additional_libs: HashMap<HashBytes, Cell>,
) -> anyhow::Result<StateReplayResult> {
    let tx_hash_hex = format!("0x{}", hex::encode(base_tx.hash));
    let (trace, artifacts) = retrace_base_tx_impl(net.clone(), base_tx, additional_libs).await?;
    Ok(StateReplayResult {
        trace,
        network: net.to_string(),
        tx_hash_hex,
        pre_shard_account: artifacts.pre_shard_account,
        post_shard_account: artifacts.post_shard_account,
        inbound_message: artifacts.inbound_message,
    })
}
```

- [ ] **Step 3: Keep existing `retrace` behavior unchanged**

Ensure the existing `retrace` and `retrace_base_tx` functions still return `TraceResult`. The new state-flow function should be additive and should not change `acton retrace` output.

- [ ] **Step 4: Export the new API**

In `crates/ton-retrace/src/lib.rs`, change the runner export to:

```rust
pub use crate::runner::{Network, retrace, retrace_base_tx, retrace_with_state_flow};
```

Change the type export to include:

```rust
StateReplayResult,
```

- [ ] **Step 5: Run retrace crate tests**

Run:

```bash
cargo test -p ton-retrace
```

Expected:

```text
test result: ok
```

- [ ] **Step 6: Commit**

```bash
git add crates/ton-retrace/src/types.rs crates/ton-retrace/src/runner.rs crates/ton-retrace/src/lib.rs
git commit -m "feat(reverse): expose retrace state replay artifacts"
```

---

### Task 4: Convert Replay Artifacts into `StateFlowTx`

**Files:**
- Create: `crates/ton-stateflow/src/convert.rs`
- Test: `crates/ton-stateflow/src/convert.rs`

- [ ] **Step 1: Add conversion helpers**

Create `crates/ton-stateflow/src/convert.rs`:

```rust
use crate::model::{
    AccountStateSnapshot, ActionSummary, CellRef, InboundMessageFlow, MessageBodyInfo,
    RecvInternalEvidence, StateFlowTx, StateFlowVersion, StorageDiff, TxIdentity, VmTraceInfo,
};
use thiserror::Error;
use ton_retrace::StateReplayResult;
use ton_retrace::trace::{ExecutedActions, Trace};
use tycho_types::boc::Boc;
use tycho_types::cell::Cell;
use tycho_types::models::{AccountState, MsgInfo, ShardAccount};

#[derive(Debug, Error)]
pub enum StateFlowConvertError {
    #[error("failed to serialize cell: {0}")]
    Cell(String),
    #[error("failed to parse account state: {0}")]
    Account(String),
}

pub fn state_flow_from_retrace(
    replay: StateReplayResult,
) -> Result<StateFlowTx, StateFlowConvertError> {
    let trace = replay.trace;
    let vm_trace = Trace::new(&trace.emulated_tx.vm_logs, None);
    let executed = ExecutedActions::from(&trace.emulated_tx.executor_logs);

    let inbound = inbound_message_flow(&trace, replay.inbound_message.as_ref());
    let pre = account_snapshot(&replay.pre_shard_account)?;
    let post = account_snapshot(&replay.post_shard_account)?;
    let storage_diff = storage_diff(&pre, &post);

    Ok(StateFlowTx {
        schema_version: StateFlowVersion::V1,
        identity: TxIdentity {
            network: replay.network,
            hash: replay.tx_hash_hex,
            lt: trace.emulated_tx.lt.to_string(),
            account: trace.in_msg.contract.to_string(),
            utime: trace.emulated_tx.utime,
            state_update_hash_ok: trace.state_update_hash_ok,
        },
        inbound,
        pre,
        post,
        storage_diff,
        vm_trace: VmTraceInfo {
            raw_log: trace.emulated_tx.vm_logs.to_string(),
            steps: vm_trace.steps.len(),
            final_c5: trace.emulated_tx.c5.as_ref().map(cell_ref),
            field_reads: vec![],
        },
        recv_internal: RecvInternalEvidence {
            entrypoint: "recv_internal".to_owned(),
            opcode: trace.in_msg.opcode.map(|opcode| format!("0x{opcode:08x}")),
            body_parse: vec![],
            schema_sketch: None,
        },
        executor_logs: trace.emulated_tx.executor_logs.to_string(),
        c5: trace.emulated_tx.c5.as_ref().map(cell_ref),
        installed_actions: vec![],
        executed_actions: executed
            .actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| ActionSummary {
                kind: format!("{action:?}"),
                index,
                hash: None,
                body: None,
                effect: "executor_action".to_owned(),
                failure_code: None,
            })
            .collect(),
        out_messages: vec![],
        abi_decode: None,
        schema_candidates: vec![],
    })
}

fn storage_diff(pre: &AccountStateSnapshot, post: &AccountStateSnapshot) -> StorageDiff {
    let before = pre.balance.parse::<i128>().unwrap_or(0);
    let after = post.balance.parse::<i128>().unwrap_or(0);
    StorageDiff {
        data_hash_changed: pre.data_hash != post.data_hash,
        balance_delta: (after - before).to_string(),
        changed_fields: vec![],
    }
}

fn inbound_message_flow(
    trace: &ton_retrace::TraceResult,
    inbound_message: Option<&Cell>,
) -> InboundMessageFlow {
    let mut bounced = false;
    let mut body = None;
    let mut body_hash = None;

    if let Some(message_cell) = inbound_message {
        if let Ok(message) = message_cell.parse::<tycho_types::models::Message<'_>>() {
            if let MsgInfo::Int(info) = &message.info {
                bounced = info.bounced;
            }
            body = Some(cell_ref(&message.body.1));
            body_hash = Some(format_hash(message.body.1.repr_hash().as_slice()));
        }
    }

    InboundMessageFlow {
        sender: trace.in_msg.sender.as_ref().map(ToString::to_string),
        destination: trace.in_msg.contract.to_string(),
        value: trace.in_msg.amount.map(|value| value.to_string()),
        bounced,
        raw_message: inbound_message.map(cell_ref),
        body: MessageBodyInfo {
            opcode: trace.in_msg.opcode.map(|opcode| format!("0x{opcode:08x}")),
            body,
            body_hash,
        },
    }
}

fn account_snapshot(cell: &Cell) -> Result<AccountStateSnapshot, StateFlowConvertError> {
    let shard_account = cell
        .parse::<ShardAccount>()
        .map_err(|err| StateFlowConvertError::Account(err.to_string()))?;
    let account = shard_account
        .load_account()
        .map_err(|err| StateFlowConvertError::Account(err.to_string()))?;

    let (code, data, code_hash, data_hash, balance) = match account {
        Some(account) => {
            let balance = u128::from(account.balance.tokens).to_string();
            match account.state {
                AccountState::Active(state) => {
                    let code_hash = state.code.as_ref().map(|cell| format_hash(cell.repr_hash().as_slice()));
                    let data_hash = state.data.as_ref().map(|cell| format_hash(cell.repr_hash().as_slice()));
                    (
                        state.code.as_ref().map(cell_ref),
                        state.data.as_ref().map(cell_ref),
                        code_hash,
                        data_hash,
                        balance,
                    )
                }
                _ => (None, None, None, None, balance),
            }
        }
        None => (None, None, None, None, "0".to_owned()),
    };

    Ok(AccountStateSnapshot {
        shard_account: cell_ref(cell),
        code,
        data,
        balance,
        data_hash,
        code_hash,
    })
}

fn cell_ref(cell: &Cell) -> CellRef {
    CellRef {
        boc64: Boc::encode_base64(cell.clone()),
        hash: format_hash(cell.repr_hash().as_slice()),
    }
}

fn format_hash(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}
```

- [ ] **Step 2: Run conversion crate tests**

Run:

```bash
cargo test -p ton-stateflow
```

Expected:

```text
test result: ok
```

- [ ] **Step 3: Commit**

```bash
git add crates/ton-stateflow
git commit -m "feat(reverse): convert retrace artifacts to state flow json"
```

---

### Task 5: Add `acton reverse retrace`

**Files:**
- Create: `src/commands/reverse/mod.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/bin/acton.rs`

- [ ] **Step 1: Create the reverse command module**

Create `src/commands/reverse/mod.rs`:

```rust
use anyhow::Context;
use clap::Subcommand;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use ton_retrace::{Network, retrace_with_state_flow};
use ton_stateflow::state_flow_from_retrace;

#[derive(Subcommand, Clone)]
pub enum ReverseCommand {
    #[command(about = "Retrace a transaction and write a state-flow JSON artifact")]
    Retrace {
        #[arg(help = "Transaction hash in hex form")]
        hash: String,
        #[arg(long, help = "Network to query: mainnet or testnet")]
        net: String,
        #[arg(long, value_name = "PATH", help = "Output JSON file")]
        out: PathBuf,
    },
}

pub fn reverse_cmd(command: ReverseCommand) -> anyhow::Result<()> {
    match command {
        ReverseCommand::Retrace { hash, net, out } => reverse_retrace_cmd(hash, net, out),
    }
}

fn reverse_retrace_cmd(hash: String, net: String, out: PathBuf) -> anyhow::Result<()> {
    let network = Network::from_str(&net)?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let replay = rt.block_on(retrace_with_state_flow(network, &hash, HashMap::new()))?;
    let state_flow = state_flow_from_retrace(replay)?;
    let json = serde_json::to_string_pretty(&state_flow)?;

    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }
    fs::write(&out, json).with_context(|| format!("Failed to write {}", out.display()))?;
    println!("Wrote state-flow JSON to {}", out.display());
    Ok(())
}
```

- [ ] **Step 2: Export the command module**

In `src/commands/mod.rs`, add:

```rust
pub mod reverse;
```

- [ ] **Step 3: Wire the CLI enum**

In `src/bin/acton.rs`, add imports:

```rust
use acton::commands::reverse::{ReverseCommand, reverse_cmd};
```

Add a top-level command variant near `Retrace`:

```rust
    #[command(
        about = "Reverse-engineer TON contract state flows",
        after_help = detailed_help_pointer("reverse")
    )]
    Reverse {
        #[command(subcommand)]
        command: ReverseCommand,
    },
```

Add dispatch in the main match:

```rust
        Commands::Reverse { command } => {
            reverse_cmd(command)?;
        }
```

Add `Commands::Reverse { .. }` to any helper match that groups project-aware commands in the same way as `Commands::Rpc { .. }` if the compiler points to an exhaustiveness error.

- [ ] **Step 4: Run CLI help**

Run:

```bash
cargo run -q -- reverse --help
```

Expected:

```text
Reverse-engineer TON contract state flows
```

- [ ] **Step 5: Run formatter and targeted compile**

Run:

```bash
cargo fmt
cargo check -p acton
```

Expected:

```text
Finished `dev` profile
```

- [ ] **Step 6: Commit**

```bash
git add src/commands/mod.rs src/commands/reverse src/bin/acton.rs Cargo.toml crates/ton-stateflow crates/ton-retrace
git commit -m "feat(reverse): add state-flow retrace command"
```

---

### Task 6: Add CLI Help Test Coverage

**Files:**
- Modify: `tests/integration_test.rs`
- Add snapshot files generated by the existing test harness if required.

- [ ] **Step 1: Add a help test**

In `tests/integration_test.rs`, add a test near the existing help tests:

```rust
#[test]
fn test_acton_reverse_help() {
    Command::cargo_bin("acton")
        .expect("binary exists")
    snapbox::cmd::Command::acton_ui()
        .args(["reverse", "--help"])
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/reverse/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}
```

- [ ] **Step 2: Run the help test**

Run:

```bash
cargo test --test integration_test test_acton_reverse_help
```

Expected:

```text
test test_acton_reverse_help ... ok
```

- [ ] **Step 3: Commit**

```bash
git add tests/integration_test.rs tests/integration/snapshots tests/snapshots
git commit -m "test(reverse): cover reverse help"
```

---

### Task 7: Baseline and Manual Smoke Test

**Files:**
- No source edits unless test failures reveal missing compile fixes.

- [ ] **Step 1: Run targeted checks**

Run:

```bash
cargo test -p ton-stateflow
cargo check -p acton
cargo test --test integration_test test_acton_reverse_help
```

Expected:

```text
test result: ok
Finished `dev` profile
test test_acton_reverse_help ... ok
```

- [ ] **Step 2: Run a network smoke test only when API access is configured**

Check:

```bash
env | rg '^TONCENTER_(MAINNET|TESTNET)_API_KEY='
```

If a key is present, run:

```bash
cargo run -q -- reverse retrace 3c1b02a33390e596d83b306eab57b3f7271bc90e2e527ea4cafccfde25139d41 --net mainnet --out /tmp/acton-stateflow-smoke.json
jq '.schema_version, .identity, .inbound.body.opcode, .pre.balance, .post.balance' /tmp/acton-stateflow-smoke.json
```

Expected:

```text
"v1"
{
  ...
}
```

If no key is present, record in the final message:

```text
Network smoke test skipped because TONCENTER_MAINNET_API_KEY / TONCENTER_TESTNET_API_KEY is not set.
```

- [ ] **Step 3: Push the branch**

Run:

```bash
git status --short
git push -u origin feat/ton-reverse-stateflow
```

Expected:

```text
branch 'feat/ton-reverse-stateflow' set up to track 'origin/feat/ton-reverse-stateflow'
```

---

## Follow-on Plans

After Phase 1 is merged, create separate plans:

1. `ton-reverse-collect`: address history collection into corpus directories.
2. `ton-schema-infer`: opcode clustering and body-field candidates.
3. `ton-replay-lab`: message construction, mutation, local replay, and state/effect diff.
4. `ton-reverse-report`: Markdown/JSON/graph reports over corpus plus schemas.
5. `acton-test-ui-stateflow`: State Flow / Schema / Replay UI views over the stable JSON artifacts.

## Self-Review Notes

- The first milestone avoids UI work and focuses on CLI plus JSON artifacts.
- The plan preserves existing `acton retrace` behavior by adding new `ton-retrace` APIs instead of changing current return types.
- `StateFlowTx` includes the required fields: transaction identity, inbound raw/body/op, pre/post state, VM trace, executor logs, c5, installed/executed actions, out messages, ABI decode section, and schema candidate section.
- Later reverse-engineering features are represented as follow-on plans so the first implementation stays testable.
