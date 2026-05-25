use anyhow::Context;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
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
        #[arg(
            required_unless_present = "artifact_manifest",
            conflicts_with = "artifact_manifest",
            help = "State-flow corpus JSON produced by `acton reverse collect`"
        )]
        corpus: Option<PathBuf>,
        #[arg(
            long,
            value_name = "ARTIFACTS",
            help = "State-flow artifact manifest produced by `acton reverse smoke`"
        )]
        artifact_manifest: Option<PathBuf>,
        #[arg(
            long,
            requires = "artifact_manifest",
            help = "Target id to select from an artifact manifest"
        )]
        target_id: Option<String>,
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
    #[command(about = "Replay or mutate a StateFlowTx or corpus artifact and emit a diff")]
    Replay {
        #[arg(
            required_unless_present = "artifact_manifest",
            conflicts_with = "artifact_manifest",
            help = "StateFlowTx JSON from `acton reverse retrace` or corpus JSON from `acton reverse collect`"
        )]
        state_flow: Option<PathBuf>,
        #[arg(
            long,
            value_name = "ARTIFACTS",
            help = "State-flow artifact manifest produced by `acton reverse smoke`"
        )]
        artifact_manifest: Option<PathBuf>,
        #[arg(
            long,
            requires = "artifact_manifest",
            help = "Target id to select from an artifact manifest"
        )]
        target_id: Option<String>,
        #[arg(
            long,
            help = "Transaction index to replay when the input is a corpus artifact"
        )]
        tx_index: Option<usize>,
        #[arg(
            long,
            help = "Transaction hash to replay when the input is a corpus artifact"
        )]
        tx_hash: Option<String>,
        #[arg(
            long,
            conflicts_with_all = ["body_boc64", "set_body_uint"],
            value_name = "FLIP_BODY_BIT",
            help = "Flip one inbound message body bit before replay"
        )]
        flip_body_bit: Option<u16>,
        #[arg(
            long,
            conflicts_with_all = ["flip_body_bit", "set_body_uint"],
            value_name = "BODY_BOC64",
            help = "Replace inbound message body with this base64 BoC before replay"
        )]
        body_boc64: Option<String>,
        #[arg(
            long,
            conflicts_with_all = ["flip_body_bit", "body_boc64"],
            value_name = "BIT_OFFSET:BITS:VALUE",
            help = "Set an unsigned integer field in the inbound body before replay"
        )]
        set_body_uint: Option<String>,
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
        #[arg(
            required_unless_present = "artifact_manifest",
            conflicts_with = "artifact_manifest",
            help = "State-flow corpus JSON produced by `acton reverse collect`"
        )]
        corpus: Option<PathBuf>,
        #[arg(
            long,
            required_unless_present = "artifact_manifest",
            conflicts_with = "artifact_manifest",
            help = "Schema candidate JSON produced by `acton reverse infer`"
        )]
        schema: Option<PathBuf>,
        #[arg(
            long,
            value_name = "REPLAY",
            conflicts_with = "artifact_manifest",
            help = "Replay diff JSON produced by `acton reverse replay`"
        )]
        replay: Vec<PathBuf>,
        #[arg(
            long,
            value_name = "ARTIFACTS",
            help = "State-flow artifact manifest produced by `acton reverse smoke`"
        )]
        artifact_manifest: Option<PathBuf>,
        #[arg(
            long,
            requires = "artifact_manifest",
            help = "Target id to select from an artifact manifest"
        )]
        target_id: Option<String>,
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write Markdown report to a file"
        )]
        output: Option<PathBuf>,
    },
    #[command(about = "Validate a state-flow artifact manifest bundle")]
    VerifyArtifacts {
        #[arg(
            value_name = "ARTIFACTS",
            help = "State-flow artifact manifest produced by `acton reverse smoke`"
        )]
        artifacts: PathBuf,
        #[arg(long, help = "Only validate artifacts for this target id")]
        target_id: Option<String>,
        #[arg(long, help = "Pretty-print JSON output")]
        pretty: bool,
    },
    #[command(about = "Run collect, infer, replay, and report for one target address")]
    Analyze {
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
            long,
            conflicts_with = "replay_tx_hash",
            help = "Corpus transaction index to replay"
        )]
        replay_tx_index: Option<usize>,
        #[arg(
            long,
            conflicts_with = "replay_tx_index",
            help = "Corpus transaction hash to replay"
        )]
        replay_tx_hash: Option<String>,
        #[arg(
            long,
            conflicts_with_all = ["body_boc64", "set_body_uint"],
            value_name = "FLIP_BODY_BIT",
            help = "Flip one inbound message body bit before replay"
        )]
        flip_body_bit: Option<u16>,
        #[arg(
            long,
            conflicts_with_all = ["flip_body_bit", "set_body_uint"],
            value_name = "BODY_BOC64",
            help = "Replace inbound message body with this base64 BoC before replay"
        )]
        body_boc64: Option<String>,
        #[arg(
            long,
            conflicts_with_all = ["flip_body_bit", "body_boc64"],
            value_name = "BIT_OFFSET:BITS:VALUE",
            help = "Set an unsigned integer field in the inbound body before replay"
        )]
        set_body_uint: Option<String>,
        #[arg(long, help = "Ignore TVM signature checks during local replay")]
        ignore_chksig: bool,
        #[arg(
            long,
            default_value = "target/stateflow-analysis",
            help = "Directory for analysis output artifacts"
        )]
        out_dir: PathBuf,
        #[arg(long, help = "Pretty-print JSON output artifacts")]
        pretty: bool,
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
            artifact_manifest,
            target_id,
            output,
            pretty,
        } => reverse_infer_cmd(corpus, artifact_manifest, target_id, output, pretty),
        ReverseCommand::Replay {
            state_flow,
            artifact_manifest,
            target_id,
            tx_index,
            tx_hash,
            flip_body_bit,
            body_boc64,
            set_body_uint,
            ignore_chksig,
            output,
            pretty,
        } => reverse_replay_cmd(
            state_flow,
            artifact_manifest,
            target_id,
            tx_index,
            tx_hash,
            flip_body_bit,
            body_boc64,
            set_body_uint,
            ignore_chksig,
            output,
            pretty,
        ),
        ReverseCommand::Report {
            corpus,
            schema,
            replay,
            artifact_manifest,
            target_id,
            output,
        } => reverse_report_cmd(corpus, schema, replay, artifact_manifest, target_id, output),
        ReverseCommand::VerifyArtifacts {
            artifacts,
            target_id,
            pretty,
        } => reverse_verify_artifacts_cmd(artifacts, target_id, pretty),
        ReverseCommand::Analyze {
            address,
            net,
            limit,
            replay_tx_index,
            replay_tx_hash,
            flip_body_bit,
            body_boc64,
            set_body_uint,
            ignore_chksig,
            out_dir,
            pretty,
        } => reverse_analyze_cmd(
            &address,
            &net,
            limit,
            replay_tx_index,
            replay_tx_hash,
            flip_body_bit,
            body_boc64,
            set_body_uint,
            ignore_chksig,
            out_dir,
            pretty,
        ),
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

