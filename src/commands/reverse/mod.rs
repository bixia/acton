use anyhow::Context;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use ton_retrace::Network;
use ton_stateflow::{
    ReplayMutation, StateFlowCorpus, StateFlowReplayDiff, StateFlowSchemaReport, StateFlowTx,
};

const DEFAULT_SMOKE_TARGETS: &str = "crates/ton-stateflow/smoke-targets.json";

#[derive(Subcommand, Clone)]
pub enum ReverseCommand {
    #[command(about = "Replay a transaction and emit state-flow JSON")]
    Retrace {
        #[arg(help = "Transaction hash in hex format to retrace")]
        hash: String,
        #[arg(long, help = "Network to use")]
        net: Option<String>,
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write state-flow JSON to a file"
        )]
        output: Option<PathBuf>,
        #[arg(long, help = "Pretty-print JSON output")]
        pretty: bool,
    },
    #[command(about = "Collect account history into a state-flow corpus")]
    Collect {
        #[arg(help = "Account address in friendly or raw format")]
        address: String,
        #[arg(long, help = "Network to use")]
        net: String,
        #[arg(
            long,
            default_value_t = 10,
            value_parser = clap::value_parser!(u32).range(1..),
            help = "Maximum number of recent account transactions to collect"
        )]
        limit: u32,
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write state-flow corpus JSON to a file"
        )]
        output: Option<PathBuf>,
        #[arg(long, help = "Pretty-print JSON output")]
        pretty: bool,
    },
    #[command(about = "Infer opcode and effect schema candidates from a state-flow corpus")]
    Infer {
        #[arg(help = "State-flow corpus JSON produced by `acton reverse collect`")]
        corpus: PathBuf,
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write schema candidate JSON to a file"
        )]
        output: Option<PathBuf>,
        #[arg(long, help = "Pretty-print JSON output")]
        pretty: bool,
    },
    #[command(about = "Replay or mutate a StateFlowTx artifact and emit a diff")]
    Replay {
        #[arg(help = "StateFlowTx JSON produced by `acton reverse retrace`")]
        state_flow: PathBuf,
        #[arg(
            long,
            conflicts_with = "body_boc64",
            value_name = "FLIP_BODY_BIT",
            help = "Flip one inbound message body bit before replay"
        )]
        flip_body_bit: Option<u16>,
        #[arg(
            long,
            conflicts_with = "flip_body_bit",
            value_name = "BODY_BOC64",
            help = "Replace inbound message body with this base64 BoC before replay"
        )]
        body_boc64: Option<String>,
        #[arg(long, help = "Ignore TVM signature checks during local replay")]
        ignore_chksig: bool,
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write replay diff JSON to a file"
        )]
        output: Option<PathBuf>,
        #[arg(long, help = "Pretty-print JSON output")]
        pretty: bool,
    },
    #[command(about = "Generate a state-flow reverse-engineering report")]
    Report {
        #[arg(help = "State-flow corpus JSON produced by `acton reverse collect`")]
        corpus: PathBuf,
        #[arg(long, help = "Schema candidate JSON produced by `acton reverse infer`")]
        schema: PathBuf,
        #[arg(
            long,
            value_name = "REPLAY",
            help = "Replay diff JSON produced by `acton reverse replay`"
        )]
        replay: Vec<PathBuf>,
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write Markdown report to a file"
        )]
        output: Option<PathBuf>,
    },
    #[command(about = "Run state-flow smoke targets through collect, infer, replay, and report")]
    Smoke {
        #[arg(
            long,
            default_value = DEFAULT_SMOKE_TARGETS,
            help = "Smoke target manifest JSON"
        )]
        targets: PathBuf,
        #[arg(long, help = "Only run the smoke target with this id")]
        target_id: Option<String>,
        #[arg(
            long,
            default_value = "target/stateflow-smoke",
            help = "Directory for smoke output artifacts"
        )]
        out_dir: PathBuf,
        #[arg(long, help = "Pretty-print JSON output artifacts")]
        pretty: bool,
    },
}

