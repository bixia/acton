use super::utils::handle_result;
use crate::api::toncenter_v2 as v2;
use crate::localnet::Localnet;
use crate::server::models::{
    FaucetRequest, GetAddressNameQuery, GetCompilerAbiQuery, RegisterCompilerAbisRequest,
    SendBocRequest, SetAddressNameRequest, SetShardAccountRequest, StatePathRequest,
};
use crate::server::{StartupWallet, StateSourceInfo};
use crate::types::Hash256;
use anyhow::Context;
use axum::{Json, extract::State};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const STATE_FLOW_ARTIFACT_BUNDLE_DIRS: &[&str] =
    &["target/stateflow-smoke", "target/stateflow-analysis"];
const STATE_FLOW_ARTIFACT_SOURCE_KIND: &str = "stateFlowArtifactBundle";

pub async fn faucet(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<FaucetRequest>,
) -> Json<Value> {
    handle_result(node.faucet(payload.address, payload.amount), |res| {
        res.clone()
    })
    .await
}

#[derive(Serialize)]
struct LocalnetAdminStatus {
    uptime_seconds: u64,
    last_block_seqno: u64,
    #[serde(flatten)]
    state_source: StateSourceInfo,
}

#[derive(Serialize)]
struct StateFlowArtifactBundleResponse {
    kind: &'static str,
    sources: Vec<StateFlowArtifactSource>,
}

#[derive(Serialize)]
struct StateFlowArtifactSource {
    name: String,
    raw: String,
}

pub async fn get_status(
    State(node): State<Arc<Localnet>>,
    State(state_source): State<Arc<StateSourceInfo>>,
) -> Json<Value> {
    handle_result(
        async move {
            let masterchain_info = node.get_masterchain_info().await?;

            Ok(LocalnetAdminStatus {
                uptime_seconds: node.uptime_seconds(),
                last_block_seqno: u64::from(masterchain_info.last.seqno),
                state_source: state_source.as_ref().clone(),
            })
        },
        |res| serde_json::to_value(res).unwrap_or(Value::Null),
    )
    .await
}

pub async fn get_state_flow_artifacts(State(project_root): State<Arc<PathBuf>>) -> Json<Value> {
    handle_result(
        async move {
            let sources = collect_state_flow_artifact_sources(&project_root)?;
            Ok::<_, anyhow::Error>(StateFlowArtifactBundleResponse {
                kind: STATE_FLOW_ARTIFACT_SOURCE_KIND,
                sources,
            })
        },
        |res| serde_json::to_value(res).unwrap_or(Value::Null),
    )
    .await
}

pub async fn get_startup_wallets(
    State(startup_wallets): State<Arc<Vec<StartupWallet>>>,
) -> Json<Value> {
    handle_result(
        async move { Ok::<_, anyhow::Error>(startup_wallets.as_ref().clone()) },
        |res| serde_json::to_value(res).unwrap_or(Value::Null),
    )
    .await
}

pub async fn dump_state(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<StatePathRequest>,
) -> Json<Value> {
    handle_result(node.dump_state(payload.path), |()| Value::Null).await
}

pub async fn load_state(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<StatePathRequest>,
) -> Json<Value> {
    handle_result(node.load_state(payload.path), |()| Value::Null).await
}

pub async fn set_shard_account(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<SetShardAccountRequest>,
) -> Json<Value> {
    handle_result(
        node.set_shard_account(payload.address, payload.shard_account),
        |()| Value::Null,
    )
    .await
}

pub async fn send_internal_message(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<SendBocRequest>,
) -> Json<Value> {
    handle_result(
        node.send_internal_boc(payload.boc),
        v2::map_send_boc_return_hash,
    )
    .await
}

pub async fn set_address_name(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<SetAddressNameRequest>,
) -> Json<Value> {
    handle_result(node.set_address_name(payload.address, payload.name), |()| {
        Value::Null
    })
    .await
}

pub async fn get_address_name(
    State(node): State<Arc<Localnet>>,
    axum::extract::Query(payload): axum::extract::Query<GetAddressNameQuery>,
) -> Json<Value> {
    handle_result(node.get_address_name(payload.address), |res| {
        serde_json::to_value(res).unwrap_or(Value::Null)
    })
    .await
}

pub async fn register_compiler_abis(
    State(node): State<Arc<Localnet>>,
    Json(payload): Json<RegisterCompilerAbisRequest>,
) -> Json<Value> {
    handle_result(
        async move {
            let entries = payload
                .entries
                .into_iter()
                .map(|entry| Ok((parse_hash_any(&entry.code_hash)?, entry.compiler_abi)))
                .collect::<anyhow::Result<Vec<_>>>()?;
            node.register_compiler_abis(entries).await
        },
        |()| Value::Null,
    )
    .await
}

pub async fn get_compiler_abi(
    State(node): State<Arc<Localnet>>,
    axum::extract::Query(payload): axum::extract::Query<GetCompilerAbiQuery>,
) -> Json<Value> {
    handle_result(
        async move {
            let code_hash = parse_hash_any(&payload.code_hash)?;
            node.get_compiler_abi(code_hash).await
        },
        |res| res.clone().unwrap_or(Value::Null),
    )
    .await
}

fn parse_hash_any(hash: &str) -> anyhow::Result<Hash256> {
    if let Ok(parsed) = Hash256::from_hex(hash) {
        return Ok(parsed);
    }
    if let Ok(parsed) = Hash256::from_base64(hash) {
        return Ok(parsed);
    }
    anyhow::bail!("Invalid hash format")
}

fn collect_state_flow_artifact_sources(
    project_root: &Path,
) -> anyhow::Result<Vec<StateFlowArtifactSource>> {
    let mut sources = Vec::new();
    for relative_dir in STATE_FLOW_ARTIFACT_BUNDLE_DIRS {
        let bundle_dir = project_root.join(relative_dir);
        if !bundle_dir.join("artifacts.json").is_file() {
            continue;
        }

        let bundle_dir = fs::canonicalize(&bundle_dir)
            .with_context(|| format!("failed to resolve {}", bundle_dir.display()))?;
        if !bundle_dir.starts_with(project_root) {
            continue;
        }

        collect_state_flow_artifact_sources_from_dir(project_root, &bundle_dir, &mut sources)?;
    }
    sources.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(sources)
}

fn collect_state_flow_artifact_sources_from_dir(
    project_root: &Path,
    dir: &Path,
    sources: &mut Vec<StateFlowArtifactSource>,
) -> anyhow::Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed to list {}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", path.display()))?;
        if file_type.is_dir() {
            collect_state_flow_artifact_sources_from_dir(project_root, &path, sources)?;
            continue;
        }
        if !file_type.is_file() || !is_state_flow_artifact_source_file(&path) {
            continue;
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let name = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        sources.push(StateFlowArtifactSource { name, raw });
    }

    Ok(())
}

fn is_state_flow_artifact_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("json" | "md" | "txt")
    )
}
