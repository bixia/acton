use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use ton_retrace::{AccountTxRef, ComputeInfo, Network, TraceResult};
use tycho_types::boc::Boc;
use tycho_types::cell::{Cell, CellBuilder, CellFamily, CellSlice, HashBytes, Store};
use tycho_types::models::{
    AccountState, IntAddr, MsgInfo, OutAction, RelaxedMsgInfo, ShardAccount,
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
    pub opcode_candidates: Vec<OpcodeSchemaCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpcodeSchemaCandidate {
    pub opcode: Option<String>,
    pub count: usize,
    pub examples: Vec<String>,
    pub inbound_body: BodyShapeCandidate,
    pub state_transitions: Vec<StateTransitionCandidate>,
    pub outbound_effects: Vec<EffectCandidate>,
    pub out_actions: Vec<EffectCandidate>,
    pub confidence: String,
    pub unknown_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyShapeCandidate {
    pub min_bits: u16,
    pub max_bits: u16,
    pub min_refs: u8,
    pub max_refs: u8,
    pub body_hashes: Vec<String>,
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
    pub frozen_hash: Option<String>,
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

    StateFlowSchemaReport {
        schema_version: STATE_FLOW_SCHEMA_VERSION,
        network: corpus.network.clone(),
        address: corpus.address.clone(),
        transaction_count: corpus.transactions.len(),
        opcode_candidates: by_opcode
            .into_iter()
            .map(|(opcode, transactions)| opcode_candidate(opcode, &transactions))
            .collect(),
    }
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

fn opcode_candidate(
    opcode: Option<String>,
    transactions: &[&StateFlowTx],
) -> OpcodeSchemaCandidate {
    let inbound_body = inbound_body_shape(transactions);
    let state_transitions = summarize_pairs(
        transactions
            .iter()
            .map(|tx| (tx.state.pre.status.clone(), tx.state.post.status.clone())),
    );
    let outbound_effects = summarize_kinds(
        transactions
            .iter()
            .flat_map(|tx| tx.outbound.iter().map(|msg| msg.kind.clone())),
    );
    let out_actions = summarize_kinds(
        transactions
            .iter()
            .flat_map(|tx| tx.out_actions.iter().map(|action| action.kind.clone())),
    );
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

    OpcodeSchemaCandidate {
        opcode,
        count: transactions.len(),
        examples: transactions
            .iter()
            .take(5)
            .map(|tx| tx.query_hash.clone())
            .collect(),
        inbound_body,
        state_transitions,
        outbound_effects,
        out_actions,
        confidence,
        unknown_fields,
    }
}

fn inbound_body_shape(transactions: &[&StateFlowTx]) -> BodyShapeCandidate {
    let mut min_bits = u16::MAX;
    let mut max_bits = 0;
    let mut min_refs = u8::MAX;
    let mut max_refs = 0;
    let mut body_hashes = BTreeSet::new();
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

fn summarize_kinds(kinds: impl Iterator<Item = String>) -> Vec<EffectCandidate> {
    let mut counts = BTreeMap::<String, usize>::new();
    for kind in kinds {
        *counts.entry(kind).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| EffectCandidate { kind, count })
        .collect()
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
    let (account_address, status, balance_nanotons, code_hash, data_hash, frozen_hash) =
        if let Some(account) = account {
            let balance = account.balance.tokens.to_string();
            match account.state {
                AccountState::Uninit => (
                    Some(format_int_addr(&account.address)),
                    "uninit".to_owned(),
                    balance,
                    None,
                    None,
                    None,
                ),
                AccountState::Active(state) => (
                    Some(format_int_addr(&account.address)),
                    "active".to_owned(),
                    balance,
                    state.code.as_ref().map(cell_hash),
                    state.data.as_ref().map(cell_hash),
                    None,
                ),
                AccountState::Frozen(hash) => (
                    Some(format_int_addr(&account.address)),
                    "frozen".to_owned(),
                    balance,
                    None,
                    None,
                    Some(hash.to_string()),
                ),
            }
        } else {
            (None, "none".to_owned(), "0".to_owned(), None, None, None)
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
    let slice = cell.as_slice_allow_exotic();
    Ok(CellArtifact {
        boc64: Boc::encode_base64(cell),
        hash: cell_hash(cell),
        bits: slice.size_bits(),
        refs: slice.size_refs(),
    })
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
        CellArtifact, LogArtifact, MessageDirection, MoneyFlow, ReplaySummary, StateFlowCompute,
        StateFlowCorpus, StateFlowTx, StateTransition, TransactionIdentity,
    };

    #[test]
    fn state_flow_tx_serializes_camel_case_schema_version() {
        let flow = sample_flow("abc", Some("0x00000001"));

        let json = serde_json::to_value(&flow).expect("state flow should serialize");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["transaction"]["stateUpdateHashOk"], true);
        assert_eq!(json["inbound"]["direction"], "inbound");
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
        assert_eq!(candidate.state_transitions[0].from_status, "none");
        assert_eq!(candidate.state_transitions[0].to_status, "active");
        assert_eq!(candidate.confidence, "medium");
        assert!(
            candidate
                .unknown_fields
                .iter()
                .any(|field| field.contains("TL-B"))
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
}