fn reverse_infer_cmd(
    corpus: Option<PathBuf>,
    artifact_manifest: Option<PathBuf>,
    target_id: Option<String>,
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    let corpus = infer_corpus_input_path(corpus, artifact_manifest, target_id.as_deref())?;
    let json = fs::read_to_string(&corpus)
        .with_context(|| format!("failed to read {}", corpus.display()))?;
    let corpus: StateFlowCorpus = serde_json::from_str(&json)
        .with_context(|| format!("failed to parse {}", corpus.display()))?;
    let report = ton_stateflow::infer_schema_candidates(&corpus);
    write_json(&report, output, pretty, "State-flow schema report JSON")
}

fn infer_corpus_input_path(
    corpus: Option<PathBuf>,
    artifact_manifest: Option<PathBuf>,
    target_id: Option<&str>,
) -> anyhow::Result<PathBuf> {
    if let Some(manifest_path) = artifact_manifest {
        let manifest = load_artifact_manifest(&manifest_path)?;
        return infer_corpus_from_manifest(&manifest, &manifest_path, target_id);
    }

    corpus.context("state-flow corpus JSON is required")
}

fn reverse_replay_cmd(
    state_flow: Option<PathBuf>,
    artifact_manifest: Option<PathBuf>,
    target_id: Option<String>,
    tx_index: Option<usize>,
    tx_hash: Option<String>,
    flip_body_bit: Option<u16>,
    body_boc64: Option<String>,
    set_body_uint: Option<String>,
    ignore_chksig: bool,
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    let state_flow =
        replay_state_flow_input_path(state_flow, artifact_manifest, target_id.as_deref())?;
    let json = fs::read_to_string(&state_flow)
        .with_context(|| format!("failed to read {}", state_flow.display()))?;
    let flow = parse_replay_input(&json, &state_flow, tx_index, tx_hash.as_deref())?;
    let mutation = replay_mutation_from_args(flip_body_bit, body_boc64, set_body_uint)?;
    let diff = ton_stateflow::replay_state_flow_tx(&flow, mutation, ignore_chksig)?;
    write_json(&diff, output, pretty, "State-flow replay diff JSON")
}

fn replay_state_flow_input_path(
    state_flow: Option<PathBuf>,
    artifact_manifest: Option<PathBuf>,
    target_id: Option<&str>,
) -> anyhow::Result<PathBuf> {
    if let Some(manifest_path) = artifact_manifest {
        let manifest = load_artifact_manifest(&manifest_path)?;
        return replay_state_flow_from_manifest(&manifest, &manifest_path, target_id);
    }

    state_flow.context("StateFlowTx JSON or corpus JSON is required")
}

fn replay_mutation_from_args(
    flip_body_bit: Option<u16>,
    body_boc64: Option<String>,
    set_body_uint: Option<String>,
) -> anyhow::Result<ReplayMutation> {
    match (flip_body_bit, body_boc64, set_body_uint) {
        (Some(bit), None, None) => Ok(ReplayMutation::FlipBodyBit { bit }),
        (None, Some(body_boc64), None) => Ok(ReplayMutation::ReplaceBody { body_boc64 }),
        (None, None, Some(set_body_uint)) => {
            let (bit_offset, bits, value) = parse_set_body_uint_arg(&set_body_uint)?;
            Ok(ReplayMutation::SetBodyUint {
                bit_offset,
                bits,
                value,
            })
        }
        (None, None, None) => Ok(ReplayMutation::None),
        _ => anyhow::bail!("only one replay mutation can be selected"),
    }
}

fn parse_set_body_uint_arg(value: &str) -> anyhow::Result<(u16, u16, String)> {
    let mut parts = value.splitn(3, ':');
    let bit_offset = parts
        .next()
        .context("setBodyUint requires BIT_OFFSET:BITS:VALUE")?
        .parse::<u16>()
        .with_context(|| format!("invalid setBodyUint bit offset in {value:?}"))?;
    let bits = parts
        .next()
        .context("setBodyUint requires BIT_OFFSET:BITS:VALUE")?
        .parse::<u16>()
        .with_context(|| format!("invalid setBodyUint bit length in {value:?}"))?;
    let value = parts
        .next()
        .filter(|value| !value.is_empty())
        .context("setBodyUint requires BIT_OFFSET:BITS:VALUE")?
        .to_owned();
    Ok((bit_offset, bits, value))
}

fn parse_replay_input(
    json: &str,
    path: &Path,
    tx_index: Option<usize>,
    tx_hash: Option<&str>,
) -> anyhow::Result<StateFlowTx> {
    if tx_index.is_some() && tx_hash.is_some() {
        anyhow::bail!("only one corpus transaction selector can be provided");
    }

    let value: serde_json::Value = serde_json::from_str(json)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    if is_corpus_json(&value) {
        let corpus: StateFlowCorpus = serde_json::from_value(value)
            .with_context(|| format!("failed to parse corpus {}", path.display()))?;
        if let Some(tx_hash) = tx_hash {
            return corpus
                .transactions
                .into_iter()
                .find(|flow| flow.query_hash == tx_hash)
                .with_context(|| {
                    format!(
                        "corpus transaction hash {tx_hash:?} was not found in {}",
                        path.display()
                    )
                });
        }

        let tx_index = tx_index.unwrap_or(0);
        let len = corpus.transactions.len();
        return corpus
            .transactions
            .into_iter()
            .nth(tx_index)
            .with_context(|| {
                format!(
                    "corpus transaction index {tx_index} out of range for {} transaction(s) in {}",
                    len,
                    path.display()
                )
            });
    }

    if tx_index.is_some() || tx_hash.is_some() {
        anyhow::bail!("corpus transaction selectors require a corpus replay input");
    }
    serde_json::from_value::<StateFlowTx>(value)
        .with_context(|| format!("failed to parse StateFlowTx {}", path.display()))
}

fn is_corpus_json(value: &serde_json::Value) -> bool {
    value
        .get("transactions")
        .and_then(serde_json::Value::as_array)
        .is_some()
        && value.get("opcodeSummary").is_some()
}

