use anyhow::Context;
use serde::Serialize;
use std::collections::HashMap;
use ton_retrace::{ComputeInfo, Network, TraceResult};
use tycho_types::boc::Boc;
use tycho_types::cell::{Cell, CellBuilder, CellFamily, CellSlice, HashBytes, Store};
use tycho_types::models::{
    AccountState, IntAddr, MsgInfo, OutAction, RelaxedMsgInfo, ShardAccount,
};

pub const STATE_FLOW_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionIdentity {
    pub lt: u64,
    pub utime: u64,
    pub account: String,
    pub state_update_hash_ok: bool,
    pub transaction_boc64: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySummary {
    pub mc_seqno: u32,
    pub rand_seed_hex: String,
    pub replayed_prev_tx_count: usize,
    pub block_config_boc64: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTransition {
    pub pre: ShardAccountSnapshot,
    pub post: ShardAccountSnapshot,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellArtifact {
    pub boc64: String,
    pub hash: String,
    pub bits: u16,
    pub refs: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFlowCompute {
    pub skipped: bool,
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
    pub vm_steps: Option<u32>,
    pub gas_used: Option<u64>,
    pub gas_fees: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyFlow {
    pub balance_before: u64,
    pub sent_total: u64,
    pub total_fees: u64,
    pub balance_after: u64,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEffect {
    pub mode: String,
    pub hash: Option<String>,
    pub cell: Option<CellArtifact>,
}

#[derive(Debug, Clone, Serialize)]
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
        StateFlowTx, StateTransition, TransactionIdentity,
    };

    #[test]
    fn state_flow_tx_serializes_camel_case_schema_version() {
        let flow = StateFlowTx {
            schema_version: 1,
            network: "mainnet".to_owned(),
            query_hash: "abc".to_owned(),
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
                opcode: Some("0x00000001".to_owned()),
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
        };

        let json = serde_json::to_value(&flow).expect("state flow should serialize");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["transaction"]["stateUpdateHashOk"], true);
        assert_eq!(json["inbound"]["direction"], "inbound");
    }
}