pub fn reverse_cmd(command: ReverseCommand) -> anyhow::Result<()> {
    match command {
        ReverseCommand::Retrace {
            hash,
            net,
            output,
            pretty,
        } => reverse_retrace_cmd(&hash, net.as_deref(), output, pretty),
        ReverseCommand::Collect {
            address,
            net,
            limit,
            output,
            pretty,
        } => reverse_collect_cmd(&address, &net, limit, output, pretty),
        ReverseCommand::Infer {
            corpus,
            output,
            pretty,
        } => reverse_infer_cmd(corpus, output, pretty),
        ReverseCommand::Replay {
            state_flow,
            flip_body_bit,
            body_boc64,
            ignore_chksig,
            output,
            pretty,
        } => reverse_replay_cmd(
            state_flow,
            flip_body_bit,
            body_boc64,
            ignore_chksig,
            output,
            pretty,
        ),
        ReverseCommand::Report {
            corpus,
            schema,
            replay,
            output,
        } => reverse_report_cmd(corpus, schema, replay, output),
        ReverseCommand::Smoke {
            targets,
            target_id,
            out_dir,
            pretty,
        } => reverse_smoke_cmd(targets, target_id.as_deref(), out_dir, pretty),
    }
}

fn reverse_retrace_cmd(
    hash: &str,
    net: Option<&str>,
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let networks = if let Some(net) = net {
        vec![Network::from_str(net)?]
    } else {
        vec![Network::Mainnet, Network::Testnet]
    };

    let mut last_error = None;
    for network in networks {
        let result = rt.block_on(ton_stateflow::retrace_with_state_flow(
            network,
            hash,
            HashMap::new(),
        ));
        match result {
            Ok(flow) => return write_state_flow(&flow, output, pretty),
            Err(err) => last_error = Some(err),
        }
    }

    if let Some(err) = last_error {
        anyhow::bail!("Failed to retrace transaction in any network: {err}");
    }
    anyhow::bail!("Failed to retrace transaction");
}

fn reverse_collect_cmd(
    address: &str,
    net: &str,
    limit: u32,
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    let network = Network::from_str(net)?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let corpus = rt.block_on(ton_stateflow::collect_state_flow_corpus(
        network,
        address,
        limit,
        HashMap::new(),
    ))?;
    write_json(&corpus, output, pretty, "State-flow corpus JSON")
}

fn reverse_infer_cmd(corpus: PathBuf, output: Option<PathBuf>, pretty: bool) -> anyhow::Result<()> {
    let json = fs::read_to_string(&corpus)
        .with_context(|| format!("failed to read {}", corpus.display()))?;
    let corpus: StateFlowCorpus = serde_json::from_str(&json)
        .with_context(|| format!("failed to parse {}", corpus.display()))?;
    let report = ton_stateflow::infer_schema_candidates(&corpus);
    write_json(&report, output, pretty, "State-flow schema report JSON")
}

fn reverse_replay_cmd(
    state_flow: PathBuf,
    flip_body_bit: Option<u16>,
    body_boc64: Option<String>,
    ignore_chksig: bool,
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    let json = fs::read_to_string(&state_flow)
        .with_context(|| format!("failed to read {}", state_flow.display()))?;
    let flow: StateFlowTx = serde_json::from_str(&json)
        .with_context(|| format!("failed to parse {}", state_flow.display()))?;
    let mutation = match (flip_body_bit, body_boc64) {
        (Some(bit), None) => ReplayMutation::FlipBodyBit { bit },
        (None, Some(body_boc64)) => ReplayMutation::ReplaceBody { body_boc64 },
        (None, None) => ReplayMutation::None,
        (Some(_), Some(_)) => anyhow::bail!("only one replay mutation can be selected"),
    };
    let diff = ton_stateflow::replay_state_flow_tx(&flow, mutation, ignore_chksig)?;
    write_json(&diff, output, pretty, "State-flow replay diff JSON")
}