fn reverse_report_cmd(
    corpus: Option<PathBuf>,
    schema: Option<PathBuf>,
    replay: Vec<PathBuf>,
    artifact_manifest: Option<PathBuf>,
    target_id: Option<String>,
    output: Option<PathBuf>,
) -> anyhow::Result<()> {
    let report_artifacts = if let Some(manifest_path) = artifact_manifest {
        let manifest = load_artifact_manifest(&manifest_path)?;
        report_artifacts_from_manifest(&manifest, &manifest_path, target_id.as_deref())?
    } else {
        ReportArtifactInputs {
            corpus: corpus.context("state-flow corpus JSON is required")?,
            schema: schema.context("schema candidate JSON is required")?,
            replays: replay,
        }
    };

    let corpus_json = fs::read_to_string(&report_artifacts.corpus)
        .with_context(|| format!("failed to read {}", report_artifacts.corpus.display()))?;
    let corpus: StateFlowCorpus = serde_json::from_str(&corpus_json)
        .with_context(|| format!("failed to parse {}", report_artifacts.corpus.display()))?;
    let schema_json = fs::read_to_string(&report_artifacts.schema)
        .with_context(|| format!("failed to read {}", report_artifacts.schema.display()))?;
    let schema: StateFlowSchemaReport = serde_json::from_str(&schema_json)
        .with_context(|| format!("failed to parse {}", report_artifacts.schema.display()))?;
    let replays = report_artifacts
        .replays
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

fn reverse_verify_artifacts_cmd(
    artifacts: PathBuf,
    target_id: Option<String>,
    pretty: bool,
) -> anyhow::Result<()> {
    let manifest = load_artifact_manifest(&artifacts)?;
    let validation =
        validate_artifact_manifest_bundle(&manifest, &artifacts, target_id.as_deref())?;
    write_json(
        &validation,
        None,
        pretty,
        "State-flow artifact manifest validation JSON",
    )?;
    anyhow::ensure!(
        validation.passed,
        "artifact manifest validation failed: {}",
        validation.gate_failures.join("; ")
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReportArtifactInputs {
    corpus: PathBuf,
    schema: PathBuf,
    replays: Vec<PathBuf>,
}

fn reverse_analyze_cmd(
    address: &str,
    net: &str,
    limit: u32,
    replay_tx_index: Option<usize>,
    replay_tx_hash: Option<String>,
    flip_body_bit: Option<u16>,
    body_boc64: Option<String>,
    set_body_uint: Option<String>,
    ignore_chksig: bool,
    out_dir: PathBuf,
    pretty: bool,
) -> anyhow::Result<()> {
    let target = analysis_target_from_args(
        address,
        net,
        limit,
        replay_tx_index,
        replay_tx_hash,
        flip_body_bit,
        body_boc64,
        set_body_uint,
        ignore_chksig,
    )?;
    run_state_flow_targets(vec![&target], out_dir, pretty)
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
    run_state_flow_targets(selected_targets, out_dir, pretty)
}

fn run_state_flow_targets(
    selected_targets: Vec<&SmokeTarget>,
    out_dir: PathBuf,
    pretty: bool,
) -> anyhow::Result<()> {
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
        let mut replay_paths = Vec::new();
        let mut replay_path = None;
        let mut transaction_path = None;
        if let Some(plan) = &target.replay_mutation {
            let (flow_index, flow) = select_corpus_transaction_ref(
                &corpus,
                target.replay_tx_index,
                target.replay_tx_hash.as_deref(),
                &format!("smoke target {}", target.id),
            )?;
            let tx_path = target_dir.join(format!("transaction-{flow_index}.json"));
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
            replay_path = Some(path.clone());
            replay_paths.push(path);
            replays.push(replay);

            let probe_replays = run_schema_replay_probes(
                &corpus,
                &schema,
                &target_dir,
                pretty,
                plan.ignore_chksig,
            )?;
            for (path, replay) in probe_replays {
                replay_paths.push(path);
                replays.push(replay);
            }
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
            passed: false,
            gate_failures: Vec::new(),
            output_dir: target_dir.display().to_string(),
            corpus: corpus_path.display().to_string(),
            schema: schema_path.display().to_string(),
            transaction: transaction_path.map(|path| path.display().to_string()),
            replay: replay_path.map(|path| path.display().to_string()),
            replays: replay_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            report: report_path.display().to_string(),
        });
    }

    let summary = SmokeRunSummary::from_targets(target_summaries);
    write_smoke_summary_and_gate(&summary, &out_dir, pretty)
}

fn run_schema_replay_probes(
    corpus: &StateFlowCorpus,
    schema: &StateFlowSchemaReport,
    target_dir: &Path,
    pretty: bool,
    ignore_chksig: bool,
) -> anyhow::Result<Vec<(PathBuf, StateFlowReplayDiff)>> {
    let mut replays = Vec::new();
    let mut used_paths = HashSet::new();

    for candidate in &schema.opcode_candidates {
        for probe in &candidate.replay_probes {
            let Some(tx_hash) = probe.evidence.first() else {
                continue;
            };
            let (_, flow) =
                select_corpus_transaction_ref(corpus, None, Some(tx_hash), "schema replay probe")?;
            let replay =
                ton_stateflow::replay_state_flow_tx(flow, probe.mutation.clone(), ignore_chksig)?;
            let path = unique_replay_probe_path(target_dir, probe, &mut used_paths);
            write_json(
                &replay,
                Some(path.clone()),
                pretty,
                "State-flow smoke replay probe diff JSON",
            )?;
            replays.push((path, replay));
        }
    }

    Ok(replays)
}

fn unique_replay_probe_path(
    target_dir: &Path,
    probe: &ton_stateflow::ReplayProbeCandidate,
    used_paths: &mut HashSet<String>,
) -> PathBuf {
    let base = safe_path_segment(&format!(
        "replay-probe-{}-{}-{}",
        probe.field_name, probe.bit_offset, probe.bits
    ));
    let mut name = format!("{base}.json");
    let mut index = 2;
    while !used_paths.insert(name.clone()) {
        name = format!("{base}-{index}.json");
        index += 1;
    }
    target_dir.join(name)
}

fn write_smoke_summary_and_gate(
    summary: &SmokeRunSummary,
    out_dir: &Path,
    pretty: bool,
) -> anyhow::Result<()> {
    let portable_summary = summary.with_paths_relative_to(out_dir);
    let summary_path = out_dir.join("summary.json");
    write_json(
        &portable_summary,
        Some(summary_path.clone()),
        pretty,
        "State-flow smoke summary JSON",
    )?;
    let manifest = SmokeArtifactManifest::from_summary(&portable_summary, out_dir);
    write_json(
        &manifest,
        Some(out_dir.join("artifacts.json")),
        pretty,
        "State-flow artifact manifest JSON",
    )?;
    summary.ensure_passes_gate()
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
    #[serde(default)]
    replay_tx_index: Option<usize>,
    #[serde(default)]
    replay_tx_hash: Option<String>,
    replay_mutation: Option<SmokeReplayMutation>,
}

fn analysis_target_from_args(
    address: &str,
    net: &str,
    limit: u32,
    replay_tx_index: Option<usize>,
    replay_tx_hash: Option<String>,
    flip_body_bit: Option<u16>,
    body_boc64: Option<String>,
    set_body_uint: Option<String>,
    ignore_chksig: bool,
) -> anyhow::Result<SmokeTarget> {
    if replay_tx_index.is_some() && replay_tx_hash.is_some() {
        anyhow::bail!("only one replay transaction selector can be provided");
    }

    Ok(SmokeTarget {
        id: "analysis".to_owned(),
        network: net.to_owned(),
        address: address.to_owned(),
        source_url: None,
        collect_limit: limit,
        replay_tx_index,
        replay_tx_hash,
        replay_mutation: Some(SmokeReplayMutation::from_args(
            flip_body_bit,
            body_boc64,
            set_body_uint,
            ignore_chksig,
        )?),
    })
}

fn load_artifact_manifest(manifest_path: &Path) -> anyhow::Result<SmokeArtifactManifest> {
    let json = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    serde_json::from_str(&json)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmokeReplayMutation {
    #[serde(rename = "type")]
    mutation_type: String,
    bit: Option<u16>,
    body_boc64: Option<String>,
    bit_offset: Option<u16>,
    bits: Option<u16>,
    value: Option<String>,
    #[serde(default)]
    ignore_chksig: bool,
}

impl SmokeReplayMutation {
    fn from_args(
        flip_body_bit: Option<u16>,
        body_boc64: Option<String>,
        set_body_uint: Option<String>,
        ignore_chksig: bool,
    ) -> anyhow::Result<Self> {
        match (flip_body_bit, body_boc64, set_body_uint) {
            (Some(bit), None, None) => Ok(Self {
                mutation_type: "flipBodyBit".to_owned(),
                bit: Some(bit),
                body_boc64: None,
                bit_offset: None,
                bits: None,
                value: None,
                ignore_chksig,
            }),
            (None, Some(body_boc64), None) => Ok(Self {
                mutation_type: "replaceBody".to_owned(),
                bit: None,
                body_boc64: Some(body_boc64),
                bit_offset: None,
                bits: None,
                value: None,
                ignore_chksig,
            }),
            (None, None, Some(set_body_uint)) => {
                let (bit_offset, bits, value) = parse_set_body_uint_arg(&set_body_uint)?;
                Ok(Self {
                    mutation_type: "setBodyUint".to_owned(),
                    bit: None,
                    body_boc64: None,
                    bit_offset: Some(bit_offset),
                    bits: Some(bits),
                    value: Some(value),
                    ignore_chksig,
                })
            }
            (None, None, None) => Ok(Self {
                mutation_type: "none".to_owned(),
                bit: None,
                body_boc64: None,
                bit_offset: None,
                bits: None,
                value: None,
                ignore_chksig,
            }),
            _ => anyhow::bail!("only one replay mutation can be selected"),
        }
    }

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
            "setBodyUint" => Ok(ReplayMutation::SetBodyUint {
                bit_offset: self
                    .bit_offset
                    .context("setBodyUint replay mutation requires bitOffset")?,
                bits: self
                    .bits
                    .context("setBodyUint replay mutation requires bits")?,
                value: self
                    .value
                    .clone()
                    .context("setBodyUint replay mutation requires value")?,
            }),
            mutation_type => anyhow::bail!("unsupported replay mutation type {mutation_type:?}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SmokeRunSummary {
    schema_version: u32,
    target_count: usize,
    passed: bool,
    absolute_path_count: usize,
    gate_failures: Vec<String>,
    targets: Vec<SmokeTargetRunSummary>,
}

impl SmokeRunSummary {
    fn from_targets(targets: Vec<SmokeTargetRunSummary>) -> Self {
        let mut summary = Self {
            schema_version: 1,
            target_count: targets.len(),
            passed: false,
            absolute_path_count: 0,
            gate_failures: Vec::new(),
            targets,
        };
        summary.refresh_gate_status();
        summary
    }

    fn with_paths_relative_to(&self, artifact_dir: &Path) -> Self {
        let mut summary = self.clone();
        for target in &mut summary.targets {
            target.rewrite_paths_relative_to(artifact_dir);
        }
        summary.refresh_gate_status();
        summary
    }

    fn refresh_gate_status(&mut self) {
        self.target_count = self.targets.len();
        self.absolute_path_count = self
            .targets
            .iter()
            .map(smoke_target_absolute_path_count)
            .sum();
        for target in &mut self.targets {
            target.refresh_gate_status();
        }
        self.gate_failures = self
            .targets
            .iter()
            .flat_map(|target| {
                target
                    .gate_failures
                    .iter()
                    .map(|failure| format!("{}: {failure}", target.id))
            })
            .collect();
        self.passed = self.gate_failures.is_empty();
    }

    fn ensure_passes_gate(&self) -> anyhow::Result<()> {
        let mut failure_messages = Vec::new();
        for target in &self.targets {
            let failures = target.quality_gate_failures();
            if !failures.is_empty() {
                let target_failures = failures
                    .iter()
                    .map(|failure| format!("{}: {failure}", target.id))
                    .collect::<Vec<_>>()
                    .join(", ");
                failure_messages.push(format!(
                    "smoke target {} failed quality gate: {}",
                    target.id, target_failures
                ));
            }
        }

        if !failure_messages.is_empty() {
            anyhow::bail!("smoke quality gate failed: {}", failure_messages.join("; "));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
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
    passed: bool,
    gate_failures: Vec<String>,
    output_dir: String,
    corpus: String,
    schema: String,
    transaction: Option<String>,
    replay: Option<String>,
    replays: Vec<String>,
    report: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmokeArtifactManifest {
    schema_version: u32,
    kind: String,
    summary: String,
    target_count: usize,
    #[serde(default)]
    absolute_path_count: usize,
    artifacts: Vec<SmokeArtifactManifestEntry>,
}

impl SmokeArtifactManifest {
    fn from_summary(summary: &SmokeRunSummary, artifact_dir: &Path) -> Self {
        let summary_path = artifact_dir.join("summary.json");
        let mut artifacts = vec![SmokeArtifactManifestEntry::new(
            "runSummary",
            manifest_relative_path(&summary_path, artifact_dir),
            None,
        )];

        for target in &summary.targets {
            artifacts.push(SmokeArtifactManifestEntry::new(
                "corpus",
                manifest_relative_path(Path::new(&target.corpus), artifact_dir),
                Some(target.id.clone()),
            ));
            artifacts.push(SmokeArtifactManifestEntry::new(
                "schema",
                manifest_relative_path(Path::new(&target.schema), artifact_dir),
                Some(target.id.clone()),
            ));
            if let Some(transaction) = &target.transaction {
                artifacts.push(SmokeArtifactManifestEntry::new(
                    "transaction",
                    manifest_relative_path(Path::new(transaction), artifact_dir),
                    Some(target.id.clone()),
                ));
            }
            for replay in replay_artifact_paths(target) {
                artifacts.push(SmokeArtifactManifestEntry::new(
                    "replay",
                    manifest_relative_path(Path::new(&replay), artifact_dir),
                    Some(target.id.clone()),
                ));
            }
            artifacts.push(SmokeArtifactManifestEntry::new(
                "report",
                manifest_relative_path(Path::new(&target.report), artifact_dir),
                Some(target.id.clone()),
            ));
        }

        let summary_artifact_path = manifest_relative_path(&summary_path, artifact_dir);
        let absolute_path_count =
            smoke_manifest_absolute_path_count(&summary_artifact_path, &artifacts);

        Self {
            schema_version: 1,
            kind: "stateFlowArtifactManifest".to_owned(),
            summary: summary_artifact_path,
            target_count: summary.target_count,
            absolute_path_count,
            artifacts,
        }
    }
}

fn smoke_target_absolute_path_count(target: &SmokeTargetRunSummary) -> usize {
    [
        Some(target.output_dir.as_str()),
        Some(target.corpus.as_str()),
        Some(target.schema.as_str()),
        target.transaction.as_deref(),
        target.replay.as_deref(),
        Some(target.report.as_str()),
    ]
    .into_iter()
    .flatten()
    .filter(|path| Path::new(path).is_absolute())
    .count()
        + target
            .replays
            .iter()
            .filter(|path| Path::new(path).is_absolute())
            .count()
}

fn smoke_manifest_absolute_path_count(
    summary: &str,
    artifacts: &[SmokeArtifactManifestEntry],
) -> usize {
    usize::from(Path::new(summary).is_absolute())
        + artifacts
            .iter()
            .filter(|artifact| Path::new(&artifact.path).is_absolute())
            .count()
}

fn manifest_relative_path(path: &Path, artifact_dir: &Path) -> String {
    if path.is_absolute() != artifact_dir.is_absolute() {
        return path.display().to_string();
    }
    pathdiff::diff_paths(path, artifact_dir)
        .unwrap_or_else(|| path.to_path_buf())
        .display()
        .to_string()
}

fn report_artifacts_from_manifest(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: Option<&str>,
) -> anyhow::Result<ReportArtifactInputs> {
    ensure_supported_artifact_manifest(manifest, manifest_path)?;
    let selected_target_id = select_manifest_target_id(manifest, manifest_path, target_id)?;

    Ok(ReportArtifactInputs {
        corpus: required_manifest_artifact_path(
            manifest,
            manifest_path,
            &selected_target_id,
            "corpus",
        )?,
        schema: required_manifest_artifact_path(
            manifest,
            manifest_path,
            &selected_target_id,
            "schema",
        )?,
        replays: manifest_artifact_paths(manifest, manifest_path, &selected_target_id, "replay"),
    })
}

fn replay_state_flow_from_manifest(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: Option<&str>,
) -> anyhow::Result<PathBuf> {
    infer_corpus_from_manifest(manifest, manifest_path, target_id)
}

fn infer_corpus_from_manifest(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: Option<&str>,
) -> anyhow::Result<PathBuf> {
    ensure_supported_artifact_manifest(manifest, manifest_path)?;
    let selected_target_id = select_manifest_target_id(manifest, manifest_path, target_id)?;
    required_manifest_artifact_path(manifest, manifest_path, &selected_target_id, "corpus")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactManifestValidation {
    schema_version: u32,
    kind: String,
    manifest: String,
    target_count: usize,
    absolute_path_count: usize,
    expected_absolute_path_count: usize,
    passed: bool,
    gate_failures: Vec<String>,
    targets: Vec<ArtifactManifestTargetValidation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactManifestTargetValidation {
    id: String,
    artifact_count: usize,
    replay_count: usize,
    passed: bool,
    gate_failures: Vec<String>,
}

fn validate_artifact_manifest_bundle(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: Option<&str>,
) -> anyhow::Result<ArtifactManifestValidation> {
    ensure_supported_artifact_manifest(manifest, manifest_path)?;
    let target_ids = manifest_target_ids(manifest);
    let selected_target_ids = if let Some(target_id) = target_id {
        anyhow::ensure!(
            target_ids.iter().any(|id| id == target_id),
            "artifact manifest {} does not contain target id {:?}",
            manifest_path.display(),
            target_id
        );
        vec![target_id.to_owned()]
    } else {
        target_ids.clone()
    };

    let expected_absolute_path_count =
        smoke_manifest_absolute_path_count(&manifest.summary, &manifest.artifacts);
    let mut gate_failures = Vec::new();
    if manifest.absolute_path_count != expected_absolute_path_count {
        gate_failures.push(format!(
            "absolute path count mismatch: manifest {}, actual {}",
            manifest.absolute_path_count, expected_absolute_path_count
        ));
    }
    if manifest.target_count != target_ids.len() {
        gate_failures.push(format!(
            "target count mismatch: manifest {}, actual {}",
            manifest.target_count,
            target_ids.len()
        ));
    }

    let summary_path = resolve_manifest_artifact_path(manifest_path, &manifest.summary);
    if !summary_path.exists() {
        gate_failures.push(format!("missing summary artifact {}", manifest.summary));
    }
    for artifact in &manifest.artifacts {
        let path = resolve_manifest_artifact_path(manifest_path, &artifact.path);
        if !path.exists() {
            gate_failures.push(format!("missing artifact {}", artifact.path));
        }
    }

    let targets = selected_target_ids
        .iter()
        .map(|target_id| validate_artifact_manifest_target(manifest, target_id))
        .collect::<Vec<_>>();
    gate_failures.extend(targets.iter().flat_map(|target| {
        target
            .gate_failures
            .iter()
            .map(|failure| format!("{}: {failure}", target.id))
    }));

    Ok(ArtifactManifestValidation {
        schema_version: 1,
        kind: "stateFlowArtifactManifestValidation".to_owned(),
        manifest: manifest_path.display().to_string(),
        target_count: selected_target_ids.len(),
        absolute_path_count: manifest.absolute_path_count,
        expected_absolute_path_count,
        passed: gate_failures.is_empty(),
        gate_failures,
        targets,
    })
}

fn validate_artifact_manifest_target(
    manifest: &SmokeArtifactManifest,
    target_id: &str,
) -> ArtifactManifestTargetValidation {
    let artifacts = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.target_id.as_deref() == Some(target_id))
        .collect::<Vec<_>>();
    let mut gate_failures = Vec::new();
    for kind in ["corpus", "schema", "replay", "report"] {
        if !artifacts.iter().any(|artifact| artifact.kind == kind) {
            gate_failures.push(format!("missing {kind} artifact"));
        }
    }

    ArtifactManifestTargetValidation {
        id: target_id.to_owned(),
        artifact_count: artifacts.len(),
        replay_count: artifacts
            .iter()
            .filter(|artifact| artifact.kind == "replay")
            .count(),
        passed: gate_failures.is_empty(),
        gate_failures,
    }
}

fn ensure_supported_artifact_manifest(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        manifest.kind == "stateFlowArtifactManifest",
        "unsupported artifact manifest kind {} in {}",
        manifest.kind,
        manifest_path.display()
    );
    anyhow::ensure!(
        manifest.schema_version == 1,
        "unsupported artifact manifest schema version {} in {}",
        manifest.schema_version,
        manifest_path.display()
    );

    Ok(())
}

fn select_manifest_target_id(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: Option<&str>,
) -> anyhow::Result<String> {
    let target_ids = manifest_target_ids(manifest);
    let selected_target_id = match target_id {
        Some(target_id) => target_id.to_owned(),
        None if target_ids.len() == 1 => target_ids[0].clone(),
        None => {
            anyhow::bail!(
                "artifact manifest {} contains {} target(s); pass --target-id",
                manifest_path.display(),
                target_ids.len()
            )
        }
    };

    anyhow::ensure!(
        target_ids.iter().any(|id| id == &selected_target_id),
        "artifact manifest {} does not contain target id {:?}",
        manifest_path.display(),
        selected_target_id
    );

    Ok(selected_target_id)
}

fn manifest_target_ids(manifest: &SmokeArtifactManifest) -> Vec<String> {
    let mut target_ids = Vec::new();
    for artifact in &manifest.artifacts {
        let Some(target_id) = &artifact.target_id else {
            continue;
        };
        if !target_ids.iter().any(|known| known == target_id) {
            target_ids.push(target_id.clone());
        }
    }
    target_ids
}

fn required_manifest_artifact_path(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: &str,
    kind: &str,
) -> anyhow::Result<PathBuf> {
    let paths = manifest_artifact_paths(manifest, manifest_path, target_id, kind);
    match paths.as_slice() {
        [path] => Ok(path.clone()),
        [] => anyhow::bail!("artifact manifest target {target_id:?} is missing {kind} artifact"),
        _ => anyhow::bail!("artifact manifest target {target_id:?} has multiple {kind} artifacts"),
    }
}

fn manifest_artifact_paths(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: &str,
    kind: &str,
) -> Vec<PathBuf> {
    manifest
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.kind == kind && artifact.target_id.as_deref() == Some(target_id)
        })
        .map(|artifact| resolve_manifest_artifact_path(manifest_path, &artifact.path))
        .collect()
}

fn resolve_manifest_artifact_path(manifest_path: &Path, artifact_path: &str) -> PathBuf {
    let path = PathBuf::from(artifact_path);
    if path.is_absolute() {
        return path;
    }
    let Some(manifest_dir) = manifest_path.parent() else {
        return path;
    };
    let joined = manifest_dir.join(&path);
    if joined.exists() || !path.exists() {
        joined
    } else {
        path
    }
}

fn replay_artifact_paths(target: &SmokeTargetRunSummary) -> Vec<String> {
    if !target.replays.is_empty() {
        return target.replays.clone();
    }
    target.replay.iter().cloned().collect()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmokeArtifactManifestEntry {
    kind: String,
    path: String,
    target_id: Option<String>,
}

impl SmokeArtifactManifestEntry {
    fn new(kind: impl Into<String>, path: impl Into<String>, target_id: Option<String>) -> Self {
        Self {
            kind: kind.into(),
            path: path.into(),
            target_id,
        }
    }
}

impl SmokeTargetRunSummary {
    fn rewrite_paths_relative_to(&mut self, artifact_dir: &Path) {
        self.output_dir = manifest_relative_path(Path::new(&self.output_dir), artifact_dir);
        self.corpus = manifest_relative_path(Path::new(&self.corpus), artifact_dir);
        self.schema = manifest_relative_path(Path::new(&self.schema), artifact_dir);
        self.transaction = self
            .transaction
            .as_deref()
            .map(|path| manifest_relative_path(Path::new(path), artifact_dir));
        self.replay = self
            .replay
            .as_deref()
            .map(|path| manifest_relative_path(Path::new(path), artifact_dir));
        self.replays = self
            .replays
            .iter()
            .map(|path| manifest_relative_path(Path::new(path), artifact_dir))
            .collect();
        self.report = manifest_relative_path(Path::new(&self.report), artifact_dir);
    }

    fn refresh_gate_status(&mut self) {
        self.gate_failures = self.quality_gate_failures();
        self.passed = self.gate_failures.is_empty();
    }

    fn quality_gate_failures(&self) -> Vec<String> {
        let mut failures = Vec::new();
        if self.source_tx_count == 0 {
            failures.push("source transactions 0".to_owned());
        }
        if self.retraced_count == 0 {
            failures.push("retraced transactions 0".to_owned());
        }
        if self.failure_count > 0 {
            failures.push(format!("collection failures {}", self.failure_count));
        }
        if self.opcode_candidate_count == 0 {
            failures.push("opcode candidates 0".to_owned());
        }
        if self.state_edge_count == 0 {
            failures.push("state edges 0".to_owned());
        }
        if self.audit_signal_count == 0 {
            failures.push("audit signals 0".to_owned());
        }
        if self.replay_count == 0 {
            failures.push("replays 0".to_owned());
        }
        failures
    }
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

fn select_corpus_transaction_ref<'a>(
    corpus: &'a StateFlowCorpus,
    tx_index: Option<usize>,
    tx_hash: Option<&str>,
    label: &str,
) -> anyhow::Result<(usize, &'a StateFlowTx)> {
    if tx_index.is_some() && tx_hash.is_some() {
        anyhow::bail!("only one corpus transaction selector can be provided");
    }
    if let Some(tx_hash) = tx_hash {
        return corpus
            .transactions
            .iter()
            .enumerate()
            .find(|(_, flow)| flow.query_hash == tx_hash)
            .with_context(|| format!("{label} transaction hash {tx_hash:?} was not found"));
    }

    let tx_index = tx_index.unwrap_or(0);
    corpus
        .transactions
        .get(tx_index)
        .map(|flow| (tx_index, flow))
        .with_context(|| {
            format!(
                "{label} transaction index {tx_index} out of range for {} transaction(s)",
                corpus.transactions.len()
            )
        })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use ton_stateflow::ReplayMutation;

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
        summary.refresh_gate_status();

        let err = summary.ensure_passes_gate().unwrap_err().to_string();

        assert!(err.contains("smoke target target-a failed quality gate"));
        assert!(err.contains("collection failures 1"));
    }

    #[test]
    fn smoke_summary_gate_requires_replay_and_state_edges() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].state_edge_count = 0;
        summary.targets[0].replay_count = 0;
        summary.refresh_gate_status();

        let err = summary.ensure_passes_gate().unwrap_err().to_string();

        assert!(err.contains("state edges 0"));
        assert!(err.contains("replays 0"));
    }

    #[test]
    fn smoke_summary_gate_accepts_full_artifacts() {
        let summary = sample_smoke_summary();

        summary.ensure_passes_gate().unwrap();
    }

    #[test]
    fn smoke_summary_serializes_gate_status_for_full_artifacts() {
        let summary = sample_smoke_summary();
        let json = serde_json::to_value(&summary).expect("summary should serialize");

        assert_eq!(json["passed"], true);
        assert_eq!(json["absolutePathCount"], 0);
        assert_eq!(json["gateFailures"], serde_json::json!([]));
        assert_eq!(json["targets"][0]["passed"], true);
        assert_eq!(json["targets"][0]["gateFailures"], serde_json::json!([]));
        assert_eq!(
            json["targets"][0]["replays"],
            serde_json::json!(["out/target-a/replay.json"])
        );
    }

    #[test]
    fn smoke_summary_records_gate_failures_for_machine_consumers() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].failure_count = 1;
        summary.targets[0].state_edge_count = 0;
        summary.refresh_gate_status();
        let json = serde_json::to_value(&summary).expect("summary should serialize");

        assert_eq!(json["passed"], false);
        assert_eq!(json["targets"][0]["passed"], false);
        assert_eq!(
            json["targets"][0]["gateFailures"],
            serde_json::json!(["collection failures 1", "state edges 0"])
        );
        assert_eq!(
            json["gateFailures"],
            serde_json::json!(["target-a: collection failures 1", "target-a: state edges 0"])
        );
    }

    #[test]
    fn smoke_summary_gate_error_reports_all_targets() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].failure_count = 1;
        let mut second = sample_smoke_summary().targets.remove(0);
        second.id = "target-b".to_owned();
        second.replay_count = 0;
        summary.targets.push(second);
        summary.target_count = summary.targets.len();
        summary.refresh_gate_status();

        let err = summary.ensure_passes_gate().unwrap_err().to_string();

        assert!(err.contains("target-a: collection failures 1"));
        assert!(err.contains("target-b: replays 0"));
    }

    #[test]
    fn smoke_summary_is_written_before_gate_error() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].failure_count = 1;
        summary.refresh_gate_status();
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");

        let err = super::write_smoke_summary_and_gate(&summary, temp_dir.path(), true)
            .unwrap_err()
            .to_string();

        assert!(err.contains("target-a: collection failures 1"));
        let written = fs::read_to_string(temp_dir.path().join("summary.json"))
            .expect("summary should be written before gate error");
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("summary should be valid JSON");
        assert_eq!(json["passed"], false);
        assert_eq!(
            json["gateFailures"],
            serde_json::json!(["target-a: collection failures 1"])
        );
    }

    #[test]
    fn smoke_summary_paths_are_written_relative_to_artifact_dir() {
        let mut summary = sample_smoke_summary();
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        rewrite_sample_summary_paths(&mut summary, temp_dir.path());

        super::write_smoke_summary_and_gate(&summary, temp_dir.path(), true)
            .expect("portable summary should pass gate");

        let written = fs::read_to_string(temp_dir.path().join("summary.json"))
            .expect("summary should be written");
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("summary should be valid JSON");
        assert_eq!(json["targets"][0]["outputDir"], "target-a");
        assert_eq!(json["targets"][0]["corpus"], "target-a/corpus.json");
        assert_eq!(json["targets"][0]["schema"], "target-a/schema.json");
        assert_eq!(
            json["targets"][0]["transaction"],
            "target-a/transaction-0.json"
        );
        assert_eq!(json["targets"][0]["replay"], "target-a/replay.json");
        assert_eq!(
            json["targets"][0]["replays"],
            serde_json::json!(["target-a/replay.json"])
        );
        assert_eq!(json["targets"][0]["report"], "target-a/report.md");
        assert_eq!(json["absolutePathCount"], 0);
    }

    #[test]
    fn smoke_artifact_manifest_indexes_target_outputs() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].replay_count = 2;
        summary.targets[0].replays = vec![
            "out/target-a/replay.json".to_owned(),
            "out/target-a/replay-query-id.json".to_owned(),
        ];

        let manifest = super::SmokeArtifactManifest::from_summary(&summary, Path::new("out"));
        let json = serde_json::to_value(&manifest).expect("manifest should serialize");

        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["kind"], "stateFlowArtifactManifest");
        assert_eq!(json["summary"], "summary.json");
        assert_eq!(json["targetCount"], 1);
        assert_eq!(json["absolutePathCount"], 0);
        assert_eq!(json["artifacts"].as_array().unwrap().len(), 7);
        assert_eq!(
            json["artifacts"],
            serde_json::json!([
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "transaction", "path": "target-a/transaction-0.json", "targetId": "target-a"},
                {"kind": "replay", "path": "target-a/replay.json", "targetId": "target-a"},
                {"kind": "replay", "path": "target-a/replay-query-id.json", "targetId": "target-a"},
                {"kind": "report", "path": "target-a/report.md", "targetId": "target-a"}
            ])
        );
    }

    #[test]
    fn smoke_artifact_manifest_is_written_before_gate_error() {
        let mut summary = sample_smoke_summary();
        summary.targets[0].replay_count = 0;
        summary.targets[0].replay = None;
        summary.targets[0].replays = Vec::new();
        summary.refresh_gate_status();
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let target_dir = temp_dir.path().join("target-a");
        summary.targets[0].output_dir = target_dir.display().to_string();
        summary.targets[0].corpus = target_dir.join("corpus.json").display().to_string();
        summary.targets[0].schema = target_dir.join("schema.json").display().to_string();
        summary.targets[0].transaction =
            Some(target_dir.join("transaction-0.json").display().to_string());
        summary.targets[0].report = target_dir.join("report.md").display().to_string();

        let err = super::write_smoke_summary_and_gate(&summary, temp_dir.path(), true)
            .unwrap_err()
            .to_string();

        assert!(err.contains("target-a: replays 0"));
        let written = fs::read_to_string(temp_dir.path().join("artifacts.json"))
            .expect("artifact manifest should be written before gate error");
        let json: serde_json::Value =
            serde_json::from_str(&written).expect("artifact manifest should be valid JSON");
        assert_eq!(json["kind"], "stateFlowArtifactManifest");
        assert_eq!(json["summary"], "summary.json");
        assert_eq!(json["artifacts"][1]["path"], "target-a/corpus.json");
        assert_eq!(json["artifacts"].as_array().unwrap().len(), 5);
        assert!(
            json["artifacts"]
                .as_array()
                .unwrap()
                .iter()
                .all(|entry| entry["kind"] != "replay")
        );
    }

    #[test]
    fn report_artifacts_from_manifest_selects_target_bundle() {
        let manifest: super::SmokeArtifactManifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 1,
            "kind": "stateFlowArtifactManifest",
            "summary": "out/summary.json",
            "targetCount": 2,
            "artifacts": [
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "replay", "path": "target-a/replay.json", "targetId": "target-a"},
                {"kind": "report", "path": "target-a/report.md", "targetId": "target-a"},
                {"kind": "corpus", "path": "target-b/corpus.json", "targetId": "target-b"},
                {"kind": "schema", "path": "target-b/schema.json", "targetId": "target-b"},
                {"kind": "replay", "path": "target-b/replay.json", "targetId": "target-b"},
                {"kind": "replay", "path": "target-b/replay-probe-query_id-32-64.json", "targetId": "target-b"},
                {"kind": "report", "path": "target-b/report.md", "targetId": "target-b"}
            ]
        }))
        .expect("artifact manifest should deserialize");

        let inputs = super::report_artifacts_from_manifest(
            &manifest,
            Path::new("out/artifacts.json"),
            Some("target-b"),
        )
        .expect("target report artifacts should resolve");

        assert_eq!(inputs.corpus, PathBuf::from("out/target-b/corpus.json"));
        assert_eq!(inputs.schema, PathBuf::from("out/target-b/schema.json"));
        assert_eq!(
            inputs.replays,
            vec![
                PathBuf::from("out/target-b/replay.json"),
                PathBuf::from("out/target-b/replay-probe-query_id-32-64.json")
            ]
        );
    }

    #[test]
    fn replay_state_flow_from_manifest_selects_target_corpus() {
        let manifest: super::SmokeArtifactManifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 1,
            "kind": "stateFlowArtifactManifest",
            "summary": "out/summary.json",
            "targetCount": 2,
            "artifacts": [
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "corpus", "path": "target-b/corpus.json", "targetId": "target-b"},
                {"kind": "schema", "path": "target-b/schema.json", "targetId": "target-b"}
            ]
        }))
        .expect("artifact manifest should deserialize");

        let state_flow = super::replay_state_flow_from_manifest(
            &manifest,
            Path::new("out/artifacts.json"),
            Some("target-b"),
        )
        .expect("target replay input should resolve");

        assert_eq!(state_flow, PathBuf::from("out/target-b/corpus.json"));
    }

    #[test]
    fn infer_corpus_from_manifest_selects_target_corpus() {
        let manifest: super::SmokeArtifactManifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 1,
            "kind": "stateFlowArtifactManifest",
            "summary": "out/summary.json",
            "targetCount": 2,
            "artifacts": [
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "corpus", "path": "target-b/corpus.json", "targetId": "target-b"},
                {"kind": "schema", "path": "target-b/schema.json", "targetId": "target-b"}
            ]
        }))
        .expect("artifact manifest should deserialize");

        let corpus = super::infer_corpus_from_manifest(
            &manifest,
            Path::new("out/artifacts.json"),
            Some("target-b"),
        )
        .expect("target infer input should resolve");

        assert_eq!(corpus, PathBuf::from("out/target-b/corpus.json"));
    }

    #[test]
    fn artifact_manifest_validation_accepts_portable_bundle() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        for path in [
            "summary.json",
            "target-a/corpus.json",
            "target-a/schema.json",
            "target-a/replay.json",
            "target-a/report.md",
        ] {
            let path = temp_dir.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).expect("parent dir should be created");
            fs::write(path, "{}").expect("artifact should be written");
        }
        let manifest: super::SmokeArtifactManifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 1,
            "kind": "stateFlowArtifactManifest",
            "summary": "summary.json",
            "targetCount": 1,
            "absolutePathCount": 0,
            "artifacts": [
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "replay", "path": "target-a/replay.json", "targetId": "target-a"},
                {"kind": "report", "path": "target-a/report.md", "targetId": "target-a"}
            ]
        }))
        .expect("artifact manifest should deserialize");

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(validation.passed);
        assert_eq!(validation.absolute_path_count, 0);
        assert_eq!(validation.gate_failures, Vec::<String>::new());
        assert_eq!(validation.targets[0].id, "target-a");
        assert!(validation.targets[0].passed);
    }

    #[test]
    fn artifact_manifest_validation_reports_missing_replay() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        for path in [
            "summary.json",
            "target-a/corpus.json",
            "target-a/schema.json",
            "target-a/report.md",
        ] {
            let path = temp_dir.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).expect("parent dir should be created");
            fs::write(path, "{}").expect("artifact should be written");
        }
        let manifest: super::SmokeArtifactManifest = serde_json::from_value(serde_json::json!({
            "schemaVersion": 1,
            "kind": "stateFlowArtifactManifest",
            "summary": "summary.json",
            "targetCount": 1,
            "absolutePathCount": 0,
            "artifacts": [
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "report", "path": "target-a/report.md", "targetId": "target-a"}
            ]
        }))
        .expect("artifact manifest should deserialize");

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert_eq!(
            validation.gate_failures,
            vec!["target-a: missing replay artifact"]
        );
    }

    #[test]
    fn analysis_target_defaults_to_baseline_replay() {
        let target = super::analysis_target_from_args(
            "addr", "mainnet", 2, None, None, None, None, None, false,
        )
        .expect("analysis target should build");

        assert_eq!(target.id, "analysis");
        assert_eq!(target.address, "addr");
        assert_eq!(target.network, "mainnet");
        assert_eq!(target.collect_limit, 2);
        assert_eq!(target.replay_tx_index, None);
        assert_eq!(target.replay_tx_hash, None);
        let replay = target
            .replay_mutation
            .expect("analysis should replay by default");
        assert_eq!(replay.mutation_type, "none");
        assert!(!replay.ignore_chksig);
    }

    #[test]
    fn set_body_uint_replay_mutation_parses_for_analysis() {
        let target = super::analysis_target_from_args(
            "addr",
            "mainnet",
            2,
            None,
            None,
            None,
            None,
            Some("32:64:42".to_owned()),
            true,
        )
        .expect("analysis target should build");

        let replay = target
            .replay_mutation
            .expect("analysis should replay by default");
        assert_eq!(replay.mutation_type, "setBodyUint");
        assert_eq!(replay.bit_offset, Some(32));
        assert_eq!(replay.bits, Some(64));
        assert_eq!(replay.value.as_deref(), Some("42"));
        assert!(replay.ignore_chksig);
        assert!(matches!(
            replay.to_replay_mutation().unwrap(),
            ReplayMutation::SetBodyUint {
                bit_offset: 32,
                bits: 64,
                value
            } if value == "42"
        ));
    }

    #[test]
    fn replay_input_selects_transaction_from_corpus_by_index() {
        let corpus = sample_replay_corpus_json();

        let flow = super::parse_replay_input(&corpus, Path::new("corpus.json"), Some(1), None)
            .expect("corpus replay input should parse");

        assert_eq!(flow.query_hash, "tx-b");
    }

    #[test]
    fn replay_input_selects_transaction_from_corpus_by_hash() {
        let corpus = sample_replay_corpus_json();

        let flow = super::parse_replay_input(&corpus, Path::new("corpus.json"), None, Some("tx-b"))
            .expect("corpus replay input should parse");

        assert_eq!(flow.query_hash, "tx-b");
    }

    #[test]
    fn replay_input_reports_missing_corpus_transaction() {
        let corpus = sample_replay_corpus_json();

        let err = super::parse_replay_input(&corpus, Path::new("corpus.json"), Some(9), None)
            .unwrap_err()
            .to_string();

        assert!(err.contains("corpus transaction index 9 out of range"));
        assert!(err.contains("2 transaction(s)"));
    }

    fn sample_smoke_summary() -> super::SmokeRunSummary {
        super::SmokeRunSummary::from_targets(vec![super::SmokeTargetRunSummary {
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
            passed: false,
            gate_failures: Vec::new(),
            output_dir: "out/target-a".to_owned(),
            corpus: "out/target-a/corpus.json".to_owned(),
            schema: "out/target-a/schema.json".to_owned(),
            transaction: Some("out/target-a/transaction-0.json".to_owned()),
            replay: Some("out/target-a/replay.json".to_owned()),
            replays: vec!["out/target-a/replay.json".to_owned()],
            report: "out/target-a/report.md".to_owned(),
        }])
    }

    fn rewrite_sample_summary_paths(summary: &mut super::SmokeRunSummary, out_dir: &Path) {
        let target_dir = out_dir.join("target-a");
        summary.targets[0].output_dir = target_dir.display().to_string();
        summary.targets[0].corpus = target_dir.join("corpus.json").display().to_string();
        summary.targets[0].schema = target_dir.join("schema.json").display().to_string();
        summary.targets[0].transaction =
            Some(target_dir.join("transaction-0.json").display().to_string());
        summary.targets[0].replay = Some(target_dir.join("replay.json").display().to_string());
        summary.targets[0].replays = vec![target_dir.join("replay.json").display().to_string()];
        summary.targets[0].report = target_dir.join("report.md").display().to_string();
        summary.refresh_gate_status();
    }

    fn sample_replay_corpus_json() -> String {
        serde_json::json!({
            "schemaVersion": 1,
            "network": "mainnet",
            "address": "addr",
            "requestedLimit": 2,
            "sourceTxCount": 2,
            "retracedCount": 2,
            "failureCount": 0,
            "opcodeSummary": [],
            "transactions": [
                sample_state_flow_json("tx-a"),
                sample_state_flow_json("tx-b")
            ],
            "failures": []
        })
        .to_string()
    }

    fn sample_state_flow_json(query_hash: &str) -> serde_json::Value {
        serde_json::json!({
            "schemaVersion": 1,
            "network": "mainnet",
            "queryHash": query_hash,
            "transaction": {
                "lt": 42,
                "utime": 1,
                "account": "addr",
                "stateUpdateHashOk": true,
                "transactionBoc64": "tx"
            },
            "replay": {
                "mcSeqno": 7,
                "randSeedHex": "00",
                "replayedPrevTxCount": 0,
                "blockConfigBoc64": "config",
                "libsBoc64": "libs"
            },
            "state": {
                "pre": sample_snapshot_json("pre", "none"),
                "post": sample_snapshot_json("post", "active")
            },
            "inbound": {
                "direction": "inbound",
                "index": null,
                "kind": "internal",
                "src": "src",
                "dst": "dst",
                "valueNanotons": "1",
                "bounced": false,
                "bounce": true,
                "opcode": "0x00000001",
                "messageBoc64": "msg",
                "body": {"boc64": "body", "hash": "hash", "bits": 32, "refs": 0}
            },
            "outbound": [],
            "compute": {
                "skipped": false,
                "success": true,
                "exitCode": 0,
                "vmSteps": 1,
                "gasUsed": 2,
                "gasFees": 3
            },
            "money": {
                "balanceBefore": 10,
                "sentTotal": 1,
                "totalFees": 2,
                "balanceAfter": 7
            },
            "c5": null,
            "outActions": [],
            "vmTrace": {"lineCount": 0, "text": ""},
            "executorTrace": {"lineCount": 0, "text": ""}
        })
    }

    fn sample_snapshot_json(boc64: &str, status: &str) -> serde_json::Value {
        serde_json::json!({
            "shardAccountBoc64": boc64,
            "lastTransLt": 0,
            "lastTransHash": "00",
            "accountAddress": null,
            "status": status,
            "balanceNanotons": "0",
            "codeHash": null,
            "dataHash": null,
            "codeCell": null,
            "dataCell": null,
            "frozenHash": null
        })
    }
}
