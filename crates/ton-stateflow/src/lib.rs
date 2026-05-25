use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use ton_retrace::{
    AccountTxRef, ComputeInfo, Network, ReplayTransactionArgs, ReplayTransactionResult,
    ReplayTransactionSuccess, TraceResult,
};
use tycho_types::boc::Boc;
use tycho_types::cell::{Cell, CellBuilder, CellFamily, CellSlice, HashBytes, Store};
use tycho_types::models::{
    AccountState, IntAddr, MsgInfo, OutAction, OwnedMessage, RelaxedMsgInfo, ShardAccount,
};

pub const STATE_FLOW_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowTx {
    pub schema_version: u32,
    pub network: String,
    pub query_hash: String,
    pub transaction: TransactionIdentity,
    pub replay: ReplaySummary,
    pub state: StateTransition,
    pub inbound: MessageArtifact,
    pub outbound: Vec<MessageArtifact>,
    pub compute: StateFlowCompute,
    pub money: MoneyFlow,
    pub c5: Option<CellArtifact>,
    pub out_actions: Vec<ActionEffect>,
    pub vm_trace: LogArtifact,
    pub executor_trace: LogArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowCorpus {
    pub schema_version: u32,
    pub network: String,
    pub address: String,
    pub requested_limit: u32,
    pub source_tx_count: usize,
    pub retraced_count: usize,
    pub failure_count: usize,
    pub opcode_summary: Vec<OpcodeSummary>,
    pub transactions: Vec<StateFlowTx>,
    pub failures: Vec<StateFlowFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpcodeSummary {
    pub opcode: Option<String>,
    pub count: usize,
    pub tx_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowFailure {
    pub hash: String,
    pub lt: u64,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowSchemaReport {
    pub schema_version: u32,
    pub network: String,
    pub address: String,
    pub transaction_count: usize,
    #[serde(default)]
    pub state_machine: StateMachineGraph,
    #[serde(default)]
    pub audit_signals: Vec<AuditSignal>,
    pub opcode_candidates: Vec<OpcodeSchemaCandidate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateMachineGraph {
    pub edges: Vec<StateMachineEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateMachineEdge {
    pub from_status: String,
    pub to_status: String,
    pub opcode: Option<String>,
    pub count: usize,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSignal {
    pub kind: String,
    pub severity: String,
    pub description: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowReplayDiff {
    pub schema_version: u32,
    pub source_query_hash: String,
    pub mutation: ReplayMutation,
    pub ignore_chksig: bool,
    pub baseline: ReplayObservation,
    pub replay: ReplayObservation,
    pub diff: ReplayDiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ReplayMutation {
    None,
    FlipBodyBit {
        bit: u16,
    },
    ReplaceBody {
        #[serde(rename = "bodyBoc64")]
        body_boc64: String,
    },
    SetBodyUint {
        #[serde(rename = "bitOffset")]
        bit_offset: u16,
        bits: u16,
        value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayObservation {
    pub accepted: bool,
    pub state: Option<ShardAccountSnapshot>,
    pub inbound: MessageArtifact,
    pub outbound: Vec<MessageArtifact>,
    pub compute: Option<StateFlowCompute>,
    pub money: Option<MoneyFlow>,
    pub c5: Option<CellArtifact>,
    pub out_actions: Vec<ActionEffect>,
    pub vm_trace: Option<LogArtifact>,
    pub executor_trace: Option<LogArtifact>,
    pub error: Option<ReplayErrorArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayErrorArtifact {
    pub message: String,
    pub external_not_accepted: bool,
    pub vm_exit_code: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayDiffSummary {
    pub replay_accepted: bool,
    pub input_changed: bool,
    pub state_changed: Option<bool>,
    pub code_hash_changed: Option<bool>,
    pub data_hash_changed: Option<bool>,
    pub balance_delta_diff: Option<i128>,
    pub exit_code_changed: Option<bool>,
    pub outbound_count_delta: Option<i64>,
    pub action_count_delta: Option<i64>,
    pub c5_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpcodeSchemaCandidate {
    pub opcode: Option<String>,
    pub count: usize,
    pub examples: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<SchemaEvidence>,
    pub inbound_body: BodyShapeCandidate,
    #[serde(default)]
    pub replay_probes: Vec<ReplayProbeCandidate>,
    #[serde(default)]
    pub storage: StorageShapeCandidate,
    pub state_transitions: Vec<StateTransitionCandidate>,
    pub outbound_effects: Vec<EffectCandidate>,
    pub out_actions: Vec<EffectCandidate>,
    pub confidence: String,
    pub unknown_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayProbeCandidate {
    pub field_name: String,
    pub bit_offset: u16,
    pub bits: u16,
    pub value: String,
    pub mutation: ReplayMutation,
    pub cli_arg: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaEvidence {
    pub tx_hash: String,
    pub inbound_body_hash: String,
    pub inbound_body_bits: u16,
    pub inbound_body_refs: u8,
    pub from_status: String,
    pub to_status: String,
    pub pre_data_hash: Option<String>,
    pub post_data_hash: Option<String>,
    pub pre_code_hash: Option<String>,
    pub post_code_hash: Option<String>,
    pub outbound_kinds: Vec<String>,
    pub out_action_kinds: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageShapeCandidate {
    pub balance_delta_min: i128,
    pub balance_delta_max: i128,
    pub data_hash_changed_count: usize,
    pub code_hash_changed_count: usize,
    #[serde(default)]
    pub post_data_shape: Option<CellShapeRange>,
    #[serde(default)]
    pub post_code_shape: Option<CellShapeRange>,
    #[serde(default)]
    pub fields: Vec<StorageFieldCandidate>,
    pub post_data_hashes: Vec<String>,
    pub post_code_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageFieldCandidate {
    pub name: String,
    pub cell_path: String,
    pub bit_offset: u16,
    pub min_bits: u16,
    pub max_bits: u16,
    pub min_refs: u8,
    pub max_refs: u8,
    pub kind: String,
    pub present_count: usize,
    pub value_samples: Vec<String>,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellShapeRange {
    pub min_bits: u16,
    pub max_bits: u16,
    pub min_refs: u8,
    pub max_refs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyShapeCandidate {
    pub min_bits: u16,
    pub max_bits: u16,
    pub min_refs: u8,
    pub max_refs: u8,
    pub body_hashes: Vec<String>,
    #[serde(default)]
    pub field_candidates: Vec<BodyFieldCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyFieldCandidate {
    pub name: String,
    pub bit_offset: u16,
    pub min_bits: u16,
    pub max_bits: u16,
    pub min_refs: u8,
    pub max_refs: u8,
    pub kind: String,
    pub present_count: usize,
    pub value_samples: Vec<String>,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTransitionCandidate {
    pub from_status: String,
    pub to_status: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectCandidate {
    pub kind: String,
    pub count: usize,
    #[serde(default)]
    pub tx_hashes: Vec<String>,
    #[serde(default)]
    pub modes: Vec<String>,
    #[serde(default)]
    pub destinations: Vec<String>,
    pub value_nanotons_min: Option<String>,
    pub value_nanotons_max: Option<String>,
    #[serde(default)]
    pub body_shape: Option<CellShapeRange>,
    #[serde(default)]
    pub code_shape: Option<CellShapeRange>,
    #[serde(default)]
    pub library_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionIdentity {
    pub lt: u64,
    pub utime: u64,
    pub account: String,
    pub state_update_hash_ok: bool,
    pub transaction_boc64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySummary {
    pub mc_seqno: u32,
    pub rand_seed_hex: String,
    pub replayed_prev_tx_count: usize,
    pub block_config_boc64: String,
    #[serde(default)]
    pub libs_boc64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTransition {
    pub pre: ShardAccountSnapshot,
    pub post: ShardAccountSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardAccountSnapshot {
    pub shard_account_boc64: String,
    pub last_trans_lt: u64,
    pub last_trans_hash: String,
    pub account_address: Option<String>,
    pub status: String,
    pub balance_nanotons: String,
    pub code_hash: Option<String>,
    pub data_hash: Option<String>,
    #[serde(default)]
    pub code_cell: Option<CellShape>,
    #[serde(default)]
    pub data_cell: Option<CellShape>,
    pub frozen_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellShape {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boc64: Option<String>,
    pub hash: String,
    pub bits: u16,
    pub refs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageArtifact {
    pub direction: MessageDirection,
    pub index: Option<usize>,
    pub kind: String,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub value_nanotons: Option<String>,
    pub bounced: Option<bool>,
    pub bounce: Option<bool>,
    pub opcode: Option<String>,
    pub message_boc64: String,
    pub body: CellArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellArtifact {
    pub boc64: String,
    pub hash: String,
    pub bits: u16,
    pub refs: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowCompute {
    pub skipped: bool,
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
    pub vm_steps: Option<u32>,
    pub gas_used: Option<u64>,
    pub gas_fees: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyFlow {
    pub balance_before: u64,
    pub sent_total: u64,
    pub total_fees: u64,
    pub balance_after: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEffect {
    pub index: usize,
    pub kind: String,
    pub mode: Option<String>,
    pub value_nanotons: Option<String>,
    pub destination: Option<String>,
    pub body: Option<CellArtifact>,
    pub code: Option<CellArtifact>,
    pub library: Option<LibraryEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEffect {
    pub mode: String,
    pub hash: Option<String>,
    pub cell: Option<CellArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogArtifact {
    pub line_count: usize,
    pub text: String,
}

pub async fn retrace_with_state_flow(
    network: Network,
    hash: &str,
    additional_libs: HashMap<HashBytes, Cell>,
) -> anyhow::Result<StateFlowTx> {
    let result = ton_retrace::retrace(network.clone(), hash, additional_libs).await?;
    StateFlowTx::from_retrace_result(network.to_string(), hash, &result)
}

pub async fn collect_state_flow_corpus(
    network: Network,
    address: &str,
    limit: u32,
    additional_libs: HashMap<HashBytes, Cell>,
) -> anyhow::Result<StateFlowCorpus> {
    let tx_refs = ton_retrace::collect_account_transaction_refs(network.clone(), address, limit)
        .await
        .with_context(|| format!("failed to collect transactions for {address}"))?;

    let mut transactions = Vec::new();
    let mut failures = Vec::new();
    for tx_ref in &tx_refs {
        match retrace_with_state_flow(network.clone(), &tx_ref.hash, additional_libs.clone()).await
        {
            Ok(flow) => transactions.push(flow),
            Err(err) => failures.push(StateFlowFailure::from_error(tx_ref, err)),
        }
    }

    Ok(StateFlowCorpus {
        schema_version: STATE_FLOW_SCHEMA_VERSION,
        network: network.to_string(),
        address: address.to_owned(),
        requested_limit: limit,
        source_tx_count: tx_refs.len(),
        retraced_count: transactions.len(),
        failure_count: failures.len(),
        opcode_summary: opcode_summary(&transactions),
        transactions,
        failures,
    })
}

pub fn infer_schema_candidates(corpus: &StateFlowCorpus) -> StateFlowSchemaReport {
    let mut by_opcode = BTreeMap::<Option<String>, Vec<&StateFlowTx>>::new();
    for tx in &corpus.transactions {
        by_opcode
            .entry(tx.inbound.opcode.clone())
            .or_default()
            .push(tx);
    }
    let opcode_candidates: Vec<_> = by_opcode
        .into_iter()
        .map(|(opcode, transactions)| opcode_candidate(opcode, &transactions))
        .collect();

    StateFlowSchemaReport {
        schema_version: STATE_FLOW_SCHEMA_VERSION,
        network: corpus.network.clone(),
        address: corpus.address.clone(),
        transaction_count: corpus.transactions.len(),
        state_machine: state_machine_graph(&corpus.transactions),
        audit_signals: infer_schema_audit_signals(corpus, &opcode_candidates),
        opcode_candidates,
    }
}

pub fn replay_state_flow_tx(
    flow: &StateFlowTx,
    mutation: ReplayMutation,
    ignore_chksig: bool,
) -> anyhow::Result<StateFlowReplayDiff> {
    let message_boc64 = apply_replay_mutation(&flow.inbound.message_boc64, &mutation)?;
    let replay_inbound = inbound_artifact_from_boc64(&message_boc64)?;
    let result = ton_retrace::replay_transaction(replay_args_from_flow(
        flow,
        message_boc64.clone(),
        ignore_chksig,
    ))?;

    let baseline = ReplayObservation::from_flow(flow);
    let replay = match result {
        ReplayTransactionResult::Success(success) => {
            ReplayObservation::from_success(&message_boc64, replay_inbound, &success)?
        }
        ReplayTransactionResult::Error(error) => ReplayObservation {
            accepted: false,
            state: None,
            inbound: replay_inbound,
            outbound: Vec::new(),
            compute: None,
            money: None,
            c5: None,
            out_actions: Vec::new(),
            vm_trace: error.vm_logs.as_deref().map(LogArtifact::from),
            executor_trace: error.executor_logs.as_deref().map(LogArtifact::from),
            error: Some(ReplayErrorArtifact {
                message: error.error,
                external_not_accepted: error.external_not_accepted,
                vm_exit_code: error.vm_exit_code,
            }),
        },
    };
    let diff = ReplayDiffSummary::compare(&baseline, &replay);

    Ok(StateFlowReplayDiff {
        schema_version: STATE_FLOW_SCHEMA_VERSION,
        source_query_hash: flow.query_hash.clone(),
        mutation,
        ignore_chksig,
        baseline,
        replay,
        diff,
    })
}

fn replay_args_from_flow(
    flow: &StateFlowTx,
    message_boc64: String,
    ignore_chksig: bool,
) -> ReplayTransactionArgs {
    ReplayTransactionArgs {
        message_boc64,
        shard_account_boc64: flow.state.pre.shard_account_boc64.clone(),
        block_config_boc64: flow.replay.block_config_boc64.clone(),
        rand_seed_hex: flow.replay.rand_seed_hex.clone(),
        now: flow.transaction.utime.try_into().unwrap_or(u32::MAX),
        lt: flow.transaction.lt,
        libs_boc64: flow.replay.libs_boc64.clone(),
        ignore_chksig,
    }
}

pub fn render_state_flow_report(
    corpus: &StateFlowCorpus,
    schema: &StateFlowSchemaReport,
    replays: &[StateFlowReplayDiff],
) -> String {
    let mut report = String::new();
    writeln!(report, "# TON State Flow Reverse Report").ok();
    writeln!(report).ok();
    writeln!(report, "## Target").ok();
    writeln!(report, "- Network: `{}`", corpus.network).ok();
    writeln!(report, "- Address: `{}`", corpus.address).ok();
    writeln!(report, "- Source transactions: {}", corpus.source_tx_count).ok();
    writeln!(report, "- Retraced transactions: {}", corpus.retraced_count).ok();
    writeln!(
        report,
        "- Replay failures while collecting: {}",
        corpus.failure_count
    )
    .ok();
    writeln!(report, "- Replay diffs: {}", replays.len()).ok();
    writeln!(report).ok();

    writeln!(report, "## Opcode Candidates").ok();
    writeln!(
        report,
        "| Opcode | Count | Confidence | Body bits | Body refs | Storage | State transitions | Outbound effects | Out actions | Evidence |"
    )
    .ok();
    writeln!(
        report,
        "| --- | ---: | --- | --- | --- | --- | --- | --- | --- | --- |"
    )
    .ok();
    for candidate in &schema.opcode_candidates {
        writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            markdown_code_opt(candidate.opcode.as_deref()),
            candidate.count,
            candidate.confidence,
            format_range(
                candidate.inbound_body.min_bits,
                candidate.inbound_body.max_bits
            ),
            format_range(
                candidate.inbound_body.min_refs,
                candidate.inbound_body.max_refs
            ),
            markdown_escape(&format_storage(&candidate.storage)),
            markdown_escape(&format_state_transitions(&candidate.state_transitions)),
            markdown_escape(&format_effects(&candidate.outbound_effects)),
            markdown_escape(&format_effects(&candidate.out_actions)),
            markdown_escape(&candidate.examples.join(", ")),
        )
        .ok();
    }
    writeln!(report).ok();

    writeln!(report, "## Schema Evidence").ok();
    let mut evidence_rows = 0;
    for candidate in &schema.opcode_candidates {
        evidence_rows += candidate.evidence.len();
    }
    if evidence_rows == 0 {
        writeln!(report, "- No compact schema evidence rows were inferred.").ok();
    } else {
        writeln!(
            report,
            "| Opcode | Tx | Body hash | Body bits/refs | State | Data hash | Code hash | Outbound | Actions |"
        )
        .ok();
        writeln!(
            report,
            "| --- | --- | --- | ---: | --- | --- | --- | --- | --- |"
        )
        .ok();
        for candidate in &schema.opcode_candidates {
            for evidence in &candidate.evidence {
                writeln!(
                    report,
                    "| {} | `{}` | {} | {}/{} | {} -> {} | {} | {} | {} | {} |",
                    markdown_code_opt(candidate.opcode.as_deref()),
                    markdown_escape(&evidence.tx_hash),
                    markdown_code_opt(Some(&evidence.inbound_body_hash)),
                    evidence.inbound_body_bits,
                    evidence.inbound_body_refs,
                    markdown_escape(&evidence.from_status),
                    markdown_escape(&evidence.to_status),
                    format_hash_transition(&evidence.pre_data_hash, &evidence.post_data_hash),
                    format_hash_transition(&evidence.pre_code_hash, &evidence.post_code_hash),
                    markdown_escape(&format_kind_list(&evidence.outbound_kinds)),
                    markdown_escape(&format_kind_list(&evidence.out_action_kinds)),
                )
                .ok();
            }
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Message Body Fields").ok();
    let mut body_field_rows = 0;
    for candidate in &schema.opcode_candidates {
        body_field_rows += candidate.inbound_body.field_candidates.len();
    }
    if body_field_rows == 0 {
        writeln!(report, "- No message body field candidates were inferred.").ok();
    } else {
        writeln!(
            report,
            "| Opcode | Field | Offset | Bits | Refs | Kind | Samples | Confidence |"
        )
        .ok();
        writeln!(report, "| --- | --- | ---: | --- | --- | --- | --- | --- |").ok();
        for candidate in &schema.opcode_candidates {
            for field in &candidate.inbound_body.field_candidates {
                writeln!(
                    report,
                    "| {} | `{}` | {} | {} | {} | {} | {} | {} |",
                    markdown_code_opt(candidate.opcode.as_deref()),
                    markdown_escape(&field.name),
                    field.bit_offset,
                    format_field_range(field.min_bits, field.max_bits),
                    format_field_range(field.min_refs, field.max_refs),
                    markdown_escape(&field.kind),
                    markdown_code_list(&field.value_samples),
                    markdown_escape(&field.confidence),
                )
                .ok();
            }
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Replay Probes").ok();
    let mut probe_rows = 0;
    for candidate in &schema.opcode_candidates {
        probe_rows += candidate.replay_probes.len();
    }
    if probe_rows == 0 {
        writeln!(report, "- No replay probe candidates were inferred.").ok();
    } else {
        writeln!(
            report,
            "| Opcode | Field | CLI mutation | Confidence | Evidence |"
        )
        .ok();
        writeln!(report, "| --- | --- | --- | --- | --- |").ok();
        for candidate in &schema.opcode_candidates {
            for probe in &candidate.replay_probes {
                writeln!(
                    report,
                    "| {} | `{}` | `{}` | {} | {} |",
                    markdown_code_opt(candidate.opcode.as_deref()),
                    markdown_escape(&probe.field_name),
                    markdown_escape(&probe.cli_arg),
                    markdown_escape(&probe.confidence),
                    markdown_code_list(&probe.evidence),
                )
                .ok();
            }
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Storage Fields").ok();
    let mut storage_field_rows = 0;
    for candidate in &schema.opcode_candidates {
        storage_field_rows += candidate.storage.fields.len();
    }
    if storage_field_rows == 0 {
        writeln!(report, "- No storage field candidates were inferred.").ok();
    } else {
        writeln!(
            report,
            "| Opcode | Field | Cell | Offset | Bits | Refs | Kind | Samples | Confidence |"
        )
        .ok();
        writeln!(
            report,
            "| --- | --- | --- | ---: | --- | --- | --- | --- | --- |"
        )
        .ok();
        for candidate in &schema.opcode_candidates {
            for field in &candidate.storage.fields {
                writeln!(
                    report,
                    "| {} | `{}` | {} | {} | {} | {} | {} | {} | {} |",
                    markdown_code_opt(candidate.opcode.as_deref()),
                    markdown_escape(&field.name),
                    markdown_escape(&field.cell_path),
                    field.bit_offset,
                    format_field_range(field.min_bits, field.max_bits),
                    format_field_range(field.min_refs, field.max_refs),
                    markdown_escape(&field.kind),
                    markdown_code_list(&field.value_samples),
                    markdown_escape(&field.confidence),
                )
                .ok();
            }
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Outbound Effects").ok();
    let mut effect_rows = 0;
    for candidate in &schema.opcode_candidates {
        effect_rows += candidate.outbound_effects.len() + candidate.out_actions.len();
    }
    if effect_rows == 0 {
        writeln!(report, "- No outbound effect candidates were inferred.").ok();
    } else {
        writeln!(
            report,
            "| Opcode | Source | Kind | Count | Value | Modes | Destinations | Body | Code | Libraries | Evidence |"
        )
        .ok();
        writeln!(
            report,
            "| --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- | --- |"
        )
        .ok();
        for candidate in &schema.opcode_candidates {
            for effect in &candidate.outbound_effects {
                write_effect_row(&mut report, candidate.opcode.as_deref(), "outbound", effect);
            }
            for effect in &candidate.out_actions {
                write_effect_row(&mut report, candidate.opcode.as_deref(), "action", effect);
            }
        }
    }
    writeln!(report).ok();

    writeln!(report, "## State Machine").ok();
    let state_machine = render_state_machine(schema);
    if state_machine.is_empty() {
        writeln!(report, "- No state transitions were inferred.").ok();
    } else {
        writeln!(report, "```mermaid").ok();
        writeln!(report, "stateDiagram-v2").ok();
        for line in state_machine {
            writeln!(report, "    {line}").ok();
        }
        writeln!(report, "```").ok();
    }
    writeln!(report).ok();

    writeln!(report, "## State Machine Evidence").ok();
    if schema.state_machine.edges.is_empty() {
        writeln!(report, "- No state-machine edge evidence was inferred.").ok();
    } else {
        writeln!(
            report,
            "| From | To | Opcode | Count | Confidence | Evidence |"
        )
        .ok();
        writeln!(report, "| --- | --- | --- | ---: | --- | --- |").ok();
        for edge in &schema.state_machine.edges {
            writeln!(
                report,
                "| {} | {} | {} | {} | {} | {} |",
                markdown_escape(&edge.from_status),
                markdown_escape(&edge.to_status),
                markdown_code_opt(edge.opcode.as_deref()),
                edge.count,
                state_machine_edge_confidence(edge.count),
                markdown_code_list(&edge.examples),
            )
            .ok();
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Unknown Fields").ok();
    if schema.opcode_candidates.is_empty() {
        writeln!(report, "- No opcode candidates were inferred.").ok();
    }
    for candidate in &schema.opcode_candidates {
        writeln!(
            report,
            "- {}:",
            markdown_code_opt(candidate.opcode.as_deref())
        )
        .ok();
        for field in &candidate.unknown_fields {
            writeln!(
                report,
                "  - {} (confidence: {}; evidence: {})",
                field,
                markdown_escape(&candidate.confidence),
                markdown_code_list(&candidate.examples)
            )
            .ok();
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Replay Diffs").ok();
    if replays.is_empty() {
        writeln!(report, "- No replay diff artifacts were provided.").ok();
    } else {
        writeln!(
            report,
            "| Source tx | Mutation | Accepted | Input changed | State changed | Exit changed | Outbound delta | Action delta |"
        )
        .ok();
        writeln!(
            report,
            "| --- | --- | --- | --- | --- | --- | ---: | ---: |"
        )
        .ok();
        for replay in replays {
            writeln!(
                report,
                "| `{}` | {} | {} | {} | {} | {} | {} | {} |",
                replay.source_query_hash,
                markdown_escape(&mutation_label(&replay.mutation)),
                replay.diff.replay_accepted,
                replay.diff.input_changed,
                format_optional_bool(replay.diff.state_changed),
                format_optional_bool(replay.diff.exit_code_changed),
                replay
                    .diff
                    .outbound_count_delta
                    .map_or("n/a".to_owned(), |value| value.to_string()),
                replay
                    .diff
                    .action_count_delta
                    .map_or("n/a".to_owned(), |value| value.to_string()),
            )
            .ok();
        }
    }
    writeln!(report).ok();

    writeln!(report, "## Risk Points").ok();
    let risk_points = infer_risk_points(corpus, schema, replays);
    if risk_points.is_empty() {
        writeln!(
            report,
            "- No risk points were inferred from the provided artifacts."
        )
        .ok();
    } else {
        for risk in risk_points {
            writeln!(report, "- {risk}").ok();
        }
    }
    writeln!(report).ok();

    if !corpus.failures.is_empty() {
        writeln!(report, "## Collection Failures").ok();
        for failure in &corpus.failures {
            writeln!(
                report,
                "- `{}` at lt {}: {}",
                failure.hash, failure.lt, failure.error
            )
            .ok();
        }
        writeln!(report).ok();
    }

    report
}

fn render_state_machine(schema: &StateFlowSchemaReport) -> Vec<String> {
    let mut lines = Vec::new();
    if schema.state_machine.edges.is_empty() {
        for candidate in &schema.opcode_candidates {
            let opcode = candidate.opcode.as_deref().unwrap_or("<none>");
            for transition in &candidate.state_transitions {
                lines.push(format!(
                    "{} --> {}: {} ({})",
                    mermaid_state_id(&transition.from_status),
                    mermaid_state_id(&transition.to_status),
                    mermaid_label(opcode),
                    transition.count,
                ));
            }
        }
    } else {
        for edge in &schema.state_machine.edges {
            let opcode = edge.opcode.as_deref().unwrap_or("<none>");
            lines.push(format!(
                "{} --> {}: {} ({})",
                mermaid_state_id(&edge.from_status),
                mermaid_state_id(&edge.to_status),
                mermaid_label(opcode),
                edge.count,
            ));
        }
    }
    lines.sort();
    lines.dedup();
    lines
}

fn state_machine_edge_confidence(count: usize) -> &'static str {
    match count {
        3.. => "high",
        2 => "medium",
        _ => "low",
    }
}

fn infer_risk_points(
    corpus: &StateFlowCorpus,
    schema: &StateFlowSchemaReport,
    replays: &[StateFlowReplayDiff],
) -> Vec<String> {
    let schema_signals = if schema.audit_signals.is_empty() {
        infer_schema_audit_signals(corpus, &schema.opcode_candidates)
    } else {
        schema.audit_signals.clone()
    };
    let mut risks: Vec<_> = schema_signals
        .iter()
        .map(format_audit_signal_risk)
        .collect();

    for replay in replays {
        let mutation = markdown_escape(&mutation_label(&replay.mutation));
        if replay.diff.input_changed && replay.diff.replay_accepted {
            if replay.diff.state_changed == Some(true) {
                risks.push(format!(
                    "Mutation `{mutation}` changed state for `{}`.",
                    replay.source_query_hash
                ));
            }
            if replay.diff.outbound_count_delta.unwrap_or_default() != 0
                || replay.diff.action_count_delta.unwrap_or_default() != 0
            {
                risks.push(format!(
                    "Mutation `{mutation}` changed outbound/action counts for `{}`.",
                    replay.source_query_hash
                ));
            }
        } else if replay.diff.input_changed && !replay.diff.replay_accepted {
            risks.push(format!(
                "Mutation `{mutation}` was rejected for `{}`.",
                replay.source_query_hash
            ));
        }
    }

    risks.sort();
    risks.dedup();
    risks
}

fn state_machine_graph(transactions: &[StateFlowTx]) -> StateMachineGraph {
    let mut by_edge = BTreeMap::<(String, String, Option<String>), (usize, Vec<String>)>::new();
    for tx in transactions {
        let key = (
            tx.state.pre.status.clone(),
            tx.state.post.status.clone(),
            tx.inbound.opcode.clone(),
        );
        let entry = by_edge.entry(key).or_default();
        entry.0 += 1;
        entry.1.push(tx.query_hash.clone());
    }

    StateMachineGraph {
        edges: by_edge
            .into_iter()
            .map(
                |((from_status, to_status, opcode), (count, examples))| StateMachineEdge {
                    from_status,
                    to_status,
                    opcode,
                    count,
                    examples,
                },
            )
            .collect(),
    }
}

fn infer_schema_audit_signals(
    corpus: &StateFlowCorpus,
    candidates: &[OpcodeSchemaCandidate],
) -> Vec<AuditSignal> {
    let mut signals = Vec::new();
    if corpus.failure_count > 0 {
        signals.push(AuditSignal {
            kind: "collection-failure".to_owned(),
            severity: "medium".to_owned(),
            description: format!(
                "{} transaction(s) failed during collection and are absent from inference.",
                corpus.failure_count
            ),
            evidence: corpus
                .failures
                .iter()
                .map(|failure| failure.hash.clone())
                .collect(),
        });
    }

    for candidate in candidates {
        let opcode = plain_opcode_label(candidate.opcode.as_deref());
        if candidate.confidence == "low" {
            signals.push(AuditSignal {
                kind: "low-confidence-schema".to_owned(),
                severity: "medium".to_owned(),
                description: format!(
                    "Low-confidence schema candidate for opcode {opcode}; body shape varied or evidence is sparse."
                ),
                evidence: candidate.examples.clone(),
            });
        }
        if !candidate.unknown_fields.is_empty() {
            signals.push(AuditSignal {
                kind: "unknown-fields".to_owned(),
                severity: "medium".to_owned(),
                description: format!(
                    "Unknown fields remain for opcode {opcode}: {}.",
                    candidate.unknown_fields.join("; ")
                ),
                evidence: candidate.examples.clone(),
            });
        }
        if candidate.storage.data_hash_changed_count > 0 {
            signals.push(AuditSignal {
                kind: "storage-data-hash-change".to_owned(),
                severity: "medium".to_owned(),
                description: format!(
                    "Opcode {opcode} changed storage data hash in {} observed transaction(s).",
                    candidate.storage.data_hash_changed_count
                ),
                evidence: candidate.examples.clone(),
            });
        }
        if candidate.storage.code_hash_changed_count > 0 {
            signals.push(AuditSignal {
                kind: "storage-code-hash-change".to_owned(),
                severity: "high".to_owned(),
                description: format!(
                    "Opcode {opcode} changed code hash in {} observed transaction(s).",
                    candidate.storage.code_hash_changed_count
                ),
                evidence: candidate.examples.clone(),
            });
        }
        if !candidate.outbound_effects.is_empty() || !candidate.out_actions.is_empty() {
            signals.push(AuditSignal {
                kind: "outbound-or-action-effects".to_owned(),
                severity: "medium".to_owned(),
                description: format!(
                    "Opcode {opcode} produced outbound effects or c5 actions; payload fields still require TL-B recovery."
                ),
                evidence: candidate.examples.clone(),
            });
        }
    }

    signals.sort_by(|left, right| {
        (
            left.severity.as_str(),
            left.kind.as_str(),
            left.description.as_str(),
        )
            .cmp(&(
                right.severity.as_str(),
                right.kind.as_str(),
                right.description.as_str(),
            ))
    });
    signals.dedup_by(|left, right| {
        left.kind == right.kind
            && left.severity == right.severity
            && left.description == right.description
            && left.evidence == right.evidence
    });
    signals
}

fn format_audit_signal_risk(signal: &AuditSignal) -> String {
    if signal.evidence.is_empty() {
        return markdown_escape(&signal.description);
    }
    format!(
        "{} Evidence: {}.",
        markdown_escape(&signal.description),
        signal
            .evidence
            .iter()
            .map(|item| format!("`{}`", markdown_escape(item)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn plain_opcode_label(opcode: Option<&str>) -> String {
    opcode.unwrap_or("<none>").to_owned()
}

fn mermaid_state_id(value: &str) -> String {
    let mut id = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch);
        } else {
            id.push('_');
        }
    }
    if id.is_empty() {
        "unknown".to_owned()
    } else {
        id
    }
}

fn mermaid_label(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            ':' | '`' | '"' | '\n' | '\r' => ' ',
            _ => ch,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_range<T>(min: T, max: T) -> String
where
    T: Copy + Eq + std::fmt::Display,
{
    if min == max {
        min.to_string()
    } else {
        format!("{min}-{max}")
    }
}

fn format_field_range<T>(min: T, max: T) -> String
where
    T: Copy + std::fmt::Display,
{
    format!("{min}..{max}")
}

fn format_state_transitions(transitions: &[StateTransitionCandidate]) -> String {
    if transitions.is_empty() {
        return "none".to_owned();
    }
    transitions
        .iter()
        .map(|transition| {
            format!(
                "{} -> {} ({})",
                transition.from_status, transition.to_status, transition.count
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_effects(effects: &[EffectCandidate]) -> String {
    if effects.is_empty() {
        return "none".to_owned();
    }
    effects
        .iter()
        .map(|effect| format!("{} ({})", effect.kind, effect.count))
        .collect::<Vec<_>>()
        .join("; ")
}

fn write_effect_row(
    report: &mut String,
    opcode: Option<&str>,
    source: &str,
    effect: &EffectCandidate,
) {
    writeln!(
        report,
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        markdown_code_opt(opcode),
        markdown_escape(source),
        markdown_escape(&effect.kind),
        effect.count,
        format_effect_value(effect),
        markdown_code_list_or_none(&effect.modes),
        markdown_code_list_or_none(&effect.destinations),
        format_optional_shape(&effect.body_shape),
        format_optional_shape(&effect.code_shape),
        markdown_code_list_or_none(&effect.library_hashes),
        markdown_code_list_or_none(&effect.tx_hashes),
    )
    .ok();
}

fn format_effect_value(effect: &EffectCandidate) -> String {
    match (&effect.value_nanotons_min, &effect.value_nanotons_max) {
        (Some(min), Some(max)) if min == max => markdown_escape(min),
        (Some(min), Some(max)) => markdown_escape(&format!("{min}..{max}")),
        _ => "n/a".to_owned(),
    }
}

fn format_optional_shape(shape: &Option<CellShapeRange>) -> String {
    shape
        .as_ref()
        .map(format_cell_shape_range)
        .unwrap_or_else(|| "n/a".to_owned())
}

fn format_storage(storage: &StorageShapeCandidate) -> String {
    let balance = if storage.balance_delta_min == storage.balance_delta_max {
        storage.balance_delta_min.to_string()
    } else {
        format!(
            "{}..{}",
            storage.balance_delta_min, storage.balance_delta_max
        )
    };
    let mut parts = vec![format!(
        "balance {balance}; data hash changes {}; code hash changes {}",
        storage.data_hash_changed_count, storage.code_hash_changed_count
    )];
    if let Some(shape) = &storage.post_data_shape {
        parts.push(format!("data shape {}", format_cell_shape_range(shape)));
    }
    if let Some(shape) = &storage.post_code_shape {
        parts.push(format!("code shape {}", format_cell_shape_range(shape)));
    }
    parts.join("; ")
}

fn format_cell_shape_range(shape: &CellShapeRange) -> String {
    let bits = format_range(shape.min_bits, shape.max_bits);
    let refs = format_range(shape.min_refs, shape.max_refs);
    format!("{bits}/{refs}")
}

fn format_hash_transition(before: &Option<String>, after: &Option<String>) -> String {
    format!(
        "{} -> {}",
        markdown_code_opt(before.as_deref()),
        markdown_code_opt(after.as_deref())
    )
}

fn format_kind_list(kinds: &[String]) -> String {
    if kinds.is_empty() {
        return "none".to_owned();
    }
    kinds.join(", ")
}

fn mutation_label(mutation: &ReplayMutation) -> String {
    match mutation {
        ReplayMutation::None => "none".to_owned(),
        ReplayMutation::FlipBodyBit { bit } => format!("flip body bit {bit}"),
        ReplayMutation::ReplaceBody { .. } => "replace body".to_owned(),
        ReplayMutation::SetBodyUint {
            bit_offset,
            bits,
            value,
        } => format!("set body uint {value} at {bit_offset}:{bits}"),
    }
}

fn format_optional_bool(value: Option<bool>) -> String {
    value.map_or("n/a".to_owned(), |value| value.to_string())
}

fn markdown_code_opt(value: Option<&str>) -> String {
    value.map_or_else(|| "`<none>`".to_owned(), |value| format!("`{value}`"))
}

fn markdown_code_list(values: &[String]) -> String {
    if values.is_empty() {
        return "`<none>`".to_owned();
    }
    values
        .iter()
        .map(|value| format!("`{}`", markdown_escape(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn markdown_code_list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_owned();
    }
    markdown_code_list(values)
}

fn markdown_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

impl StateFlowTx {
    pub fn from_retrace_result(
        network: impl Into<String>,
        query_hash: impl Into<String>,
        result: &TraceResult,
    ) -> anyhow::Result<Self> {
        let inbound_message = result
            .emulated_tx
            .raw
            .load_in_msg()?
            .context("retraced transaction has no inbound message")?;

        let inbound = inbound_message_artifact(&inbound_message, &result.replay.in_msg_boc64)?;
        let outbound = outbound_message_artifacts(&result.emulated_tx.raw)?;

        Ok(Self {
            schema_version: STATE_FLOW_SCHEMA_VERSION,
            network: network.into(),
            query_hash: query_hash.into(),
            transaction: TransactionIdentity {
                lt: result.emulated_tx.lt,
                utime: result.emulated_tx.utime,
                account: format_int_addr(&result.in_msg.contract),
                state_update_hash_ok: result.state_update_hash_ok,
                transaction_boc64: result.replay.transaction_boc64.clone(),
            },
            replay: ReplaySummary {
                mc_seqno: result.replay.mc_seqno,
                rand_seed_hex: result.replay.rand_seed_hex.clone(),
                replayed_prev_tx_count: result.replay.replayed_prev_tx_count,
                block_config_boc64: result.replay.block_config_boc64.clone(),
                libs_boc64: result.replay.libs_boc64.clone(),
            },
            state: StateTransition {
                pre: shard_account_snapshot(&result.replay.shard_account_before_boc64)?,
                post: shard_account_snapshot(&result.replay.shard_account_after_boc64)?,
            },
            inbound,
            outbound,
            compute: StateFlowCompute::from(&result.emulated_tx.compute_info),
            money: MoneyFlow {
                balance_before: result.money.balance_before,
                sent_total: result.money.sent_total,
                total_fees: result.money.total_fees,
                balance_after: result.money.balance_after,
            },
            c5: result
                .emulated_tx
                .c5
                .as_ref()
                .map(cell_artifact)
                .transpose()?,
            out_actions: result
                .emulated_tx
                .actions
                .iter()
                .enumerate()
                .map(|(index, action)| action_effect(index, action))
                .collect::<anyhow::Result<Vec<_>>>()?,
            vm_trace: LogArtifact::from(result.emulated_tx.vm_logs.as_ref()),
            executor_trace: LogArtifact::from(result.emulated_tx.executor_logs.as_ref()),
        })
    }
}

impl ReplayObservation {
    fn from_flow(flow: &StateFlowTx) -> Self {
        Self {
            accepted: true,
            state: Some(flow.state.post.clone()),
            inbound: flow.inbound.clone(),
            outbound: flow.outbound.clone(),
            compute: Some(flow.compute.clone()),
            money: Some(flow.money.clone()),
            c5: flow.c5.clone(),
            out_actions: flow.out_actions.clone(),
            vm_trace: Some(flow.vm_trace.clone()),
            executor_trace: Some(flow.executor_trace.clone()),
            error: None,
        }
    }

    fn from_success(
        message_boc64: &str,
        inbound: MessageArtifact,
        success: &ReplayTransactionSuccess,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            accepted: true,
            state: Some(shard_account_snapshot(
                &success.artifacts.shard_account_after_boc64,
            )?),
            inbound: MessageArtifact {
                message_boc64: message_boc64.to_owned(),
                ..inbound
            },
            outbound: outbound_message_artifacts(&success.emulated_tx.raw)?,
            compute: Some(StateFlowCompute::from(&success.emulated_tx.compute_info)),
            money: Some(MoneyFlow {
                balance_before: success.money.balance_before,
                sent_total: success.money.sent_total,
                total_fees: success.money.total_fees,
                balance_after: success.money.balance_after,
            }),
            c5: success
                .emulated_tx
                .c5
                .as_ref()
                .map(cell_artifact)
                .transpose()?,
            out_actions: success
                .emulated_tx
                .actions
                .iter()
                .enumerate()
                .map(|(index, action)| action_effect(index, action))
                .collect::<anyhow::Result<Vec<_>>>()?,
            vm_trace: Some(LogArtifact::from(success.emulated_tx.vm_logs.as_ref())),
            executor_trace: Some(LogArtifact::from(
                success.emulated_tx.executor_logs.as_ref(),
            )),
            error: None,
        })
    }
}

impl ReplayDiffSummary {
    fn compare(baseline: &ReplayObservation, replay: &ReplayObservation) -> Self {
        let baseline_state = baseline.state.as_ref();
        let replay_state = replay.state.as_ref();
        let baseline_exit = baseline
            .compute
            .as_ref()
            .and_then(|compute| compute.exit_code);
        let replay_exit = replay
            .compute
            .as_ref()
            .and_then(|compute| compute.exit_code);
        let baseline_balance = baseline
            .money
            .as_ref()
            .map(|money| money.balance_after as i128 - money.balance_before as i128);
        let replay_balance = replay
            .money
            .as_ref()
            .map(|money| money.balance_after as i128 - money.balance_before as i128);

        Self {
            replay_accepted: replay.accepted,
            input_changed: baseline.inbound.body.hash != replay.inbound.body.hash
                || baseline.inbound.message_boc64 != replay.inbound.message_boc64,
            state_changed: baseline_state
                .zip(replay_state)
                .map(|(lhs, rhs)| lhs.shard_account_boc64 != rhs.shard_account_boc64),
            code_hash_changed: baseline_state
                .zip(replay_state)
                .map(|(lhs, rhs)| lhs.code_hash != rhs.code_hash),
            data_hash_changed: baseline_state
                .zip(replay_state)
                .map(|(lhs, rhs)| lhs.data_hash != rhs.data_hash),
            balance_delta_diff: baseline_balance
                .zip(replay_balance)
                .map(|(baseline, replay)| replay - baseline),
            exit_code_changed: baseline
                .compute
                .as_ref()
                .zip(replay.compute.as_ref())
                .map(|_| baseline_exit != replay_exit),
            outbound_count_delta: replay
                .accepted
                .then_some(replay.outbound.len() as i64 - baseline.outbound.len() as i64),
            action_count_delta: replay
                .accepted
                .then_some(replay.out_actions.len() as i64 - baseline.out_actions.len() as i64),
            c5_changed: replay.accepted.then_some(
                baseline.c5.as_ref().map(|c5| &c5.hash) != replay.c5.as_ref().map(|c5| &c5.hash),
            ),
        }
    }
}

fn opcode_candidate(
    opcode: Option<String>,
    transactions: &[&StateFlowTx],
) -> OpcodeSchemaCandidate {
    let inbound_body = inbound_body_shape(transactions);
    let storage = storage_shape(transactions);
    let state_transitions = summarize_pairs(
        transactions
            .iter()
            .map(|tx| (tx.state.pre.status.clone(), tx.state.post.status.clone())),
    );
    let outbound_effects = outbound_effect_candidates(transactions);
    let out_actions = action_effect_candidates(transactions);
    let stable_body = inbound_body.min_bits == inbound_body.max_bits
        && inbound_body.min_refs == inbound_body.max_refs;
    let confidence = match (transactions.len(), stable_body) {
        (3.., true) => "high",
        (_, true) => "medium",
        _ => "low",
    }
    .to_owned();
    let mut unknown_fields = vec![
        "message body field names require TL-B recovery".to_owned(),
        "storage field names require typed storage decoding".to_owned(),
    ];
    if !stable_body {
        unknown_fields.push("variable message body shape observed".to_owned());
    }
    if !outbound_effects.is_empty() || !out_actions.is_empty() {
        unknown_fields.push("outbound effect payload fields require TL-B recovery".to_owned());
    }
    let examples = transactions
        .iter()
        .take(5)
        .map(|tx| tx.query_hash.clone())
        .collect::<Vec<_>>();
    let replay_probes = replay_probe_candidates(&inbound_body, &examples);

    OpcodeSchemaCandidate {
        opcode,
        count: transactions.len(),
        examples,
        evidence: schema_evidence(transactions),
        inbound_body,
        replay_probes,
        storage,
        state_transitions,
        outbound_effects,
        out_actions,
        confidence,
        unknown_fields,
    }
}

fn schema_evidence(transactions: &[&StateFlowTx]) -> Vec<SchemaEvidence> {
    transactions
        .iter()
        .map(|tx| SchemaEvidence {
            tx_hash: tx.query_hash.clone(),
            inbound_body_hash: tx.inbound.body.hash.clone(),
            inbound_body_bits: tx.inbound.body.bits,
            inbound_body_refs: tx.inbound.body.refs,
            from_status: tx.state.pre.status.clone(),
            to_status: tx.state.post.status.clone(),
            pre_data_hash: tx.state.pre.data_hash.clone(),
            post_data_hash: tx.state.post.data_hash.clone(),
            pre_code_hash: tx.state.pre.code_hash.clone(),
            post_code_hash: tx.state.post.code_hash.clone(),
            outbound_kinds: tx
                .outbound
                .iter()
                .map(|message| message.kind.clone())
                .collect(),
            out_action_kinds: tx
                .out_actions
                .iter()
                .map(|action| action.kind.clone())
                .collect(),
        })
        .collect()
}

fn storage_shape(transactions: &[&StateFlowTx]) -> StorageShapeCandidate {
    let mut balance_delta_min = i128::MAX;
    let mut balance_delta_max = i128::MIN;
    let mut data_hash_changed_count = 0;
    let mut code_hash_changed_count = 0;
    let mut post_data_hashes = BTreeSet::new();
    let mut post_code_hashes = BTreeSet::new();
    let mut post_data_shapes = Vec::new();
    let mut post_code_shapes = Vec::new();
    let fields = storage_field_candidates(transactions);

    for tx in transactions {
        let balance_delta = tx.money.balance_after as i128 - tx.money.balance_before as i128;
        balance_delta_min = balance_delta_min.min(balance_delta);
        balance_delta_max = balance_delta_max.max(balance_delta);
        if tx.state.pre.data_hash != tx.state.post.data_hash {
            data_hash_changed_count += 1;
        }
        if tx.state.pre.code_hash != tx.state.post.code_hash {
            code_hash_changed_count += 1;
        }
        if let Some(hash) = &tx.state.post.data_hash {
            post_data_hashes.insert(hash.clone());
        }
        if let Some(hash) = &tx.state.post.code_hash {
            post_code_hashes.insert(hash.clone());
        }
        if let Some(shape) = &tx.state.post.data_cell {
            post_data_shapes.push(shape.clone());
        }
        if let Some(shape) = &tx.state.post.code_cell {
            post_code_shapes.push(shape.clone());
        }
    }

    StorageShapeCandidate {
        balance_delta_min: if balance_delta_min == i128::MAX {
            0
        } else {
            balance_delta_min
        },
        balance_delta_max: if balance_delta_max == i128::MIN {
            0
        } else {
            balance_delta_max
        },
        data_hash_changed_count,
        code_hash_changed_count,
        post_data_shape: cell_shape_range(&post_data_shapes),
        post_code_shape: cell_shape_range(&post_code_shapes),
        fields,
        post_data_hashes: post_data_hashes.into_iter().collect(),
        post_code_hashes: post_code_hashes.into_iter().collect(),
    }
}

fn storage_field_candidates(transactions: &[&StateFlowTx]) -> Vec<StorageFieldCandidate> {
    if transactions.is_empty() {
        return Vec::new();
    }

    let mut data_word_samples = BTreeSet::new();
    let mut data_word_present_count = 0;
    let mut tail_min_bits = u16::MAX;
    let mut tail_max_bits = 0;
    let mut tail_min_refs = u8::MAX;
    let mut tail_max_refs = 0;
    let mut tail_present_count = 0;
    let mut tail_samples = BTreeSet::new();

    for tx in transactions {
        let Some(data_cell) = post_data_cell(&tx.state.post) else {
            continue;
        };
        let slice = data_cell.as_slice_allow_exotic();
        let bits = slice.size_bits();
        let refs = slice.size_refs();

        if bits >= 32 {
            if let Some(word) = read_cell_u32_at(&data_cell, 0) {
                data_word_present_count += 1;
                data_word_samples.insert(format_u32_hex(word));
            }
        }

        if bits > 32 || refs > 0 {
            let tail_bits = bits.saturating_sub(32);
            tail_min_bits = tail_min_bits.min(tail_bits);
            tail_max_bits = tail_max_bits.max(tail_bits);
            tail_min_refs = tail_min_refs.min(refs);
            tail_max_refs = tail_max_refs.max(refs);
            tail_present_count += 1;
            tail_samples.insert(format!("{tail_bits} bits, {refs} refs"));
        }
    }

    let mut candidates = Vec::new();
    if data_word_present_count > 0 {
        candidates.push(StorageFieldCandidate {
            name: "data_word_0".to_owned(),
            cell_path: "data".to_owned(),
            bit_offset: 0,
            min_bits: 32,
            max_bits: 32,
            min_refs: 0,
            max_refs: 0,
            kind: "uint32".to_owned(),
            present_count: data_word_present_count,
            value_samples: limited_samples(data_word_samples),
            confidence: field_confidence(data_word_present_count, transactions.len()),
        });
    }

    if tail_present_count > 0 {
        candidates.push(StorageFieldCandidate {
            name: "data_tail".to_owned(),
            cell_path: "data".to_owned(),
            bit_offset: 32,
            min_bits: tail_min_bits,
            max_bits: tail_max_bits,
            min_refs: if tail_min_refs == u8::MAX {
                0
            } else {
                tail_min_refs
            },
            max_refs: tail_max_refs,
            kind: "raw".to_owned(),
            present_count: tail_present_count,
            value_samples: limited_samples(tail_samples),
            confidence: "low".to_owned(),
        });
    }

    candidates
}

fn post_data_cell(snapshot: &ShardAccountSnapshot) -> Option<Cell> {
    if let Some(boc64) = snapshot
        .data_cell
        .as_ref()
        .and_then(|shape| shape.boc64.as_ref())
    {
        return Boc::decode_base64(boc64).ok();
    }

    let shard_account = Boc::decode_base64(&snapshot.shard_account_boc64)
        .ok()?
        .parse::<ShardAccount>()
        .ok()?;
    let account = shard_account.load_account().ok()??;
    match account.state {
        AccountState::Active(state) => state.data,
        _ => None,
    }
}

fn cell_shape_range(shapes: &[CellShape]) -> Option<CellShapeRange> {
    let first = shapes.first()?;
    let mut min_bits = first.bits;
    let mut max_bits = first.bits;
    let mut min_refs = first.refs;
    let mut max_refs = first.refs;

    for shape in &shapes[1..] {
        min_bits = min_bits.min(shape.bits);
        max_bits = max_bits.max(shape.bits);
        min_refs = min_refs.min(shape.refs);
        max_refs = max_refs.max(shape.refs);
    }

    Some(CellShapeRange {
        min_bits,
        max_bits,
        min_refs,
        max_refs,
    })
}

fn inbound_body_shape(transactions: &[&StateFlowTx]) -> BodyShapeCandidate {
    let mut min_bits = u16::MAX;
    let mut max_bits = 0;
    let mut min_refs = u8::MAX;
    let mut max_refs = 0;
    let mut body_hashes = BTreeSet::new();
    let field_candidates = inbound_body_field_candidates(transactions);
    for tx in transactions {
        min_bits = min_bits.min(tx.inbound.body.bits);
        max_bits = max_bits.max(tx.inbound.body.bits);
        min_refs = min_refs.min(tx.inbound.body.refs);
        max_refs = max_refs.max(tx.inbound.body.refs);
        body_hashes.insert(tx.inbound.body.hash.clone());
    }

    BodyShapeCandidate {
        min_bits: if min_bits == u16::MAX { 0 } else { min_bits },
        max_bits,
        min_refs: if min_refs == u8::MAX { 0 } else { min_refs },
        max_refs,
        body_hashes: body_hashes.into_iter().collect(),
        field_candidates,
    }
}

fn inbound_body_field_candidates(transactions: &[&StateFlowTx]) -> Vec<BodyFieldCandidate> {
    if transactions.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let mut opcode_samples = BTreeSet::new();
    let mut opcode_present_count = 0;
    let mut query_id_samples = BTreeSet::new();
    let mut query_id_present_count = 0;
    let mut tail_min_bits = u16::MAX;
    let mut tail_max_bits = 0;
    let mut tail_min_refs = u8::MAX;
    let mut tail_max_refs = 0;
    let mut tail_present_count = 0;
    let mut tail_samples = BTreeSet::new();

    for tx in transactions {
        if tx.inbound.body.bits >= 32 {
            let opcode = tx
                .inbound
                .opcode
                .clone()
                .or_else(|| read_body_u32_at(&tx.inbound.body.boc64, 0).map(format_u32_hex));
            if let Some(opcode) = opcode {
                opcode_present_count += 1;
                opcode_samples.insert(opcode);
            }
        }

        if tx.inbound.body.bits >= 96 {
            if let Some(query_id) = read_body_u64_at(&tx.inbound.body.boc64, 32) {
                query_id_present_count += 1;
                query_id_samples.insert(format_u64_hex(query_id));
            }

            if tx.inbound.body.bits > 96 || tx.inbound.body.refs > 0 {
                let tail_bits = tx.inbound.body.bits.saturating_sub(96);
                tail_min_bits = tail_min_bits.min(tail_bits);
                tail_max_bits = tail_max_bits.max(tail_bits);
                tail_min_refs = tail_min_refs.min(tx.inbound.body.refs);
                tail_max_refs = tail_max_refs.max(tx.inbound.body.refs);
                tail_present_count += 1;
                tail_samples.insert(format!("{tail_bits} bits, {} refs", tx.inbound.body.refs));
            }
        }
    }

    if opcode_present_count > 0 {
        candidates.push(BodyFieldCandidate {
            name: "opcode".to_owned(),
            bit_offset: 0,
            min_bits: 32,
            max_bits: 32,
            min_refs: 0,
            max_refs: 0,
            kind: "uint32".to_owned(),
            present_count: opcode_present_count,
            value_samples: limited_samples(opcode_samples),
            confidence: body_field_confidence(opcode_present_count, transactions.len()),
        });
    }

    if query_id_present_count > 0 {
        candidates.push(BodyFieldCandidate {
            name: "query_id".to_owned(),
            bit_offset: 32,
            min_bits: 64,
            max_bits: 64,
            min_refs: 0,
            max_refs: 0,
            kind: "uint64".to_owned(),
            present_count: query_id_present_count,
            value_samples: limited_samples(query_id_samples),
            confidence: body_field_confidence(query_id_present_count, transactions.len()),
        });
    }

    if tail_present_count > 0 {
        candidates.push(BodyFieldCandidate {
            name: "payload_tail".to_owned(),
            bit_offset: 96,
            min_bits: tail_min_bits,
            max_bits: tail_max_bits,
            min_refs: if tail_min_refs == u8::MAX {
                0
            } else {
                tail_min_refs
            },
            max_refs: tail_max_refs,
            kind: "raw".to_owned(),
            present_count: tail_present_count,
            value_samples: limited_samples(tail_samples),
            confidence: "low".to_owned(),
        });
    }

    candidates
}

fn replay_probe_candidates(
    inbound_body: &BodyShapeCandidate,
    examples: &[String],
) -> Vec<ReplayProbeCandidate> {
    inbound_body
        .field_candidates
        .iter()
        .filter_map(|field| replay_probe_candidate(field, examples))
        .collect()
}

fn replay_probe_candidate(
    field: &BodyFieldCandidate,
    examples: &[String],
) -> Option<ReplayProbeCandidate> {
    let bits = exact_uint_body_field_bits(field)?;
    let value = replay_probe_value(field, bits)?;
    let cli_arg = format!("--set-body-uint {}:{}:{}", field.bit_offset, bits, value);

    Some(ReplayProbeCandidate {
        field_name: field.name.clone(),
        bit_offset: field.bit_offset,
        bits,
        value: value.clone(),
        mutation: ReplayMutation::SetBodyUint {
            bit_offset: field.bit_offset,
            bits,
            value,
        },
        cli_arg,
        confidence: field.confidence.clone(),
        evidence: examples.iter().take(5).cloned().collect(),
    })
}

fn exact_uint_body_field_bits(field: &BodyFieldCandidate) -> Option<u16> {
    let bits = field.min_bits;
    (bits == field.max_bits
        && bits > 0
        && bits <= 64
        && field.min_refs == 0
        && field.max_refs == 0
        && field.kind.starts_with("uint"))
    .then_some(bits)
}

fn replay_probe_value(field: &BodyFieldCandidate, bits: u16) -> Option<String> {
    let sample = field
        .value_samples
        .first()
        .and_then(|value| parse_uint_value(value).ok())
        .unwrap_or(0);
    let mask = if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    let value = (sample ^ 1) & mask;
    Some(format_uint_for_bits(value, bits))
}

fn read_body_u32_at(boc64: &str, bit_offset: u16) -> Option<u32> {
    let cell = Boc::decode_base64(boc64).ok()?;
    read_cell_u32_at(&cell, bit_offset)
}

fn read_cell_u32_at(cell: &Cell, bit_offset: u16) -> Option<u32> {
    let mut slice = cell.as_slice_allow_exotic();
    slice.skip_first(bit_offset, 0).ok()?;
    slice.load_u32().ok()
}

fn read_body_u64_at(boc64: &str, bit_offset: u16) -> Option<u64> {
    let cell = Boc::decode_base64(boc64).ok()?;
    let mut slice = cell.as_slice_allow_exotic();
    slice.skip_first(bit_offset, 0).ok()?;
    slice.load_u64().ok()
}

fn format_u32_hex(value: u32) -> String {
    format!("0x{value:08x}")
}

fn format_u64_hex(value: u64) -> String {
    format!("0x{value:016x}")
}

fn format_uint_for_bits(value: u64, bits: u16) -> String {
    let digits = usize::from(bits).div_ceil(4);
    format!("0x{value:0digits$x}")
}

fn limited_samples(values: BTreeSet<String>) -> Vec<String> {
    values.into_iter().take(5).collect()
}

fn body_field_confidence(present_count: usize, transaction_count: usize) -> String {
    field_confidence(present_count, transaction_count)
}

fn field_confidence(present_count: usize, transaction_count: usize) -> String {
    if present_count == transaction_count {
        "high".to_owned()
    } else {
        "medium".to_owned()
    }
}

fn summarize_pairs(pairs: impl Iterator<Item = (String, String)>) -> Vec<StateTransitionCandidate> {
    let mut counts = BTreeMap::<(String, String), usize>::new();
    for pair in pairs {
        *counts.entry(pair).or_default() += 1;
    }
    counts
        .into_iter()
        .map(
            |((from_status, to_status), count)| StateTransitionCandidate {
                from_status,
                to_status,
                count,
            },
        )
        .collect()
}

fn outbound_effect_candidates(transactions: &[&StateFlowTx]) -> Vec<EffectCandidate> {
    let mut summaries = BTreeMap::<String, EffectSummary>::new();
    for tx in transactions {
        for message in &tx.outbound {
            let summary = summaries.entry(message.kind.clone()).or_default();
            summary.observe_tx(&tx.query_hash);
            summary.observe_value(message.value_nanotons.as_deref());
            summary.observe_destination(message.dst.as_deref());
            summary.observe_body(&message.body);
        }
    }
    summaries
        .into_iter()
        .map(|(kind, summary)| summary.into_candidate(kind))
        .collect()
}

fn action_effect_candidates(transactions: &[&StateFlowTx]) -> Vec<EffectCandidate> {
    let mut summaries = BTreeMap::<String, EffectSummary>::new();
    for tx in transactions {
        for action in &tx.out_actions {
            let summary = summaries.entry(action.kind.clone()).or_default();
            summary.observe_tx(&tx.query_hash);
            summary.observe_mode(action.mode.as_deref());
            summary.observe_value(action.value_nanotons.as_deref());
            summary.observe_destination(action.destination.as_deref());
            if let Some(body) = &action.body {
                summary.observe_body(body);
            }
            if let Some(code) = &action.code {
                summary.observe_code(code);
            }
            if let Some(library) = &action.library {
                summary.observe_library(library);
            }
        }
    }
    summaries
        .into_iter()
        .map(|(kind, summary)| summary.into_candidate(kind))
        .collect()
}

#[derive(Default)]
struct EffectSummary {
    count: usize,
    tx_hashes: BTreeSet<String>,
    modes: BTreeSet<String>,
    destinations: BTreeSet<String>,
    values: Vec<u128>,
    body_shapes: Vec<CellShape>,
    code_shapes: Vec<CellShape>,
    library_hashes: BTreeSet<String>,
}

impl EffectSummary {
    fn observe_tx(&mut self, tx_hash: &str) {
        self.count += 1;
        self.tx_hashes.insert(tx_hash.to_owned());
    }

    fn observe_mode(&mut self, mode: Option<&str>) {
        if let Some(mode) = mode {
            self.modes.insert(mode.to_owned());
        }
    }

    fn observe_value(&mut self, value: Option<&str>) {
        if let Some(value) = value.and_then(|value| value.parse::<u128>().ok()) {
            self.values.push(value);
        }
    }

    fn observe_destination(&mut self, destination: Option<&str>) {
        if let Some(destination) = destination {
            self.destinations.insert(destination.to_owned());
        }
    }

    fn observe_body(&mut self, body: &CellArtifact) {
        self.body_shapes.push(cell_artifact_shape(body));
    }

    fn observe_code(&mut self, code: &CellArtifact) {
        self.code_shapes.push(cell_artifact_shape(code));
    }

    fn observe_library(&mut self, library: &LibraryEffect) {
        if let Some(hash) = &library.hash {
            self.library_hashes.insert(hash.clone());
        }
        if let Some(cell) = &library.cell {
            self.library_hashes.insert(cell.hash.clone());
        }
    }

    fn into_candidate(self, kind: String) -> EffectCandidate {
        let value_nanotons_min = self.values.iter().min().map(ToString::to_string);
        let value_nanotons_max = self.values.iter().max().map(ToString::to_string);
        EffectCandidate {
            kind,
            count: self.count,
            tx_hashes: limited_samples(self.tx_hashes),
            modes: limited_samples(self.modes),
            destinations: limited_samples(self.destinations),
            value_nanotons_min,
            value_nanotons_max,
            body_shape: cell_shape_range(&self.body_shapes),
            code_shape: cell_shape_range(&self.code_shapes),
            library_hashes: limited_samples(self.library_hashes),
        }
    }
}

fn cell_artifact_shape(cell: &CellArtifact) -> CellShape {
    CellShape {
        boc64: None,
        hash: cell.hash.clone(),
        bits: cell.bits,
        refs: cell.refs,
    }
}

fn apply_replay_mutation(message_boc64: &str, mutation: &ReplayMutation) -> anyhow::Result<String> {
    match mutation {
        ReplayMutation::None => Ok(message_boc64.to_owned()),
        ReplayMutation::FlipBodyBit { bit } => {
            let message_cell = Boc::decode_base64(message_boc64)?;
            let message = message_cell.parse::<tycho_types::models::Message<'_>>()?;
            let body = cell_from_slice(&message.body)?;
            let mutated_body = flip_cell_bit(&body, *bit)?;
            message_with_body_boc64(&message, mutated_body)
        }
        ReplayMutation::ReplaceBody { body_boc64 } => {
            let message_cell = Boc::decode_base64(message_boc64)?;
            let message = message_cell.parse::<tycho_types::models::Message<'_>>()?;
            let body = Boc::decode_base64(body_boc64)?;
            message_with_body_boc64(&message, body)
        }
        ReplayMutation::SetBodyUint {
            bit_offset,
            bits,
            value,
        } => {
            let message_cell = Boc::decode_base64(message_boc64)?;
            let message = message_cell.parse::<tycho_types::models::Message<'_>>()?;
            let body = cell_from_slice(&message.body)?;
            let mutated_body = set_cell_uint(&body, *bit_offset, *bits, value)?;
            message_with_body_boc64(&message, mutated_body)
        }
    }
}

fn message_with_body_boc64(
    message: &tycho_types::models::Message<'_>,
    body: Cell,
) -> anyhow::Result<String> {
    let owned = OwnedMessage {
        info: message.info.clone(),
        init: message.init.clone(),
        body: body.into(),
        layout: message.layout,
    };
    Ok(Boc::encode_base64(to_cell(&owned)?))
}

fn flip_cell_bit(cell: &Cell, bit: u16) -> anyhow::Result<Cell> {
    let slice = cell.as_slice_allow_exotic();
    if bit >= slice.size_bits() {
        anyhow::bail!(
            "cannot flip body bit {bit}; body has {} bits",
            slice.size_bits()
        );
    }

    let mut builder = CellBuilder::new();
    for index in 0..slice.size_bits() {
        let value = slice.get_bit(index)?;
        builder.store_bit(if index == bit { !value } else { value })?;
    }
    for index in 0..slice.size_refs() {
        builder.store_reference(slice.get_reference_cloned(index)?)?;
    }
    Ok(builder.build()?)
}

fn set_cell_uint(cell: &Cell, bit_offset: u16, bits: u16, value: &str) -> anyhow::Result<Cell> {
    if bits == 0 || bits > 64 {
        anyhow::bail!("setBodyUint supports 1..=64 bits, got {bits}");
    }
    let value = parse_uint_value(value)?;
    if bits < 64 && value >= (1u64 << bits) {
        anyhow::bail!("value {value} does not fit in {bits} bit(s)");
    }

    let slice = cell.as_slice_allow_exotic();
    let end = bit_offset
        .checked_add(bits)
        .context("setBodyUint bit range overflows u16")?;
    if end > slice.size_bits() {
        anyhow::bail!(
            "cannot set body uint {bit_offset}:{bits}; body has {} bits",
            slice.size_bits()
        );
    }

    let mut builder = CellBuilder::new();
    for index in 0..slice.size_bits() {
        let bit = if (bit_offset..end).contains(&index) {
            let shift = u32::from(end - index - 1);
            ((value >> shift) & 1) == 1
        } else {
            slice.get_bit(index)?
        };
        builder.store_bit(bit)?;
    }
    for index in 0..slice.size_refs() {
        builder.store_reference(slice.get_reference_cloned(index)?)?;
    }
    Ok(builder.build()?)
}

fn parse_uint_value(value: &str) -> anyhow::Result<u64> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).with_context(|| format!("invalid hex uint value {value:?}"))
    } else {
        value
            .parse::<u64>()
            .with_context(|| format!("invalid uint value {value:?}"))
    }
}

fn inbound_artifact_from_boc64(message_boc64: &str) -> anyhow::Result<MessageArtifact> {
    let message_cell = Boc::decode_base64(message_boc64)?;
    let message = message_cell.parse::<tycho_types::models::Message<'_>>()?;
    inbound_message_artifact(&message, message_boc64)
}

impl StateFlowFailure {
    fn from_error(tx_ref: &AccountTxRef, err: anyhow::Error) -> Self {
        Self {
            hash: tx_ref.hash.clone(),
            lt: tx_ref.lt,
            error: err.to_string(),
        }
    }
}

impl From<&ComputeInfo> for StateFlowCompute {
    fn from(value: &ComputeInfo) -> Self {
        match value {
            ComputeInfo::Skipped => Self {
                skipped: true,
                success: None,
                exit_code: None,
                vm_steps: None,
                gas_used: None,
                gas_fees: None,
            },
            ComputeInfo::Success {
                success,
                exit_code,
                vm_steps,
                gas_used,
                gas_fees,
            } => Self {
                skipped: false,
                success: Some(*success),
                exit_code: Some(*exit_code),
                vm_steps: Some(*vm_steps),
                gas_used: Some(*gas_used),
                gas_fees: Some(*gas_fees),
            },
        }
    }
}

fn opcode_summary(transactions: &[StateFlowTx]) -> Vec<OpcodeSummary> {
    let mut by_opcode = BTreeMap::<Option<String>, Vec<String>>::new();
    for tx in transactions {
        by_opcode
            .entry(tx.inbound.opcode.clone())
            .or_default()
            .push(tx.query_hash.clone());
    }

    by_opcode
        .into_iter()
        .map(|(opcode, tx_hashes)| OpcodeSummary {
            opcode,
            count: tx_hashes.len(),
            tx_hashes,
        })
        .collect()
}

impl From<&str> for LogArtifact {
    fn from(text: &str) -> Self {
        Self {
            line_count: text.lines().count(),
            text: text.to_owned(),
        }
    }
}

fn inbound_message_artifact(
    msg: &tycho_types::models::Message<'_>,
    message_boc64: &str,
) -> anyhow::Result<MessageArtifact> {
    let body = cell_artifact_from_slice(&msg.body)?;
    let opcode = opcode_from_slice(msg.body.clone(), inbound_bounced(&msg.info));
    let (kind, src, dst, value_nanotons, bounced, bounce) = match &msg.info {
        MsgInfo::Int(info) => (
            "internal".to_owned(),
            Some(format_int_addr(&info.src)),
            Some(format_int_addr(&info.dst)),
            Some(info.value.tokens.to_string()),
            Some(info.bounced),
            Some(info.bounce),
        ),
        MsgInfo::ExtIn(info) => (
            "external-in".to_owned(),
            info.src.as_ref().map(ToString::to_string),
            Some(format_int_addr(&info.dst)),
            None,
            None,
            None,
        ),
        MsgInfo::ExtOut(info) => (
            "external-out".to_owned(),
            Some(format_int_addr(&info.src)),
            info.dst.as_ref().map(ToString::to_string),
            None,
            None,
            None,
        ),
    };

    Ok(MessageArtifact {
        direction: MessageDirection::Inbound,
        index: None,
        kind,
        src,
        dst,
        value_nanotons,
        bounced,
        bounce,
        opcode,
        message_boc64: message_boc64.to_owned(),
        body,
    })
}

fn outbound_message_artifacts(
    tx: &tycho_types::models::Transaction,
) -> anyhow::Result<Vec<MessageArtifact>> {
    tx.iter_out_msgs()
        .enumerate()
        .map(|(index, msg)| {
            let msg = msg?;
            let message_cell = to_cell(&msg)?;
            let message_boc64 = Boc::encode_base64(message_cell);
            let body = cell_artifact_from_slice(&msg.body)?;
            let opcode = opcode_from_slice(msg.body.clone(), inbound_bounced(&msg.info));
            let (kind, src, dst, value_nanotons, bounced, bounce) = match &msg.info {
                MsgInfo::Int(info) => (
                    "internal".to_owned(),
                    Some(format_int_addr(&info.src)),
                    Some(format_int_addr(&info.dst)),
                    Some(info.value.tokens.to_string()),
                    Some(info.bounced),
                    Some(info.bounce),
                ),
                MsgInfo::ExtIn(info) => (
                    "external-in".to_owned(),
                    info.src.as_ref().map(ToString::to_string),
                    Some(format_int_addr(&info.dst)),
                    None,
                    None,
                    None,
                ),
                MsgInfo::ExtOut(info) => (
                    "external-out".to_owned(),
                    Some(format_int_addr(&info.src)),
                    info.dst.as_ref().map(ToString::to_string),
                    None,
                    None,
                    None,
                ),
            };

            Ok(MessageArtifact {
                direction: MessageDirection::Outbound,
                index: Some(index),
                kind,
                src,
                dst,
                value_nanotons,
                bounced,
                bounce,
                opcode,
                message_boc64,
                body,
            })
        })
        .collect()
}

fn shard_account_snapshot(boc64: &str) -> anyhow::Result<ShardAccountSnapshot> {
    let shard_account = Boc::decode_base64(boc64)?.parse::<ShardAccount>()?;
    let account = shard_account.load_account()?;
    let (
        account_address,
        status,
        balance_nanotons,
        code_hash,
        data_hash,
        code_cell,
        data_cell,
        frozen_hash,
    ) = if let Some(account) = account {
        let balance = account.balance.tokens.to_string();
        match account.state {
            AccountState::Uninit => (
                Some(format_int_addr(&account.address)),
                "uninit".to_owned(),
                balance,
                None,
                None,
                None,
                None,
                None,
            ),
            AccountState::Active(state) => {
                let code_cell = state.code.as_ref().map(cell_shape);
                let data_cell = state.data.as_ref().map(cell_shape);
                (
                    Some(format_int_addr(&account.address)),
                    "active".to_owned(),
                    balance,
                    code_cell.as_ref().map(|shape| shape.hash.clone()),
                    data_cell.as_ref().map(|shape| shape.hash.clone()),
                    code_cell,
                    data_cell,
                    None,
                )
            }
            AccountState::Frozen(hash) => (
                Some(format_int_addr(&account.address)),
                "frozen".to_owned(),
                balance,
                None,
                None,
                None,
                None,
                Some(hash.to_string()),
            ),
        }
    } else {
        (
            None,
            "none".to_owned(),
            "0".to_owned(),
            None,
            None,
            None,
            None,
            None,
        )
    };

    Ok(ShardAccountSnapshot {
        shard_account_boc64: boc64.to_owned(),
        last_trans_lt: shard_account.last_trans_lt,
        last_trans_hash: shard_account.last_trans_hash.to_string(),
        account_address,
        status,
        balance_nanotons,
        code_hash,
        data_hash,
        code_cell,
        data_cell,
        frozen_hash,
    })
}

fn action_effect(index: usize, action: &OutAction) -> anyhow::Result<ActionEffect> {
    match action {
        OutAction::SendMsg { mode, out_msg } => {
            let msg = out_msg.load()?;
            let destination = match &msg.info {
                RelaxedMsgInfo::Int(info) => Some(format_int_addr(&info.dst)),
                RelaxedMsgInfo::ExtOut(info) => info.dst.as_ref().map(ToString::to_string),
            };
            let body = msg
                .body
                .0
                .apply(&msg.body.1)
                .ok()
                .map(|slice| cell_artifact_from_slice(&slice))
                .transpose()?;
            Ok(ActionEffect {
                index,
                kind: "send-message".to_owned(),
                mode: Some(format!("{mode:?}")),
                value_nanotons: match &msg.info {
                    RelaxedMsgInfo::Int(info) => Some(info.value.tokens.to_string()),
                    RelaxedMsgInfo::ExtOut(_) => None,
                },
                destination,
                body,
                code: None,
                library: None,
            })
        }
        OutAction::SetCode { new_code } => Ok(ActionEffect {
            index,
            kind: "set-code".to_owned(),
            mode: None,
            value_nanotons: None,
            destination: None,
            body: None,
            code: Some(cell_artifact(new_code)?),
            library: None,
        }),
        OutAction::ReserveCurrency { mode, value } => Ok(ActionEffect {
            index,
            kind: "reserve-currency".to_owned(),
            mode: Some(format!("{mode:?}")),
            value_nanotons: Some(value.tokens.to_string()),
            destination: None,
            body: None,
            code: None,
            library: None,
        }),
        OutAction::ChangeLibrary { mode, lib } => {
            let library = match lib {
                tycho_types::models::LibRef::Hash(hash) => LibraryEffect {
                    mode: format!("{mode:?}"),
                    hash: Some(hash.to_string()),
                    cell: None,
                },
                tycho_types::models::LibRef::Cell(cell) => LibraryEffect {
                    mode: format!("{mode:?}"),
                    hash: Some(cell_hash(cell)),
                    cell: Some(cell_artifact(cell)?),
                },
            };
            Ok(ActionEffect {
                index,
                kind: "change-library".to_owned(),
                mode: Some(format!("{mode:?}")),
                value_nanotons: None,
                destination: None,
                body: None,
                code: None,
                library: Some(library),
            })
        }
    }
}

fn inbound_bounced(info: &MsgInfo) -> bool {
    matches!(info, MsgInfo::Int(info) if info.bounced)
}

fn opcode_from_slice(mut slice: CellSlice<'_>, bounced: bool) -> Option<String> {
    if bounced {
        slice.skip_first(32, 0).ok()?;
    }
    Some(format!("0x{:08x}", slice.load_u32().ok()?))
}

fn cell_artifact_from_slice(slice: &CellSlice<'_>) -> anyhow::Result<CellArtifact> {
    let cell = cell_from_slice(slice)?;
    cell_artifact(&cell)
}

fn cell_from_slice(slice: &CellSlice<'_>) -> anyhow::Result<Cell> {
    let mut builder = CellBuilder::new();
    slice.store_into(&mut builder, Cell::empty_context())?;
    Ok(builder.build()?)
}

fn cell_artifact(cell: &Cell) -> anyhow::Result<CellArtifact> {
    let shape = cell_shape(cell);
    Ok(CellArtifact {
        boc64: Boc::encode_base64(cell),
        hash: shape.hash,
        bits: shape.bits,
        refs: shape.refs,
    })
}

fn cell_shape(cell: &Cell) -> CellShape {
    let slice = cell.as_slice_allow_exotic();
    CellShape {
        boc64: Some(Boc::encode_base64(cell)),
        hash: cell_hash(cell),
        bits: slice.size_bits(),
        refs: slice.size_refs(),
    }
}

fn to_cell<T: Store + ?Sized>(obj: &T) -> anyhow::Result<Cell> {
    let mut builder = CellBuilder::new();
    obj.store_into(&mut builder, Cell::empty_context())?;
    Ok(builder.build()?)
}

fn cell_hash(cell: &Cell) -> String {
    hex::encode(cell.hash(0))
}

fn format_int_addr(addr: &IntAddr) -> String {
    match addr {
        IntAddr::Std(addr) => addr.display_base64_url(false).to_string(),
        _ => addr.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CellArtifact, LogArtifact, MessageDirection, MoneyFlow, ReplayDiffSummary, ReplayMutation,
        ReplayObservation, ReplaySummary, StateFlowCompute, StateFlowCorpus, StateFlowReplayDiff,
        StateFlowTx, StateTransition, TransactionIdentity,
    };
    use tycho_types::boc::Boc;
    use tycho_types::cell::CellBuilder;
    use tycho_types::models::{IntMsgInfo, MsgInfo, OwnedMessage};

    #[test]
    fn state_flow_tx_serializes_camel_case_schema_version() {
        let flow = sample_flow("abc", Some("0x00000001"));

        let json = serde_json::to_value(&flow).expect("state flow should serialize");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["transaction"]["stateUpdateHashOk"], true);
        assert_eq!(json["inbound"]["direction"], "inbound");
        assert_eq!(json["replay"]["libsBoc64"], "libs");
        assert_eq!(json["state"]["post"]["dataCell"]["hash"], "data");
        assert_eq!(json["state"]["post"]["dataCell"]["bits"], 16);
        assert_eq!(json["state"]["post"]["dataCell"]["refs"], 1);
        assert_eq!(json["state"]["post"]["codeCell"]["hash"], "code");
        assert_eq!(json["state"]["post"]["codeCell"]["bits"], 8);
        assert_eq!(json["state"]["post"]["codeCell"]["refs"], 0);
    }

    #[test]
    fn opcode_summary_groups_transactions_by_inbound_opcode() {
        let flows = vec![
            sample_flow("tx-a", Some("0x00000001")),
            sample_flow("tx-b", Some("0x00000001")),
            sample_flow("tx-c", None),
        ];

        let summary = super::opcode_summary(&flows);

        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].opcode, None);
        assert_eq!(summary[0].count, 1);
        assert_eq!(summary[0].tx_hashes, vec!["tx-c".to_owned()]);
        assert_eq!(summary[1].opcode.as_deref(), Some("0x00000001"));
        assert_eq!(summary[1].count, 2);
        assert_eq!(
            summary[1].tx_hashes,
            vec!["tx-a".to_owned(), "tx-b".to_owned()]
        );
    }

    #[test]
    fn infer_schema_candidates_reports_body_shape_and_transitions() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 2,
            source_tx_count: 2,
            retraced_count: 2,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![
                sample_flow("tx-a", Some("0x00000001")),
                sample_flow("tx-b", Some("0x00000001")),
            ],
            failures: Vec::new(),
        };

        let report = super::infer_schema_candidates(&corpus);

        assert_eq!(report.transaction_count, 2);
        assert_eq!(report.opcode_candidates.len(), 1);
        let candidate = &report.opcode_candidates[0];
        assert_eq!(candidate.opcode.as_deref(), Some("0x00000001"));
        assert_eq!(candidate.count, 2);
        assert_eq!(candidate.inbound_body.min_bits, 32);
        assert_eq!(candidate.inbound_body.max_bits, 32);
        assert_eq!(candidate.storage.balance_delta_min, -3);
        assert_eq!(candidate.storage.balance_delta_max, -3);
        assert_eq!(candidate.storage.data_hash_changed_count, 2);
        assert_eq!(candidate.storage.code_hash_changed_count, 2);
        assert_eq!(candidate.state_transitions[0].from_status, "none");
        assert_eq!(candidate.state_transitions[0].to_status, "active");
        assert_eq!(candidate.confidence, "medium");
        assert!(
            candidate
                .unknown_fields
                .iter()
                .any(|field| field.contains("TL-B"))
        );

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["stateMachine"]["edges"][0]["fromStatus"], "none");
        assert_eq!(json["stateMachine"]["edges"][0]["toStatus"], "active");
        assert_eq!(json["stateMachine"]["edges"][0]["opcode"], "0x00000001");
        assert_eq!(json["stateMachine"]["edges"][0]["count"], 2);
        assert_eq!(
            json["stateMachine"]["edges"][0]["examples"],
            serde_json::json!(["tx-a", "tx-b"])
        );
        assert_eq!(
            json["opcodeCandidates"][0]["storage"]["postDataShape"],
            serde_json::json!({
                "minBits": 16,
                "maxBits": 16,
                "minRefs": 1,
                "maxRefs": 1
            })
        );
        assert_eq!(
            json["opcodeCandidates"][0]["storage"]["postCodeShape"],
            serde_json::json!({
                "minBits": 8,
                "maxBits": 8,
                "minRefs": 0,
                "maxRefs": 0
            })
        );
        let audit_signals = json["auditSignals"]
            .as_array()
            .expect("schema report should export structured audit signals");
        assert!(
            audit_signals
                .iter()
                .any(|signal| signal["kind"] == "unknown-fields")
        );
        assert!(
            audit_signals
                .iter()
                .any(|signal| signal["kind"] == "storage-data-hash-change")
        );
    }

    #[test]
    fn infer_schema_candidates_records_transaction_evidence_sources() {
        let mut flow = sample_flow("tx-a", Some("0x00000001"));
        let mut outbound = flow.inbound.clone();
        outbound.direction = MessageDirection::Outbound;
        outbound.index = Some(0);
        outbound.kind = "internal".to_owned();
        flow.outbound.push(outbound);
        flow.out_actions.push(super::ActionEffect {
            index: 0,
            kind: "send_msg".to_owned(),
            mode: Some("64".to_owned()),
            value_nanotons: Some("1".to_owned()),
            destination: Some("dst".to_owned()),
            body: None,
            code: None,
            library: None,
        });
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![flow],
            failures: Vec::new(),
        };

        let report = super::infer_schema_candidates(&corpus);
        let json = serde_json::to_value(&report).unwrap();
        let evidence = &json["opcodeCandidates"][0]["evidence"][0];

        assert_eq!(evidence["txHash"], "tx-a");
        assert_eq!(evidence["inboundBodyHash"], "hash");
        assert_eq!(evidence["inboundBodyBits"], 32);
        assert_eq!(evidence["inboundBodyRefs"], 0);
        assert_eq!(evidence["fromStatus"], "none");
        assert_eq!(evidence["toStatus"], "active");
        assert_eq!(evidence["preDataHash"], serde_json::Value::Null);
        assert_eq!(evidence["postDataHash"], "data");
        assert_eq!(evidence["preCodeHash"], serde_json::Value::Null);
        assert_eq!(evidence["postCodeHash"], "code");
        assert_eq!(evidence["outboundKinds"], serde_json::json!(["internal"]));
        assert_eq!(evidence["outActionKinds"], serde_json::json!(["send_msg"]));
    }

    #[test]
    fn infer_schema_candidates_reports_inbound_body_field_candidates() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 2,
            source_tx_count: 2,
            retraced_count: 2,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![
                sample_flow_with_body_fields("tx-a", 0x0000_0001, 7, 0xaa),
                sample_flow_with_body_fields("tx-b", 0x0000_0001, 8, 0xbb),
            ],
            failures: Vec::new(),
        };
        let report = super::infer_schema_candidates(&corpus);
        let candidate = &report.opcode_candidates[0];

        assert_eq!(candidate.inbound_body.field_candidates.len(), 3);
        assert_eq!(candidate.inbound_body.field_candidates[0].name, "opcode");
        assert_eq!(candidate.inbound_body.field_candidates[0].bit_offset, 0);
        assert_eq!(candidate.inbound_body.field_candidates[0].min_bits, 32);
        assert_eq!(
            candidate.inbound_body.field_candidates[0].value_samples,
            vec!["0x00000001".to_owned()]
        );
        assert_eq!(candidate.inbound_body.field_candidates[1].name, "query_id");
        assert_eq!(candidate.inbound_body.field_candidates[1].bit_offset, 32);
        assert_eq!(candidate.inbound_body.field_candidates[1].min_bits, 64);
        assert_eq!(
            candidate.inbound_body.field_candidates[1].value_samples,
            vec![
                "0x0000000000000007".to_owned(),
                "0x0000000000000008".to_owned()
            ]
        );
        assert_eq!(
            candidate.inbound_body.field_candidates[2].name,
            "payload_tail"
        );
        assert_eq!(candidate.inbound_body.field_candidates[2].bit_offset, 96);
        assert_eq!(candidate.inbound_body.field_candidates[2].min_bits, 8);
        assert_eq!(candidate.replay_probes.len(), 2);
        assert_eq!(candidate.replay_probes[1].field_name, "query_id");
        assert_eq!(candidate.replay_probes[1].bit_offset, 32);
        assert_eq!(candidate.replay_probes[1].bits, 64);
        assert_eq!(
            candidate.replay_probes[1].cli_arg,
            "--set-body-uint 32:64:0x0000000000000006"
        );

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(
            json["opcodeCandidates"][0]["inboundBody"]["fieldCandidates"][1]["name"],
            "query_id"
        );
        assert_eq!(
            json["opcodeCandidates"][0]["inboundBody"]["fieldCandidates"][1]["valueSamples"],
            serde_json::json!(["0x0000000000000007", "0x0000000000000008"])
        );
        assert_eq!(
            json["opcodeCandidates"][0]["replayProbes"][1]["mutation"],
            serde_json::json!({
                "type": "setBodyUint",
                "bitOffset": 32,
                "bits": 64,
                "value": "0x0000000000000006"
            })
        );
    }

    #[test]
    fn infer_schema_candidates_reports_storage_field_candidates() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 2,
            source_tx_count: 2,
            retraced_count: 2,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![
                sample_flow_with_storage_data("tx-a", 0xdead_beef, 0xaa),
                sample_flow_with_storage_data("tx-b", 0xcafe_babe, 0xbb),
            ],
            failures: Vec::new(),
        };
        let corpus: StateFlowCorpus =
            serde_json::from_value(serde_json::to_value(corpus).unwrap()).unwrap();
        assert!(
            corpus.transactions[0]
                .state
                .post
                .data_cell
                .as_ref()
                .and_then(|shape| shape.boc64.as_ref())
                .is_some(),
            "persisted corpus must retain data cell BoC evidence for offline inference"
        );

        let report = super::infer_schema_candidates(&corpus);
        let candidate = &report.opcode_candidates[0];

        assert_eq!(candidate.storage.fields.len(), 2);
        assert_eq!(candidate.storage.fields[0].name, "data_word_0");
        assert_eq!(candidate.storage.fields[0].cell_path, "data");
        assert_eq!(candidate.storage.fields[0].bit_offset, 0);
        assert_eq!(candidate.storage.fields[0].kind, "uint32");
        assert_eq!(
            candidate.storage.fields[0].value_samples,
            vec!["0xcafebabe".to_owned(), "0xdeadbeef".to_owned()]
        );
        assert_eq!(candidate.storage.fields[1].name, "data_tail");
        assert_eq!(candidate.storage.fields[1].bit_offset, 32);
        assert_eq!(candidate.storage.fields[1].min_bits, 8);

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(
            json["opcodeCandidates"][0]["storage"]["fields"][0]["valueSamples"],
            serde_json::json!(["0xcafebabe", "0xdeadbeef"])
        );
    }

    #[test]
    fn infer_schema_candidates_reports_outbound_effect_evidence() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![sample_flow_with_effects("tx-a")],
            failures: Vec::new(),
        };

        let report = super::infer_schema_candidates(&corpus);
        let candidate = &report.opcode_candidates[0];
        let outbound = &candidate.outbound_effects[0];
        let action = &candidate.out_actions[0];

        assert_eq!(outbound.kind, "internal");
        assert_eq!(outbound.tx_hashes, vec!["tx-a".to_owned()]);
        assert_eq!(outbound.destinations, vec!["out-dst".to_owned()]);
        assert_eq!(outbound.value_nanotons_min.as_deref(), Some("11"));
        assert_eq!(outbound.value_nanotons_max.as_deref(), Some("11"));
        assert_eq!(outbound.body_shape.as_ref().unwrap().min_bits, 40);
        assert_eq!(outbound.body_shape.as_ref().unwrap().min_refs, 1);

        assert_eq!(action.kind, "send-message");
        assert_eq!(action.modes, vec!["64".to_owned()]);
        assert_eq!(action.destinations, vec!["action-dst".to_owned()]);
        assert_eq!(action.value_nanotons_min.as_deref(), Some("7"));
        assert_eq!(action.value_nanotons_max.as_deref(), Some("7"));
        assert_eq!(action.body_shape.as_ref().unwrap().min_bits, 32);

        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(
            json["opcodeCandidates"][0]["outboundEffects"][0]["destinations"],
            serde_json::json!(["out-dst"])
        );
        assert_eq!(
            json["opcodeCandidates"][0]["outActions"][0]["modes"],
            serde_json::json!(["64"])
        );
    }

    #[test]
    fn replay_mutation_flips_message_body_bit() {
        let mut body = CellBuilder::new();
        body.store_u32(0).unwrap();
        let message = OwnedMessage {
            info: MsgInfo::Int(IntMsgInfo::default()),
            init: None,
            body: body.build().unwrap().into(),
            layout: None,
        };
        let message_boc64 = Boc::encode_base64(super::to_cell(&message).unwrap());

        let mutated =
            super::apply_replay_mutation(&message_boc64, &ReplayMutation::FlipBodyBit { bit: 0 })
                .unwrap();
        let mutated_cell = Boc::decode_base64(mutated).unwrap();
        let mutated_message = mutated_cell
            .parse::<tycho_types::models::Message<'_>>()
            .unwrap();
        let mut mutated_body = mutated_message.body;

        assert_eq!(mutated_body.load_u32().unwrap(), 0x8000_0000);
    }

    #[test]
    fn replay_mutation_sets_message_body_uint_field() {
        let mut body = CellBuilder::new();
        body.store_u32(0x0000_0001).unwrap();
        body.store_u64(7).unwrap();
        body.store_raw(&[0xaa], 8).unwrap();
        let message = OwnedMessage {
            info: MsgInfo::Int(IntMsgInfo::default()),
            init: None,
            body: body.build().unwrap().into(),
            layout: None,
        };
        let message_boc64 = Boc::encode_base64(super::to_cell(&message).unwrap());

        let mutated = super::apply_replay_mutation(
            &message_boc64,
            &ReplayMutation::SetBodyUint {
                bit_offset: 32,
                bits: 64,
                value: "42".to_owned(),
            },
        )
        .unwrap();
        let mutated_cell = Boc::decode_base64(mutated).unwrap();
        let mutated_message = mutated_cell
            .parse::<tycho_types::models::Message<'_>>()
            .unwrap();
        let mut mutated_body = mutated_message.body;

        assert_eq!(mutated_body.load_u32().unwrap(), 0x0000_0001);
        assert_eq!(mutated_body.load_u64().unwrap(), 42);
        assert_eq!(mutated_body.load_uint(8).unwrap(), 0xaa);
    }

    #[test]
    fn replay_mutation_json_uses_artifact_field_names() {
        let replace = serde_json::to_value(&ReplayMutation::ReplaceBody {
            body_boc64: "body".to_owned(),
        })
        .unwrap();
        assert_eq!(
            replace,
            serde_json::json!({"type": "replaceBody", "bodyBoc64": "body"})
        );

        let set_uint = serde_json::to_value(&ReplayMutation::SetBodyUint {
            bit_offset: 32,
            bits: 64,
            value: "42".to_owned(),
        })
        .unwrap();
        assert_eq!(
            set_uint,
            serde_json::json!({
                "type": "setBodyUint",
                "bitOffset": 32,
                "bits": 64,
                "value": "42"
            })
        );
    }

    #[test]
    fn replay_args_include_captured_libraries() {
        let flow = sample_flow("tx-a", Some("0x00000001"));

        let args = super::replay_args_from_flow(&flow, "message".to_owned(), true);

        assert_eq!(args.libs_boc64.as_deref(), Some("libs"));
        assert_eq!(args.message_boc64, "message");
        assert!(args.ignore_chksig);
    }

    #[test]
    fn report_renderer_includes_evidence_confidence_and_unknowns() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![sample_flow("tx-a", Some("0x00000001"))],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);

        let report = super::render_state_flow_report(&corpus, &schema, &[]);

        assert!(report.contains("# TON State Flow Reverse Report"));
        assert!(report.contains("## Opcode Candidates"));
        assert!(report.contains("Storage"));
        assert!(report.contains("balance -3"));
        assert!(report.contains("data shape 16/1"));
        assert!(report.contains("code shape 8/0"));
        assert!(report.contains("## State Machine"));
        assert!(report.contains("```mermaid"));
        assert!(report.contains("stateDiagram-v2"));
        assert!(report.contains("none --> active: 0x00000001 (1)"));
        assert!(report.contains("medium"));
        assert!(report.contains("tx-a"));
        assert!(report.contains("## Schema Evidence"));
        assert!(report.contains("| Opcode | Tx | Body hash | Body bits/refs | State | Data hash | Code hash | Outbound | Actions |"));
        assert!(report.contains("| `0x00000001` | `tx-a` | `hash` | 32/0 | none -> active | `<none>` -> `data` | `<none>` -> `code` | none | none |"));
        assert!(report.contains("## Unknown Fields"));
        assert!(report.contains("TL-B"));
    }

    #[test]
    fn report_renderer_includes_state_machine_edge_evidence() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 2,
            source_tx_count: 2,
            retraced_count: 2,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![
                sample_flow("tx-a", Some("0x00000001")),
                sample_flow("tx-b", Some("0x00000001")),
            ],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);

        let report = super::render_state_flow_report(&corpus, &schema, &[]);

        assert!(report.contains("## State Machine Evidence"));
        assert!(report.contains("| From | To | Opcode | Count | Confidence | Evidence |"));
        assert!(report.contains("| none | active | `0x00000001` | 2 | medium | `tx-a`, `tx-b` |"));
    }

    #[test]
    fn report_renderer_includes_unknown_field_confidence_and_evidence() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 2,
            source_tx_count: 2,
            retraced_count: 2,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![
                sample_flow("tx-a", Some("0x00000001")),
                sample_flow("tx-b", Some("0x00000001")),
            ],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);

        let report = super::render_state_flow_report(&corpus, &schema, &[]);

        assert!(report.contains(
            "  - message body field names require TL-B recovery (confidence: medium; evidence: `tx-a`, `tx-b`)"
        ));
    }

    #[test]
    fn report_renderer_includes_message_body_field_candidates() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![sample_flow_with_body_fields("tx-a", 0x0000_0001, 7, 0xaa)],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);

        let report = super::render_state_flow_report(&corpus, &schema, &[]);

        assert!(report.contains("## Message Body Fields"));
        assert!(
            report.contains(
                "| Opcode | Field | Offset | Bits | Refs | Kind | Samples | Confidence |"
            )
        );
        assert!(report.contains("| `0x00000001` | `query_id` | 32 | 64..64 | 0..0 | uint64 | `0x0000000000000007` | high |"));
        assert!(report.contains(
            "| `0x00000001` | `payload_tail` | 96 | 8..8 | 0..0 | raw | `8 bits, 0 refs` | low |"
        ));
        assert!(report.contains("## Replay Probes"));
        assert!(report.contains("| `0x00000001` | `query_id` | `--set-body-uint 32:64:0x0000000000000006` | high | `tx-a` |"));
    }

    #[test]
    fn report_renderer_includes_storage_field_candidates() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![sample_flow_with_storage_data("tx-a", 0xdead_beef, 0xaa)],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);

        let report = super::render_state_flow_report(&corpus, &schema, &[]);

        assert!(report.contains("## Storage Fields"));
        assert!(report.contains(
            "| Opcode | Field | Cell | Offset | Bits | Refs | Kind | Samples | Confidence |"
        ));
        assert!(report.contains("| `0x00000001` | `data_word_0` | data | 0 | 32..32 | 0..0 | uint32 | `0xdeadbeef` | high |"));
        assert!(report.contains("| `0x00000001` | `data_tail` | data | 32 | 8..8 | 0..0 | raw | `8 bits, 0 refs` | low |"));
    }

    #[test]
    fn report_renderer_includes_outbound_effect_candidates() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![sample_flow_with_effects("tx-a")],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);

        let report = super::render_state_flow_report(&corpus, &schema, &[]);

        assert!(report.contains("## Outbound Effects"));
        assert!(report.contains("| Opcode | Source | Kind | Count | Value | Modes | Destinations | Body | Code | Libraries | Evidence |"));
        assert!(report.contains("| `0x00000001` | outbound | internal | 1 | 11 | none | `out-dst` | 40/1 | n/a | none | `tx-a` |"));
        assert!(report.contains("| `0x00000001` | action | send-message | 1 | 7 | `64` | `action-dst` | 32/0 | n/a | none | `tx-a` |"));
    }

    #[test]
    fn report_renderer_includes_risk_points_from_schema_and_replay_diffs() {
        let corpus = StateFlowCorpus {
            schema_version: 1,
            network: "mainnet".to_owned(),
            address: "addr".to_owned(),
            requested_limit: 1,
            source_tx_count: 1,
            retraced_count: 1,
            failure_count: 0,
            opcode_summary: Vec::new(),
            transactions: vec![sample_flow("tx-a", Some("0x00000001"))],
            failures: Vec::new(),
        };
        let schema = super::infer_schema_candidates(&corpus);
        let replays = vec![sample_replay_diff(
            "tx-a",
            ReplayMutation::FlipBodyBit { bit: 32 },
            true,
            Some(true),
        )];

        let report = super::render_state_flow_report(&corpus, &schema, &replays);

        assert!(report.contains("- Replay diffs: 1"));
        assert!(report.contains("## Risk Points"));
        assert!(report.contains("Unknown fields remain for opcode 0x00000001"));
        assert!(report.contains("Evidence: `tx-a`."));
        assert!(report.contains("Mutation `flip body bit 32` changed state for `tx-a`"));
    }

    #[test]
    fn schema_report_deserializes_without_storage_for_old_artifacts() {
        let json = r#"{
            "schemaVersion": 1,
            "network": "mainnet",
            "address": "addr",
            "transactionCount": 1,
            "opcodeCandidates": [{
                "opcode": "0x00000001",
                "count": 1,
                "examples": ["tx-a"],
                "inboundBody": {
                    "minBits": 32,
                    "maxBits": 32,
                    "minRefs": 0,
                    "maxRefs": 0,
                    "bodyHashes": ["hash"]
                },
                "stateTransitions": [],
                "outboundEffects": [],
                "outActions": [],
                "confidence": "medium",
                "unknownFields": []
            }]
        }"#;

        let report: super::StateFlowSchemaReport = serde_json::from_str(json).unwrap();

        assert_eq!(report.opcode_candidates[0].storage.balance_delta_min, 0);
        assert_eq!(
            report.opcode_candidates[0].storage.post_data_hashes.len(),
            0
        );
        assert_eq!(report.opcode_candidates[0].evidence.len(), 0);
        let serialized = serde_json::to_value(&report).unwrap();
        assert_eq!(serialized["stateMachine"]["edges"], serde_json::json!([]));
        assert_eq!(serialized["auditSignals"], serde_json::json!([]));
    }

    #[test]
    fn checked_in_smoke_targets_cover_named_tonviewer_account() {
        let targets: serde_json::Value =
            serde_json::from_str(include_str!("../smoke-targets.json")).unwrap();
        let targets = targets["targets"]
            .as_array()
            .expect("smoke targets should be an array");

        assert!(
            targets.len() >= 2,
            "state-flow smoke coverage should include multiple real-chain examples"
        );
        assert!(
            targets.iter().any(|target| {
                target["id"] == "tonviewer-requested-target"
                    && target["network"] == "mainnet"
                    && target["address"] == "EQAgvOlWk7C0Pz3YgSaX-MA7UDDhE9n6eQgQRwJahOBm4VKr"
                    && target["sourceUrl"]
                        == "https://tonviewer.com/EQAgvOlWk7C0Pz3YgSaX-MA7UDDhE9n6eQgQRwJahOBm4VKr"
                    && target["collectLimit"]
                        .as_u64()
                        .is_some_and(|limit| limit >= 2)
            }),
            "smoke targets must keep the user-requested Tonviewer account"
        );
    }

    fn sample_flow(query_hash: &str, opcode: Option<&str>) -> StateFlowTx {
        StateFlowTx {
            schema_version: 1,
            network: "mainnet".to_owned(),
            query_hash: query_hash.to_owned(),
            transaction: TransactionIdentity {
                lt: 42,
                utime: 1,
                account: "addr".to_owned(),
                state_update_hash_ok: true,
                transaction_boc64: "tx".to_owned(),
            },
            replay: ReplaySummary {
                mc_seqno: 7,
                rand_seed_hex: "00".to_owned(),
                replayed_prev_tx_count: 0,
                block_config_boc64: "config".to_owned(),
                libs_boc64: Some("libs".to_owned()),
            },
            state: StateTransition {
                pre: super::ShardAccountSnapshot {
                    shard_account_boc64: "pre".to_owned(),
                    last_trans_lt: 0,
                    last_trans_hash: "00".to_owned(),
                    account_address: None,
                    status: "none".to_owned(),
                    balance_nanotons: "0".to_owned(),
                    code_hash: None,
                    data_hash: None,
                    code_cell: None,
                    data_cell: None,
                    frozen_hash: None,
                },
                post: super::ShardAccountSnapshot {
                    shard_account_boc64: "post".to_owned(),
                    last_trans_lt: 42,
                    last_trans_hash: "11".to_owned(),
                    account_address: Some("addr".to_owned()),
                    status: "active".to_owned(),
                    balance_nanotons: "1".to_owned(),
                    code_hash: Some("code".to_owned()),
                    data_hash: Some("data".to_owned()),
                    code_cell: Some(super::CellShape {
                        boc64: None,
                        hash: "code".to_owned(),
                        bits: 8,
                        refs: 0,
                    }),
                    data_cell: Some(super::CellShape {
                        boc64: None,
                        hash: "data".to_owned(),
                        bits: 16,
                        refs: 1,
                    }),
                    frozen_hash: None,
                },
            },
            inbound: super::MessageArtifact {
                direction: MessageDirection::Inbound,
                index: None,
                kind: "internal".to_owned(),
                src: Some("src".to_owned()),
                dst: Some("dst".to_owned()),
                value_nanotons: Some("1".to_owned()),
                bounced: Some(false),
                bounce: Some(true),
                opcode: opcode.map(ToOwned::to_owned),
                message_boc64: "msg".to_owned(),
                body: CellArtifact {
                    boc64: "body".to_owned(),
                    hash: "hash".to_owned(),
                    bits: 32,
                    refs: 0,
                },
            },
            outbound: Vec::new(),
            compute: StateFlowCompute {
                skipped: false,
                success: Some(true),
                exit_code: Some(0),
                vm_steps: Some(1),
                gas_used: Some(2),
                gas_fees: Some(3),
            },
            money: MoneyFlow {
                balance_before: 10,
                sent_total: 1,
                total_fees: 2,
                balance_after: 7,
            },
            c5: None,
            out_actions: Vec::new(),
            vm_trace: LogArtifact {
                line_count: 0,
                text: String::new(),
            },
            executor_trace: LogArtifact {
                line_count: 0,
                text: String::new(),
            },
        }
    }

    fn sample_flow_with_body_fields(
        query_hash: &str,
        opcode: u32,
        query_id: u64,
        tail: u8,
    ) -> StateFlowTx {
        let mut flow = sample_flow(query_hash, Some(&format!("0x{opcode:08x}")));
        let mut builder = CellBuilder::new();
        builder.store_u32(opcode).unwrap();
        builder.store_u64(query_id).unwrap();
        builder.store_raw(&[tail], 8).unwrap();
        let body = builder.build().unwrap();
        flow.inbound.body = super::cell_artifact(&body).unwrap();
        flow
    }

    fn sample_flow_with_storage_data(query_hash: &str, word: u32, tail: u8) -> StateFlowTx {
        let mut flow = sample_flow(query_hash, Some("0x00000001"));
        let mut builder = CellBuilder::new();
        builder.store_u32(word).unwrap();
        builder.store_raw(&[tail], 8).unwrap();
        let data = builder.build().unwrap();
        flow.state.post.data_hash = Some(super::cell_hash(&data));
        flow.state.post.data_cell = Some(super::cell_shape(&data));
        flow
    }

    fn sample_flow_with_effects(query_hash: &str) -> StateFlowTx {
        let mut flow = sample_flow(query_hash, Some("0x00000001"));
        flow.outbound.push(super::MessageArtifact {
            direction: MessageDirection::Outbound,
            index: Some(0),
            kind: "internal".to_owned(),
            src: Some("addr".to_owned()),
            dst: Some("out-dst".to_owned()),
            value_nanotons: Some("11".to_owned()),
            bounced: Some(false),
            bounce: Some(true),
            opcode: Some("0x00000002".to_owned()),
            message_boc64: "out-msg".to_owned(),
            body: sample_cell_artifact("out-body", 40, 1),
        });
        flow.out_actions.push(super::ActionEffect {
            index: 0,
            kind: "send-message".to_owned(),
            mode: Some("64".to_owned()),
            value_nanotons: Some("7".to_owned()),
            destination: Some("action-dst".to_owned()),
            body: Some(sample_cell_artifact("action-body", 32, 0)),
            code: None,
            library: None,
        });
        flow
    }

    fn sample_cell_artifact(hash: &str, bits: u16, refs: u8) -> CellArtifact {
        CellArtifact {
            boc64: format!("{hash}-boc"),
            hash: hash.to_owned(),
            bits,
            refs,
        }
    }

    fn sample_replay_diff(
        source_query_hash: &str,
        mutation: ReplayMutation,
        replay_accepted: bool,
        state_changed: Option<bool>,
    ) -> StateFlowReplayDiff {
        StateFlowReplayDiff {
            schema_version: 1,
            source_query_hash: source_query_hash.to_owned(),
            mutation,
            ignore_chksig: false,
            baseline: ReplayObservation {
                accepted: true,
                state: None,
                inbound: sample_flow(source_query_hash, Some("0x00000001")).inbound,
                outbound: Vec::new(),
                compute: None,
                money: None,
                c5: None,
                out_actions: Vec::new(),
                vm_trace: None,
                executor_trace: None,
                error: None,
            },
            replay: ReplayObservation {
                accepted: replay_accepted,
                state: None,
                inbound: sample_flow(source_query_hash, Some("0x00000001")).inbound,
                outbound: Vec::new(),
                compute: None,
                money: None,
                c5: None,
                out_actions: Vec::new(),
                vm_trace: None,
                executor_trace: None,
                error: None,
            },
            diff: ReplayDiffSummary {
                replay_accepted,
                input_changed: true,
                state_changed,
                code_hash_changed: Some(false),
                data_hash_changed: Some(false),
                balance_delta_diff: Some(0),
                exit_code_changed: Some(false),
                outbound_count_delta: Some(0),
                action_count_delta: Some(0),
                c5_changed: Some(false),
            },
        }
    }
}