fn reverse_report_cmd(
    corpus: PathBuf,
    schema: PathBuf,
    replay: Vec<PathBuf>,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    let corpus_json = fs::read_to_string(&corpus)
        .with_context(|| format!("failed to read {}", corpus.display()))?;
    let corpus: StateFlowCorpus = serde_json::from_str(&corpus_json)
        .with_context(|| format!("failed to parse {}", corpus.display()))?;
    let schema_json = fs::read_to_string(&schema)
        .with_context(|| format!("failed to read {}", schema.display()))?;
    let schema: StateFlowSchemaReport = serde_json::from_str(&schema_json)
        .with_context(|| format!("failed to parse {}", schema.display()))?;
    let replays = replay
        .iter()
        .map(|path| {
            let json = fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            serde_json::from_str::<StateFlowReplayDiff>(&json)
                .with_context(|| format!("failed to parse {}", path.display()))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let report = ton_stateflow::render_state_flow_report(&corpus, &schema, &replays);
    write_text(&report, output, "State-flow report")
}

fn reverse_smoke_cmd(
    targets: PathBuf,
    target_id: Option<&str>,
    out_dir: PathBuf,
    pretty: bool,
) -> anyhow::Result<()> {
    let manifest_json = fs::read_to_string(&targets)
        .with_context(|| format!("failed to read {}", targets.display()))?;
    let manifest = SmokeManifest::from_json(&manifest_json)
        .with_context(|| format!("failed to parse {}", targets.display()))?;
    let selected_targets = manifest.selected_targets(target_id)?;
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let mut target_summaries = Vec::new();

    for target in selected_targets {
        let network = Network::from_str(&target.network)
            .with_context(|| format!("invalid network for smoke target {}", target.id))?;
        let target_dir = out_dir.join(safe_path_segment(&target.id));
        fs::create_dir_all(&target_dir)
            .with_context(|| format!("failed to create {}", target_dir.display()))?;

        let corpus = rt.block_on(ton_stateflow::collect_state_flow_corpus(
            network,
            &target.address,
            target.collect_limit,
            HashMap::new(),
        ))?;
        let corpus_path = target_dir.join("corpus.json");
        write_json(
            &corpus,
            Some(corpus_path.clone()),
            pretty,
            "State-flow smoke corpus JSON",
        )?;

        let schema = ton_stateflow::infer_schema_candidates(&corpus);
        let schema_path = target_dir.join("schema.json");
        write_json(
            &schema,
            Some(schema_path.clone()),
            pretty,
            "State-flow smoke schema JSON",
        )?;

        let mut replays = Vec::new();
        let mut replay_path = None;
        let mut transaction_path = None;
        if let Some(plan) = &target.replay_mutation {
            let flow = corpus.transactions.first().with_context(|| {
                format!(
                    "smoke target {} produced no retraced transactions for replay",
                    target.id
                )
            })?;
            let tx_path = target_dir.join("transaction-0.json");
            write_json(
                flow,
                Some(tx_path.clone()),
                pretty,
                "State-flow smoke transaction JSON",
            )?;
            let replay = ton_stateflow::replay_state_flow_tx(
                flow,
                plan.to_replay_mutation()?,
                plan.ignore_chksig,
            )?;
            let path = target_dir.join("replay.json");
            write_json(
                &replay,
                Some(path.clone()),
                pretty,
                "State-flow smoke replay diff JSON",
            )?;
            transaction_path = Some(tx_path);
            replay_path = Some(path);
            replays.push(replay);
        }

        let report = ton_stateflow::render_state_flow_report(&corpus, &schema, &replays);
        let report_path = target_dir.join("report.md");
        write_text(
            &report,
            Some(report_path.clone()),
            "State-flow smoke report",
        )?;

        target_summaries.push(SmokeTargetRunSummary {
            id: target.id.clone(),
            network: target.network.clone(),
            address: target.address.clone(),
            source_url: target.source_url.clone(),
            collect_limit: target.collect_limit,
            source_tx_count: corpus.source_tx_count,
            retraced_count: corpus.retraced_count,
            failure_count: corpus.failure_count,
            opcode_candidate_count: schema.opcode_candidates.len(),
            state_edge_count: schema.state_machine.edges.len(),
            audit_signal_count: schema.audit_signals.len(),
            replay_count: replays.len(),
            output_dir: target_dir.display().to_string(),
            corpus: corpus_path.display().to_string(),
            schema: schema_path.display().to_string(),
            transaction: transaction_path.map(|path| path.display().to_string()),
            replay: replay_path.map(|path| path.display().to_string()),
            report: report_path.display().to_string(),
        });
    }

    let summary = SmokeRunSummary {
        schema_version: 1,
        target_count: target_summaries.len(),
        targets: target_summaries,
    };
    summary.ensure_passes_gate()?;
    write_json(
        &summary,
        Some(out_dir.join("summary.json")),
        pretty,
        "State-flow smoke summary JSON",
    )
}

fn write_state_flow(
    flow: &StateFlowTx,
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    write_json(flow, output, pretty, "State-flow JSON")
}

fn write_text(text: &str, output: Option<PathBuf>, label: &str) -> anyhow::Result<()> {
    if let Some(output) = output {
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&output, text)
            .with_context(|| format!("failed to write {}", output.display()))?;
        println!("{label} written to {}", output.display());
    } else {
        println!("{text}");
    }

    Ok(())
}

fn write_json<T: Serialize>(
    value: &T,
    output: Option<PathBuf>,
    pretty: bool,
    label: &str,
) -> anyhow::Result<()> {
    let json = if pretty {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };

    if let Some(output) = output {
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&output, json)
            .with_context(|| format!("failed to write {}", output.display()))?;
        println!("{label} written to {}", output.display());
    } else {
        println!("{json}");
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmokeManifest {
    schema_version: u32,
    targets: Vec<SmokeTarget>,
}

impl SmokeManifest {
    fn from_json(json: &str) -> anyhow::Result<Self> {
        let manifest: Self = serde_json::from_str(json)?;
        anyhow::ensure!(
            manifest.schema_version == 1,
            "unsupported smoke manifest schema version {}",
            manifest.schema_version
        );
        Ok(manifest)
    }

    fn selected_targets(&self, target_id: Option<&str>) -> anyhow::Result<Vec<&SmokeTarget>> {
        let targets: Vec<_> = self
            .targets
            .iter()
            .filter(|target| target_id.is_none_or(|target_id| target.id == target_id))
            .collect();
        if targets.is_empty() {
            if let Some(target_id) = target_id {
                anyhow::bail!("smoke target {target_id:?} was not found");
            }
            anyhow::bail!("smoke manifest does not contain any targets");
        }
        Ok(targets)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmokeTarget {
    id: String,
    network: String,
    address: String,
    source_url: Option<String>,
    collect_limit: u32,
    replay_mutation: Option<SmokeReplayMutation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmokeReplayMutation {
    #[serde(rename = "type")]
    mutation_type: String,
    bit: Option<u16>,
    body_boc64: Option<String>,
    #[serde(default)]
    ignore_chksig: bool,
}

impl SmokeReplayMutation {
    fn to_replay_mutation(&self) -> anyhow::Result<ReplayMutation> {
        match self.mutation_type.as_str() {
            "none" => Ok(ReplayMutation::None),
            "flipBodyBit" => Ok(ReplayMutation::FlipBodyBit {
                bit: self
                    .bit
                    .context("flipBodyBit replay mutation requires bit")?,
            }),
            "replaceBody" => Ok(ReplayMutation::ReplaceBody {
                body_boc64: self
                    .body_boc64
                    .clone()
                    .context("replaceBody replay mutation requires bodyBoc64")?,
            }),
            mutation_type => anyhow::bail!("unsupported replay mutation type {mutation_type:?}"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SmokeRunSummary {
    schema_version: u32,
    target_count: usize,
    targets: Vec<SmokeTargetRunSummary>,
}

impl SmokeRunSummary {
    fn ensure_passes_gate(&self) -> anyhow::Result<()> {
        for target in &self.targets {
            let mut failures = Vec::new();
            if target.source_tx_count == 0 {
                failures.push("source transactions 0".to_owned());
            }
            if target.retraced_count == 0 {
                failures.push("retraced transactions 0".to_owned());
            }
            if target.failure_count > 0 {
                failures.push(format!("collection failures {}", target.failure_count));
            }
            if target.opcode_candidate_count == 0 {
                failures.push("opcode candidates 0".to_owned());
            }
            if target.state_edge_count == 0 {
                failures.push("state edges 0".to_owned());
            }
            if target.audit_signal_count == 0 {
                failures.push("audit signals 0".to_owned());
            }
            if target.replay_count == 0 {
                failures.push("replays 0".to_owned());
            }

            if !failures.is_empty() {
                anyhow::bail!(
                    "smoke target {} failed quality gate: {}",
                    target.id,
                    failures.join(", ")
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SmokeTargetRunSummary {
    id: String,
    network: String,
    address: String,
    source_url: Option<String>,
    collect_limit: u32,
    source_tx_count: usize,
    retraced_count: usize,
    failure_count: usize,
    opcode_candidate_count: usize,
    state_edge_count: usize,
    audit_signal_count: usize,
    replay_count: usize,
    output_dir: String,
    corpus: String,
    schema: String,
    transaction: Option<String>,
    replay: Option<String>,
    report: String,
}

fn safe_path_segment(value: &str) -> String {
    let segment: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if segment.is_empty() {
        "target".to_owned()
    } else {
        segment
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_manifest_deserializes_checked_in_targets() {
        let manifest = super::SmokeManifest::from_json(include_str!(
            "../../../crates/ton-stateflow/smoke-targets.json"
        ))
        .expect("checked-in smoke targets should deserialize");

        assert!(manifest.targets.len() >= 2);
        assert!(manifest.targets.iter().any(|target| {
            target.id == "tonviewer-requested-target"
                && target.network == "mainnet"
                && target.address == "EQAgvOlWk7C0Pz3YgSaX-MA7UDDhE9n6eQgQRwJahOBm4VKr"
                && target
                    .replay_mutation
                    .as_ref()
                    .is_some_and(|plan| plan.ignore_chksig)
        }));
    }

    #[test]
    fn smoke_summary_gate_rejects_weak_artifacts() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].failure_count = 1;

        let err = summary.ensure_passes_gate().unwrap_err().to_string();

        assert!(err.contains("smoke target target-a failed quality gate"));
        assert!(err.contains("collection failures 1"));
    }

    #[test]
    fn smoke_summary_gate_requires_replay_and_state_edges() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].state_edge_count = 0;
        summary.targets[0].replay_count = 0;

        let err = summary.ensure_passes_gate().unwrap_err().to_string();

        assert!(err.contains("state edges 0"));
        assert!(err.contains("replays 0"));
    }

    #[test]
    fn smoke_summary_gate_accepts_full_artifacts() {
        let summary = sample_smoke_summary();

        summary.ensure_passes_gate().unwrap();
    }

    fn sample_smoke_summary() -> super::SmokeRunSummary {
        super::SmokeRunSummary {
            schema_version: 1,
            target_count: 1,
            targets: vec![super::SmokeTargetRunSummary {
                id: "target-a".to_owned(),
                network: "mainnet".to_owned(),
                address: "addr".to_owned(),
                source_url: None,
                collect_limit: 2,
                source_tx_count: 2,
                retraced_count: 2,
                failure_count: 0,
                opcode_candidate_count: 1,
                state_edge_count: 1,
                audit_signal_count: 1,
                replay_count: 1,
                output_dir: "out/target-a".to_owned(),
                corpus: "out/target-a/corpus.json".to_owned(),
                schema: "out/target-a/schema.json".to_owned(),
                transaction: Some("out/target-a/transaction-0.json".to_owned()),
                replay: Some("out/target-a/replay.json".to_owned()),
                report: "out/target-a/report.md".to_owned(),
            }],
        }
    }
}
