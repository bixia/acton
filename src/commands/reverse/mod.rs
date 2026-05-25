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
const VALIDATION_ARTIFACT_PATH: &str = "validation.json";

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
        #[arg(
            short,
            long,
            alias = "out",
            visible_alias = "out",
            help = "Write artifact validation JSON to a file"
        )]
        output: Option<PathBuf>,
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
            output,
            pretty,
        } => reverse_verify_artifacts_cmd(artifacts, target_id, output, pretty),
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
    output: Option<PathBuf>,
    pretty: bool,
) -> anyhow::Result<()> {
    let manifest = load_artifact_manifest(&artifacts)?;
    let validation =
        validate_artifact_manifest_bundle(&manifest, &artifacts, target_id.as_deref())?;
    write_json(
        &validation,
        output,
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
    let manifest_path = out_dir.join("artifacts.json");
    write_json(
        &manifest,
        Some(manifest_path.clone()),
        pretty,
        "State-flow artifact manifest JSON",
    )?;
    let validation_path = out_dir.join(VALIDATION_ARTIFACT_PATH);
    let pending_validation = pending_artifact_manifest_validation(&manifest, &manifest_path);
    write_json_to_path(&pending_validation, &validation_path, pretty)?;

    let validation = validate_artifact_manifest_bundle_for_generation(&manifest, &manifest_path)?;
    write_json(
        &validation,
        Some(validation_path),
        pretty,
        "State-flow artifact validation JSON",
    )?;

    let quality_result = summary.ensure_passes_gate();
    if quality_result.is_ok() {
        anyhow::ensure!(
            validation.passed,
            "artifact manifest validation failed: {}",
            validation.gate_failures.join("; ")
        );
    }
    quality_result
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
    if let Some(output) = output {
        write_json_to_path(value, &output, pretty)?;
        println!("{label} written to {}", output.display());
    } else {
        let json = serialize_json(value, pretty)?;
        println!("{json}");
    }

    Ok(())
}

fn write_json_to_path<T: Serialize>(value: &T, output: &Path, pretty: bool) -> anyhow::Result<()> {
    let json = serialize_json(value, pretty)?;
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(output, json).with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}

fn serialize_json<T: Serialize>(value: &T, pretty: bool) -> anyhow::Result<String> {
    if pretty {
        Ok(serde_json::to_string_pretty(value)?)
    } else {
        Ok(serde_json::to_string(value)?)
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let validation_path = artifact_dir.join(VALIDATION_ARTIFACT_PATH);
        artifacts.push(SmokeArtifactManifestEntry::new(
            "validation",
            manifest_relative_path(&validation_path, artifact_dir),
            None,
        ));

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    validate_artifact_manifest_bundle_with_mode(
        manifest,
        manifest_path,
        target_id,
        ValidationArtifactMode::RequireCurrent,
    )
}

fn validate_artifact_manifest_bundle_for_generation(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
) -> anyhow::Result<ArtifactManifestValidation> {
    validate_artifact_manifest_bundle_with_mode(
        manifest,
        manifest_path,
        None,
        ValidationArtifactMode::AllowPending,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationArtifactMode {
    RequireCurrent,
    AllowPending,
}

fn validate_artifact_manifest_bundle_with_mode(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: Option<&str>,
    validation_artifact_mode: ValidationArtifactMode,
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
    validate_manifest_run_summary_entry(manifest, manifest_path, &mut gate_failures);

    let summary = validate_manifest_summary_artifact(manifest, manifest_path, &mut gate_failures);
    if let Some(summary) = &summary {
        validate_manifest_summary_targets(manifest, &target_ids, summary, &mut gate_failures);
    }
    for artifact in &manifest.artifacts {
        let path = resolve_manifest_artifact_path(manifest_path, &artifact.path);
        if !path.exists() {
            gate_failures.push(format!("missing artifact {}", artifact.path));
            continue;
        }
        validate_manifest_artifact_content(&path, artifact, &mut gate_failures);
    }

    let targets = selected_target_ids
        .iter()
        .map(|target_id| {
            validate_artifact_manifest_target(manifest, manifest_path, target_id, summary.as_ref())
        })
        .collect::<Vec<_>>();
    gate_failures.extend(targets.iter().flat_map(|target| {
        target
            .gate_failures
            .iter()
            .map(|failure| format!("{}: {failure}", target.id))
    }));

    let expected_validation = ArtifactManifestValidation {
        schema_version: 1,
        kind: "stateFlowArtifactManifestValidation".to_owned(),
        manifest: manifest_path.display().to_string(),
        target_count: selected_target_ids.len(),
        absolute_path_count: manifest.absolute_path_count,
        expected_absolute_path_count,
        passed: gate_failures.is_empty(),
        gate_failures: gate_failures.clone(),
        targets: targets.clone(),
    };

    if validation_artifact_mode == ValidationArtifactMode::RequireCurrent && target_id.is_none() {
        validate_manifest_validation_artifact(
            manifest,
            manifest_path,
            &expected_validation,
            &mut gate_failures,
        );
    }

    Ok(ArtifactManifestValidation {
        passed: gate_failures.is_empty(),
        gate_failures,
        ..expected_validation
    })
}

fn validate_manifest_run_summary_entry(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    gate_failures: &mut Vec<String>,
) {
    let run_summaries = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == "runSummary")
        .collect::<Vec<_>>();
    match run_summaries.as_slice() {
        [run_summary] => {
            if resolve_manifest_artifact_path(manifest_path, &run_summary.path)
                != resolve_manifest_artifact_path(manifest_path, &manifest.summary)
            {
                gate_failures.push(format!(
                    "runSummary artifact path {} does not match manifest summary {}",
                    run_summary.path, manifest.summary
                ));
            }
        }
        [] => gate_failures.push("missing runSummary artifact entry".to_owned()),
        _ => gate_failures.push("multiple runSummary artifact entries".to_owned()),
    }
}

fn validate_manifest_summary_artifact(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    gate_failures: &mut Vec<String>,
) -> Option<SmokeRunSummary> {
    let summary_path = resolve_manifest_artifact_path(manifest_path, &manifest.summary);
    if !summary_path.exists() {
        gate_failures.push(format!("missing summary artifact {}", manifest.summary));
        return None;
    }

    let summary_json = match fs::read_to_string(&summary_path) {
        Ok(summary_json) => summary_json,
        Err(err) => {
            gate_failures.push(format!(
                "failed to read summary artifact {}: {err}",
                manifest.summary
            ));
            return None;
        }
    };
    match serde_json::from_str::<SmokeRunSummary>(&summary_json) {
        Ok(summary) => Some(summary),
        Err(err) => {
            gate_failures.push(format!(
                "invalid summary artifact {}: {err}",
                manifest.summary
            ));
            None
        }
    }
}

fn validate_manifest_summary_targets(
    manifest: &SmokeArtifactManifest,
    manifest_target_ids: &[String],
    summary: &SmokeRunSummary,
    gate_failures: &mut Vec<String>,
) {
    if summary.target_count != summary.targets.len() {
        gate_failures.push(format!(
            "summary target count mismatch: summary {}, actual {}",
            summary.target_count,
            summary.targets.len()
        ));
    }

    for target in &summary.targets {
        if !manifest_target_ids.iter().any(|id| id == &target.id) {
            gate_failures.push(format!(
                "summary target {} has no manifest artifacts",
                target.id
            ));
        }
    }
    for target_id in manifest_target_ids {
        if !summary.targets.iter().any(|target| &target.id == target_id) {
            gate_failures.push(format!("manifest target {target_id} has no summary target"));
        }
    }

    if manifest.target_count != summary.targets.len() {
        gate_failures.push(format!(
            "manifest target count {} does not match summary target count {}",
            manifest.target_count,
            summary.targets.len()
        ));
    }
}

fn pending_artifact_manifest_validation(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
) -> ArtifactManifestValidation {
    ArtifactManifestValidation {
        schema_version: 1,
        kind: "stateFlowArtifactManifestValidation".to_owned(),
        manifest: manifest_path.display().to_string(),
        target_count: manifest_target_ids(manifest).len(),
        absolute_path_count: manifest.absolute_path_count,
        expected_absolute_path_count: smoke_manifest_absolute_path_count(
            &manifest.summary,
            &manifest.artifacts,
        ),
        passed: false,
        gate_failures: vec!["artifact manifest validation pending".to_owned()],
        targets: Vec::new(),
    }
}

fn validate_manifest_artifact_content(
    path: &Path,
    artifact: &SmokeArtifactManifestEntry,
    gate_failures: &mut Vec<String>,
) {
    match artifact.kind.as_str() {
        "runSummary" => validate_json_artifact::<SmokeRunSummary>(path, artifact, gate_failures),
        "corpus" => validate_json_artifact::<StateFlowCorpus>(path, artifact, gate_failures),
        "schema" => validate_json_artifact::<StateFlowSchemaReport>(path, artifact, gate_failures),
        "transaction" => validate_json_artifact::<StateFlowTx>(path, artifact, gate_failures),
        "replay" => validate_json_artifact::<StateFlowReplayDiff>(path, artifact, gate_failures),
        "validation" => {
            validate_json_artifact::<ArtifactManifestValidation>(path, artifact, gate_failures)
        }
        "report" => validate_report_artifact(path, artifact, gate_failures),
        _ => {}
    }
}

fn validate_manifest_validation_artifact(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    expected: &ArtifactManifestValidation,
    gate_failures: &mut Vec<String>,
) {
    let validation_artifacts = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == "validation")
        .collect::<Vec<_>>();
    let [artifact] = validation_artifacts.as_slice() else {
        if validation_artifacts.len() > 1 {
            gate_failures.push("multiple validation artifact entries".to_owned());
        }
        return;
    };
    if artifact.target_id.is_some() {
        gate_failures.push(format!(
            "validation artifact {} must not have a target id",
            artifact.path
        ));
    }

    let path = resolve_manifest_artifact_path(manifest_path, &artifact.path);
    let Ok(json) = fs::read_to_string(&path) else {
        return;
    };
    let Ok(actual) = serde_json::from_str::<ArtifactManifestValidation>(&json) else {
        return;
    };

    validate_target_usize_field(
        "validation schema version",
        actual.schema_version as usize,
        "expected schema version",
        expected.schema_version as usize,
        gate_failures,
    );
    validate_target_text_field(
        "validation kind",
        &actual.kind,
        "expected kind",
        &expected.kind,
        gate_failures,
    );
    validate_target_usize_field(
        "validation target count",
        actual.target_count,
        "expected target count",
        expected.target_count,
        gate_failures,
    );
    validate_target_usize_field(
        "validation absolute path count",
        actual.absolute_path_count,
        "expected absolute path count",
        expected.absolute_path_count,
        gate_failures,
    );
    validate_target_usize_field(
        "validation expected absolute path count",
        actual.expected_absolute_path_count,
        "expected expected absolute path count",
        expected.expected_absolute_path_count,
        gate_failures,
    );
    if actual.passed != expected.passed {
        gate_failures.push(format!(
            "validation passed {} does not match expected passed {}",
            actual.passed, expected.passed
        ));
    }
    if actual.gate_failures != expected.gate_failures {
        gate_failures.push(format!(
            "validation gate failures {:?} do not match expected gate failures {:?}",
            actual.gate_failures, expected.gate_failures
        ));
    }
    if actual.targets != expected.targets {
        gate_failures.push("validation targets do not match expected targets".to_owned());
    }
}

fn validate_json_artifact<T>(
    path: &Path,
    artifact: &SmokeArtifactManifestEntry,
    gate_failures: &mut Vec<String>,
) where
    T: for<'de> Deserialize<'de>,
{
    match fs::read_to_string(path) {
        Ok(json) => {
            if let Err(err) = serde_json::from_str::<T>(&json) {
                gate_failures.push(format!(
                    "invalid {} artifact {}: {err}",
                    artifact.kind, artifact.path
                ));
            }
        }
        Err(err) => gate_failures.push(format!(
            "failed to read {} artifact {}: {err}",
            artifact.kind, artifact.path
        )),
    }
}

fn validate_report_artifact(
    path: &Path,
    artifact: &SmokeArtifactManifestEntry,
    gate_failures: &mut Vec<String>,
) {
    match fs::read_to_string(path) {
        Ok(markdown) if markdown.trim().is_empty() => {
            gate_failures.push(format!("empty report artifact {}", artifact.path));
        }
        Ok(_) => {}
        Err(err) => gate_failures.push(format!(
            "failed to read report artifact {}: {err}",
            artifact.path
        )),
    }
}

fn validate_artifact_manifest_target(
    manifest: &SmokeArtifactManifest,
    manifest_path: &Path,
    target_id: &str,
    summary: Option<&SmokeRunSummary>,
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
    if let Some(summary) = summary {
        validate_manifest_target_matches_summary(
            manifest_path,
            target_id,
            &artifacts,
            summary,
            &mut gate_failures,
        );
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

fn validate_manifest_target_matches_summary(
    manifest_path: &Path,
    target_id: &str,
    artifacts: &[&SmokeArtifactManifestEntry],
    summary: &SmokeRunSummary,
    gate_failures: &mut Vec<String>,
) {
    let Some(target) = summary.targets.iter().find(|target| target.id == target_id) else {
        gate_failures.push("missing summary target".to_owned());
        return;
    };

    validate_summary_single_artifact_path(
        manifest_path,
        artifacts,
        "corpus",
        &target.corpus,
        gate_failures,
    );
    validate_summary_single_artifact_path(
        manifest_path,
        artifacts,
        "schema",
        &target.schema,
        gate_failures,
    );
    validate_summary_optional_artifact_path(
        manifest_path,
        artifacts,
        "transaction",
        target.transaction.as_deref(),
        gate_failures,
    );
    validate_summary_replay_artifact_paths(manifest_path, artifacts, target, gate_failures);
    validate_summary_single_artifact_path(
        manifest_path,
        artifacts,
        "report",
        &target.report,
        gate_failures,
    );
    validate_manifest_target_content_matches_summary(
        manifest_path,
        artifacts,
        target,
        gate_failures,
    );
}

fn validate_summary_single_artifact_path(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    kind: &str,
    expected_path: &str,
    gate_failures: &mut Vec<String>,
) {
    let matches = artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .copied()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [artifact] => {
            if !manifest_artifact_path_matches_summary(manifest_path, &artifact.path, expected_path)
            {
                gate_failures.push(format!(
                    "{kind} artifact path {} does not match summary path {expected_path}",
                    artifact.path
                ));
            }
        }
        [] => {}
        _ => gate_failures.push(format!("multiple {kind} artifacts")),
    }
}

fn validate_summary_optional_artifact_path(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    kind: &str,
    expected_path: Option<&str>,
    gate_failures: &mut Vec<String>,
) {
    if let Some(expected_path) = expected_path {
        validate_summary_single_artifact_path(
            manifest_path,
            artifacts,
            kind,
            expected_path,
            gate_failures,
        );
        return;
    }

    for artifact in artifacts.iter().filter(|artifact| artifact.kind == kind) {
        gate_failures.push(format!(
            "{kind} artifact path {} is not listed in summary",
            artifact.path
        ));
    }
}

fn validate_summary_replay_artifact_paths(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    target: &SmokeTargetRunSummary,
    gate_failures: &mut Vec<String>,
) {
    let manifest_replays = artifacts
        .iter()
        .filter(|artifact| artifact.kind == "replay")
        .copied()
        .collect::<Vec<_>>();
    let summary_replays = replay_artifact_paths(target);
    if manifest_replays.is_empty() {
        return;
    }
    if manifest_replays.len() != summary_replays.len() {
        gate_failures.push(format!(
            "replay artifact count mismatch: manifest {}, summary {}",
            manifest_replays.len(),
            summary_replays.len()
        ));
    }

    for artifact in &manifest_replays {
        if !summary_replays.iter().any(|summary_path| {
            manifest_artifact_path_matches_summary(manifest_path, &artifact.path, summary_path)
        }) {
            gate_failures.push(format!(
                "replay artifact path {} is not listed in summary",
                artifact.path
            ));
        }
    }
    for summary_path in &summary_replays {
        if !manifest_replays.iter().any(|artifact| {
            manifest_artifact_path_matches_summary(manifest_path, &artifact.path, summary_path)
        }) {
            gate_failures.push(format!(
                "summary replay path {summary_path} is missing from manifest"
            ));
        }
    }
}

fn manifest_artifact_path_matches_summary(
    manifest_path: &Path,
    artifact_path: &str,
    summary_path: &str,
) -> bool {
    resolve_manifest_artifact_path(manifest_path, artifact_path)
        == resolve_manifest_artifact_path(manifest_path, summary_path)
}

fn validate_manifest_target_content_matches_summary(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    target: &SmokeTargetRunSummary,
    gate_failures: &mut Vec<String>,
) {
    let corpus =
        read_single_target_json_artifact::<StateFlowCorpus>(manifest_path, artifacts, "corpus");
    if let Some(corpus) = &corpus {
        validate_target_text_field(
            "corpus network",
            &corpus.network,
            "summary network",
            &target.network,
            gate_failures,
        );
        validate_target_text_field(
            "corpus address",
            &corpus.address,
            "summary address",
            &target.address,
            gate_failures,
        );
        validate_target_usize_field(
            "corpus requested limit",
            corpus.requested_limit as usize,
            "summary collect limit",
            target.collect_limit as usize,
            gate_failures,
        );
        validate_target_usize_field(
            "corpus source transaction count",
            corpus.source_tx_count,
            "summary source transaction count",
            target.source_tx_count,
            gate_failures,
        );
        validate_target_usize_field(
            "corpus retraced count",
            corpus.retraced_count,
            "summary retraced count",
            target.retraced_count,
            gate_failures,
        );
        validate_target_usize_field(
            "corpus failure count",
            corpus.failure_count,
            "summary failure count",
            target.failure_count,
            gate_failures,
        );
        validate_corpus_internal_counts(corpus, gate_failures);
    }
    validate_manifest_transaction_membership(
        manifest_path,
        artifacts,
        corpus.as_ref(),
        gate_failures,
    );
    validate_manifest_replay_membership(manifest_path, artifacts, corpus.as_ref(), gate_failures);

    if let Some(schema) = read_single_target_json_artifact::<StateFlowSchemaReport>(
        manifest_path,
        artifacts,
        "schema",
    ) {
        validate_target_text_field(
            "schema network",
            &schema.network,
            "summary network",
            &target.network,
            gate_failures,
        );
        validate_target_text_field(
            "schema address",
            &schema.address,
            "summary address",
            &target.address,
            gate_failures,
        );
        validate_target_usize_field(
            "schema transaction count",
            schema.transaction_count,
            "summary retraced count",
            target.retraced_count,
            gate_failures,
        );
        validate_target_usize_field(
            "schema opcode candidate count",
            schema.opcode_candidates.len(),
            "summary opcode candidate count",
            target.opcode_candidate_count,
            gate_failures,
        );
        validate_target_usize_field(
            "schema state edge count",
            schema.state_machine.edges.len(),
            "summary state edge count",
            target.state_edge_count,
            gate_failures,
        );
        validate_target_usize_field(
            "schema audit signal count",
            schema.audit_signals.len(),
            "summary audit signal count",
            target.audit_signal_count,
            gate_failures,
        );
        validate_schema_corpus_membership(&schema, corpus.as_ref(), gate_failures);
        validate_schema_replay_probe_artifacts(&schema, manifest_path, artifacts, gate_failures);
    }
    validate_manifest_report_content_matches_summary(
        manifest_path,
        artifacts,
        target,
        gate_failures,
    );
}

fn validate_corpus_internal_counts(corpus: &StateFlowCorpus, gate_failures: &mut Vec<String>) {
    validate_target_usize_field(
        "corpus retraced count",
        corpus.retraced_count,
        "transaction list length",
        corpus.transactions.len(),
        gate_failures,
    );
    validate_target_usize_field(
        "corpus failure count",
        corpus.failure_count,
        "failure list length",
        corpus.failures.len(),
        gate_failures,
    );
    validate_target_usize_field(
        "corpus source transaction count",
        corpus.source_tx_count,
        "retraced plus failure count",
        corpus.retraced_count + corpus.failure_count,
        gate_failures,
    );
}

fn validate_schema_corpus_membership(
    schema: &StateFlowSchemaReport,
    corpus: Option<&StateFlowCorpus>,
    gate_failures: &mut Vec<String>,
) {
    let Some(corpus) = corpus else {
        return;
    };
    let corpus_hashes = corpus_transaction_hashes(corpus);
    for edge in &schema.state_machine.edges {
        for example in &edge.examples {
            validate_corpus_hash_membership(
                "schema state-machine example",
                example,
                &corpus_hashes,
                gate_failures,
            );
        }
    }
    for candidate in &schema.opcode_candidates {
        for example in &candidate.examples {
            validate_corpus_hash_membership(
                "schema candidate example",
                example,
                &corpus_hashes,
                gate_failures,
            );
        }
        for evidence in &candidate.evidence {
            validate_corpus_hash_membership(
                "schema evidence tx hash",
                &evidence.tx_hash,
                &corpus_hashes,
                gate_failures,
            );
        }
        for probe in &candidate.replay_probes {
            for evidence in &probe.evidence {
                validate_corpus_hash_membership(
                    "schema replay probe evidence",
                    evidence,
                    &corpus_hashes,
                    gate_failures,
                );
            }
        }
        for effect in &candidate.outbound_effects {
            for tx_hash in &effect.tx_hashes {
                validate_corpus_hash_membership(
                    "schema outbound effect tx hash",
                    tx_hash,
                    &corpus_hashes,
                    gate_failures,
                );
            }
        }
        for effect in &candidate.out_actions {
            for tx_hash in &effect.tx_hashes {
                validate_corpus_hash_membership(
                    "schema out-action tx hash",
                    tx_hash,
                    &corpus_hashes,
                    gate_failures,
                );
            }
        }
    }
}

fn validate_schema_replay_probe_artifacts(
    schema: &StateFlowSchemaReport,
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    gate_failures: &mut Vec<String>,
) {
    let replays =
        read_target_json_artifacts::<StateFlowReplayDiff>(manifest_path, artifacts, "replay");
    for candidate in &schema.opcode_candidates {
        for probe in &candidate.replay_probes {
            if !replays
                .iter()
                .any(|replay| replay_matches_schema_probe(replay, probe))
            {
                gate_failures.push(format!(
                    "schema replay probe {} has no matching replay artifact",
                    probe.cli_arg
                ));
            } else if !replays
                .iter()
                .any(|replay| replay_matches_schema_probe_source(replay, probe))
            {
                gate_failures.push(format!(
                    "schema replay probe {} has no replay artifact for evidence source",
                    probe.cli_arg
                ));
            }
        }
    }
}

fn replay_matches_schema_probe(
    replay: &StateFlowReplayDiff,
    probe: &ton_stateflow::ReplayProbeCandidate,
) -> bool {
    replay_mutations_match(&replay.mutation, &probe.mutation)
}

fn replay_matches_schema_probe_source(
    replay: &StateFlowReplayDiff,
    probe: &ton_stateflow::ReplayProbeCandidate,
) -> bool {
    replay_matches_schema_probe(replay, probe)
        && probe
            .evidence
            .iter()
            .any(|tx_hash| tx_hash == &replay.source_query_hash)
}

fn replay_mutations_match(actual: &ReplayMutation, expected: &ReplayMutation) -> bool {
    match (actual, expected) {
        (ReplayMutation::None, ReplayMutation::None) => true,
        (
            ReplayMutation::FlipBodyBit { bit: actual },
            ReplayMutation::FlipBodyBit { bit: expected },
        ) => actual == expected,
        (
            ReplayMutation::ReplaceBody { body_boc64: actual },
            ReplayMutation::ReplaceBody {
                body_boc64: expected,
            },
        ) => actual == expected,
        (
            ReplayMutation::SetBodyUint {
                bit_offset: actual_offset,
                bits: actual_bits,
                value: actual_value,
            },
            ReplayMutation::SetBodyUint {
                bit_offset: expected_offset,
                bits: expected_bits,
                value: expected_value,
            },
        ) => {
            actual_offset == expected_offset
                && actual_bits == expected_bits
                && actual_value == expected_value
        }
        _ => false,
    }
}

fn validate_manifest_report_content_matches_summary(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    target: &SmokeTargetRunSummary,
    gate_failures: &mut Vec<String>,
) {
    let Some(markdown) = read_single_target_text_artifact(manifest_path, artifacts, "report")
    else {
        return;
    };
    let required_target_lines = [
        format!("- Network: `{}`", target.network),
        format!("- Address: `{}`", target.address),
        format!("- Source transactions: {}", target.source_tx_count),
        format!("- Retraced transactions: {}", target.retraced_count),
        format!(
            "- Replay failures while collecting: {}",
            target.failure_count
        ),
    ];
    for line in &required_target_lines {
        if !markdown_line_exists(&markdown, line) {
            gate_failures.push(format!("report target line {line:?} is missing"));
        }
    }

    for section in [
        "# TON State Flow Reverse Report",
        "## Target",
        "## Opcode Candidates",
        "## Schema Evidence",
        "## Message Body Fields",
        "## Replay Probes",
        "## Storage Fields",
        "## Outbound Effects",
        "## State Machine",
        "## Unknown Fields",
        "## Replay Diffs",
        "## Risk Points",
    ] {
        if !markdown_line_exists(&markdown, section) {
            gate_failures.push(format!("report section {section:?} is missing"));
        }
    }

    if let Some(schema) = read_single_target_json_artifact::<StateFlowSchemaReport>(
        manifest_path,
        artifacts,
        "schema",
    ) {
        validate_report_schema_deliverables(&markdown, &schema, gate_failures);
        let schema_evidence_section = markdown_section(&markdown, "## Schema Evidence");
        for evidence in schema_evidence_rows(&schema) {
            let evidence_row = schema_evidence_section
                .and_then(|section| report_schema_evidence_row(section, evidence));
            if evidence_row.is_none() {
                gate_failures.push(format!(
                    "report schema evidence tx hash {} is missing",
                    evidence.tx_hash
                ));
            }
            if let Some(row) = evidence_row {
                validate_report_schema_evidence_values(evidence, &row, gate_failures);
            }
        }
    }

    let replay_diff_section = markdown_section(&markdown, "## Replay Diffs");
    for replay in
        read_target_json_artifacts::<StateFlowReplayDiff>(manifest_path, artifacts, "replay")
    {
        let replay_row =
            replay_diff_section.and_then(|section| report_replay_diff_row(section, &replay));
        if replay_row.is_none() {
            gate_failures.push(format!(
                "report replay tx hash {} is missing",
                replay.source_query_hash
            ));
        }
        let mutation_label = report_replay_mutation_label(&replay.mutation);
        if replay_row.is_none() {
            gate_failures.push(format!(
                "report replay mutation {mutation_label:?} for tx {} is missing",
                replay.source_query_hash
            ));
        }
        if let Some(row) = replay_row {
            validate_report_replay_diff_values(&replay, &row, gate_failures);
        }
    }
}

fn report_replay_diff_row(section: &str, replay: &StateFlowReplayDiff) -> Option<Vec<String>> {
    let mutation_label = report_replay_mutation_label(&replay.mutation);
    section.lines().find_map(|line| {
        let cells = markdown_table_cells(line)?;
        (cells
            .get(0)
            .is_some_and(|cell| cell == &replay.source_query_hash)
            && cells.get(1).is_some_and(|cell| cell == &mutation_label))
        .then_some(cells)
    })
}

fn validate_report_replay_diff_values(
    replay: &StateFlowReplayDiff,
    row: &[String],
    gate_failures: &mut Vec<String>,
) {
    validate_report_replay_diff_cell(
        "accepted",
        replay.diff.replay_accepted.to_string(),
        replay,
        row.get(2),
        gate_failures,
    );
    validate_report_replay_diff_cell(
        "input changed",
        replay.diff.input_changed.to_string(),
        replay,
        row.get(3),
        gate_failures,
    );
    validate_report_replay_diff_cell(
        "state changed",
        report_optional_bool(replay.diff.state_changed),
        replay,
        row.get(4),
        gate_failures,
    );
    validate_report_replay_diff_cell(
        "exit changed",
        report_optional_bool(replay.diff.exit_code_changed),
        replay,
        row.get(5),
        gate_failures,
    );
    validate_report_replay_diff_cell(
        "outbound delta",
        replay
            .diff
            .outbound_count_delta
            .map_or("n/a".to_owned(), |value| value.to_string()),
        replay,
        row.get(6),
        gate_failures,
    );
    validate_report_replay_diff_cell(
        "action delta",
        replay
            .diff
            .action_count_delta
            .map_or("n/a".to_owned(), |value| value.to_string()),
        replay,
        row.get(7),
        gate_failures,
    );
}

fn validate_report_replay_diff_cell(
    label: &str,
    expected: String,
    replay: &StateFlowReplayDiff,
    actual: Option<&String>,
    gate_failures: &mut Vec<String>,
) {
    if actual.is_none_or(|actual| actual != &expected) {
        gate_failures.push(format!(
            "report replay {label} {expected} for tx {} is missing",
            replay.source_query_hash
        ));
    }
}

fn report_optional_bool(value: Option<bool>) -> String {
    value.map_or("n/a".to_owned(), |value| value.to_string())
}

fn schema_evidence_rows(schema: &StateFlowSchemaReport) -> Vec<&ton_stateflow::SchemaEvidence> {
    schema
        .opcode_candidates
        .iter()
        .flat_map(|candidate| candidate.evidence.iter())
        .collect()
}

fn report_schema_evidence_row(
    section: &str,
    evidence: &ton_stateflow::SchemaEvidence,
) -> Option<Vec<String>> {
    section.lines().find_map(|line| {
        let cells = markdown_table_cells(line)?;
        cells
            .get(1)
            .is_some_and(|cell| cell == &evidence.tx_hash)
            .then_some(cells)
    })
}

fn validate_report_schema_evidence_values(
    evidence: &ton_stateflow::SchemaEvidence,
    row: &[String],
    gate_failures: &mut Vec<String>,
) {
    validate_report_schema_evidence_cell(
        "body hash",
        evidence.inbound_body_hash.clone(),
        &evidence.tx_hash,
        row.get(2),
        gate_failures,
    );
    validate_report_schema_evidence_cell(
        "body bits/refs",
        format!(
            "{}/{}",
            evidence.inbound_body_bits, evidence.inbound_body_refs
        ),
        &evidence.tx_hash,
        row.get(3),
        gate_failures,
    );
    validate_report_schema_evidence_cell(
        "state",
        format!("{} -> {}", evidence.from_status, evidence.to_status),
        &evidence.tx_hash,
        row.get(4),
        gate_failures,
    );
    validate_report_schema_evidence_cell(
        "data hash",
        report_hash_transition(&evidence.pre_data_hash, &evidence.post_data_hash),
        &evidence.tx_hash,
        row.get(5),
        gate_failures,
    );
    validate_report_schema_evidence_cell(
        "code hash",
        report_hash_transition(&evidence.pre_code_hash, &evidence.post_code_hash),
        &evidence.tx_hash,
        row.get(6),
        gate_failures,
    );
    validate_report_schema_evidence_cell(
        "outbound",
        report_kind_list(&evidence.outbound_kinds),
        &evidence.tx_hash,
        row.get(7),
        gate_failures,
    );
    validate_report_schema_evidence_cell(
        "actions",
        report_kind_list(&evidence.out_action_kinds),
        &evidence.tx_hash,
        row.get(8),
        gate_failures,
    );
}

fn report_hash_transition(before: &Option<String>, after: &Option<String>) -> String {
    format!(
        "{} -> {}",
        before.as_deref().unwrap_or("<none>"),
        after.as_deref().unwrap_or("<none>")
    )
}

fn validate_report_schema_evidence_cell(
    label: &str,
    expected: String,
    tx_hash: &str,
    actual: Option<&String>,
    gate_failures: &mut Vec<String>,
) {
    if actual.is_none_or(|actual| actual != &expected) {
        gate_failures.push(format!(
            "report schema evidence {label} {expected} for tx {tx_hash} is missing"
        ));
    }
}

fn report_kind_list(kinds: &[String]) -> String {
    if kinds.is_empty() {
        return "none".to_owned();
    }
    kinds.join(", ")
}

fn validate_report_schema_deliverables(
    markdown: &str,
    schema: &StateFlowSchemaReport,
    gate_failures: &mut Vec<String>,
) {
    if let Some(section) = markdown_section(markdown, "## Opcode Candidates") {
        for candidate in &schema.opcode_candidates {
            let opcode = report_opcode_label(candidate.opcode.as_deref());
            let candidate_row = report_opcode_candidate_row(section, &opcode);
            if candidate_row.is_none() {
                gate_failures.push(format!(
                    "report opcode candidate {opcode} with confidence {} is missing",
                    candidate.confidence
                ));
            }
            if let Some(row) = candidate_row {
                validate_report_opcode_candidate_values(candidate, &opcode, &row, gate_failures);
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## Message Body Fields") {
        for candidate in &schema.opcode_candidates {
            let opcode = report_opcode_label(candidate.opcode.as_deref());
            for field in &candidate.inbound_body.field_candidates {
                let field_row = report_message_body_field_row(section, &opcode, field);
                if field_row.is_none() {
                    gate_failures.push(format!(
                        "report message body field {} is missing",
                        field.name
                    ));
                }
                if let Some(row) = field_row {
                    validate_report_message_body_field_values(field, &row, gate_failures);
                }
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## Replay Probes") {
        for candidate in &schema.opcode_candidates {
            let opcode = report_opcode_label(candidate.opcode.as_deref());
            for probe in &candidate.replay_probes {
                let probe_row = report_replay_probe_row(section, &opcode, probe);
                if probe_row.is_none() {
                    gate_failures.push(format!("report replay probe {} is missing", probe.cli_arg));
                }
                if let Some(row) = probe_row {
                    validate_report_replay_probe_values(probe, &row, gate_failures);
                }
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## Storage Fields") {
        for candidate in &schema.opcode_candidates {
            let opcode = report_opcode_label(candidate.opcode.as_deref());
            for field in &candidate.storage.fields {
                let field_row = report_storage_field_row(section, &opcode, field);
                if field_row.is_none() {
                    gate_failures.push(format!("report storage field {} is missing", field.name));
                }
                if let Some(row) = field_row {
                    validate_report_storage_field_values(field, &row, gate_failures);
                }
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## Unknown Fields") {
        for candidate in &schema.opcode_candidates {
            for field in &candidate.unknown_fields {
                if !section.contains(field) {
                    gate_failures.push(format!("report unknown field {field} is missing"));
                }
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## Outbound Effects") {
        for candidate in &schema.opcode_candidates {
            for effect in &candidate.outbound_effects {
                validate_report_effect_row("outbound", effect, section, gate_failures);
            }
            for effect in &candidate.out_actions {
                validate_report_effect_row("action", effect, section, gate_failures);
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## State Machine") {
        for edge in &schema.state_machine.edges {
            let edge_label = format!(
                "{} --> {}: {} ({})",
                edge.from_status,
                edge.to_status,
                report_opcode_label(edge.opcode.as_deref()),
                edge.count
            );
            if !section.contains(&edge_label) {
                gate_failures.push(format!("report state edge {edge_label} is missing"));
            }
        }
    }

    if let Some(section) = markdown_section(markdown, "## Risk Points") {
        for signal in &schema.audit_signals {
            if !section.contains(&signal.description) {
                gate_failures.push(format!("report risk {:?} is missing", signal.description));
            }
        }
    }
}

fn validate_report_effect_row(
    source: &str,
    effect: &ton_stateflow::EffectCandidate,
    section: &str,
    gate_failures: &mut Vec<String>,
) {
    if !section.contains(source) || !section.contains(&effect.kind) {
        gate_failures.push(format!(
            "report outbound effect {source} {} is missing",
            effect.kind
        ));
    }
}

fn report_opcode_label(opcode: Option<&str>) -> String {
    opcode.unwrap_or("<none>").to_owned()
}

fn report_opcode_candidate_row(section: &str, opcode: &str) -> Option<Vec<String>> {
    section.lines().find_map(|line| {
        let cells = markdown_table_cells(line)?;
        cells
            .get(0)
            .is_some_and(|cell| cell == opcode)
            .then_some(cells)
    })
}

fn validate_report_opcode_candidate_values(
    candidate: &ton_stateflow::OpcodeSchemaCandidate,
    opcode: &str,
    row: &[String],
    gate_failures: &mut Vec<String>,
) {
    validate_report_opcode_candidate_cell(
        "count",
        candidate.count.to_string(),
        opcode,
        row.get(1),
        gate_failures,
    );
    validate_report_opcode_candidate_cell(
        "confidence",
        candidate.confidence.clone(),
        opcode,
        row.get(2),
        gate_failures,
    );
    validate_report_opcode_candidate_cell(
        "body bits",
        report_range(
            candidate.inbound_body.min_bits,
            candidate.inbound_body.max_bits,
        ),
        opcode,
        row.get(3),
        gate_failures,
    );
    validate_report_opcode_candidate_cell(
        "body refs",
        report_range(
            candidate.inbound_body.min_refs,
            candidate.inbound_body.max_refs,
        ),
        opcode,
        row.get(4),
        gate_failures,
    );
    validate_report_opcode_candidate_cell(
        "evidence",
        candidate.examples.join(", "),
        opcode,
        row.get(9),
        gate_failures,
    );
}

fn validate_report_opcode_candidate_cell(
    label: &str,
    expected: String,
    opcode: &str,
    actual: Option<&String>,
    gate_failures: &mut Vec<String>,
) {
    if actual.is_none_or(|actual| actual != &expected) {
        gate_failures.push(format!(
            "report opcode candidate {label} {expected} for {opcode} is missing"
        ));
    }
}

fn report_message_body_field_row(
    section: &str,
    opcode: &str,
    field: &ton_stateflow::BodyFieldCandidate,
) -> Option<Vec<String>> {
    section.lines().find_map(|line| {
        let cells = markdown_table_cells(line)?;
        (cells.get(0).is_some_and(|cell| cell == opcode)
            && cells.get(1).is_some_and(|cell| cell == &field.name))
        .then_some(cells)
    })
}

fn validate_report_message_body_field_values(
    field: &ton_stateflow::BodyFieldCandidate,
    row: &[String],
    gate_failures: &mut Vec<String>,
) {
    validate_report_message_body_field_cell(
        "offset",
        field.bit_offset.to_string(),
        &field.name,
        row.get(2),
        gate_failures,
    );
    validate_report_message_body_field_cell(
        "bits",
        report_field_range(field.min_bits, field.max_bits),
        &field.name,
        row.get(3),
        gate_failures,
    );
    validate_report_message_body_field_cell(
        "refs",
        report_field_range(field.min_refs, field.max_refs),
        &field.name,
        row.get(4),
        gate_failures,
    );
    validate_report_message_body_field_cell(
        "kind",
        field.kind.clone(),
        &field.name,
        row.get(5),
        gate_failures,
    );
    validate_report_message_body_field_cell(
        "samples",
        report_sample_list(&field.value_samples),
        &field.name,
        row.get(6),
        gate_failures,
    );
    validate_report_message_body_field_cell(
        "confidence",
        field.confidence.clone(),
        &field.name,
        row.get(7),
        gate_failures,
    );
}

fn validate_report_message_body_field_cell(
    label: &str,
    expected: String,
    field_name: &str,
    actual: Option<&String>,
    gate_failures: &mut Vec<String>,
) {
    if actual.is_none_or(|actual| actual != &expected) {
        gate_failures.push(format!(
            "report message body field {label} {expected} for {field_name} is missing"
        ));
    }
}

fn report_replay_probe_row(
    section: &str,
    opcode: &str,
    probe: &ton_stateflow::ReplayProbeCandidate,
) -> Option<Vec<String>> {
    section.lines().find_map(|line| {
        let cells = markdown_table_cells(line)?;
        (cells.get(0).is_some_and(|cell| cell == opcode)
            && cells.get(1).is_some_and(|cell| cell == &probe.field_name)
            && cells.get(2).is_some_and(|cell| cell == &probe.cli_arg))
        .then_some(cells)
    })
}

fn validate_report_replay_probe_values(
    probe: &ton_stateflow::ReplayProbeCandidate,
    row: &[String],
    gate_failures: &mut Vec<String>,
) {
    validate_report_replay_probe_cell(
        "confidence",
        probe.confidence.clone(),
        probe,
        row.get(3),
        gate_failures,
    );
    validate_report_replay_probe_cell(
        "evidence",
        report_sample_list(&probe.evidence),
        probe,
        row.get(4),
        gate_failures,
    );
}

fn validate_report_replay_probe_cell(
    label: &str,
    expected: String,
    probe: &ton_stateflow::ReplayProbeCandidate,
    actual: Option<&String>,
    gate_failures: &mut Vec<String>,
) {
    if actual.is_none_or(|actual| actual != &expected) {
        gate_failures.push(format!(
            "report replay probe {label} {expected} for {} is missing",
            probe.cli_arg
        ));
    }
}

fn report_storage_field_row(
    section: &str,
    opcode: &str,
    field: &ton_stateflow::StorageFieldCandidate,
) -> Option<Vec<String>> {
    section.lines().find_map(|line| {
        let cells = markdown_table_cells(line)?;
        (cells.get(0).is_some_and(|cell| cell == opcode)
            && cells.get(1).is_some_and(|cell| cell == &field.name))
        .then_some(cells)
    })
}

fn validate_report_storage_field_values(
    field: &ton_stateflow::StorageFieldCandidate,
    row: &[String],
    gate_failures: &mut Vec<String>,
) {
    validate_report_storage_field_cell(
        "cell",
        field.cell_path.clone(),
        &field.name,
        row.get(2),
        gate_failures,
    );
    validate_report_storage_field_cell(
        "offset",
        field.bit_offset.to_string(),
        &field.name,
        row.get(3),
        gate_failures,
    );
    validate_report_storage_field_cell(
        "bits",
        report_field_range(field.min_bits, field.max_bits),
        &field.name,
        row.get(4),
        gate_failures,
    );
    validate_report_storage_field_cell(
        "refs",
        report_field_range(field.min_refs, field.max_refs),
        &field.name,
        row.get(5),
        gate_failures,
    );
    validate_report_storage_field_cell(
        "kind",
        field.kind.clone(),
        &field.name,
        row.get(6),
        gate_failures,
    );
    validate_report_storage_field_cell(
        "samples",
        report_sample_list(&field.value_samples),
        &field.name,
        row.get(7),
        gate_failures,
    );
    validate_report_storage_field_cell(
        "confidence",
        field.confidence.clone(),
        &field.name,
        row.get(8),
        gate_failures,
    );
}

fn validate_report_storage_field_cell(
    label: &str,
    expected: String,
    field_name: &str,
    actual: Option<&String>,
    gate_failures: &mut Vec<String>,
) {
    if actual.is_none_or(|actual| actual != &expected) {
        gate_failures.push(format!(
            "report storage field {label} {expected} for {field_name} is missing"
        ));
    }
}

fn report_range<T>(min: T, max: T) -> String
where
    T: Eq + std::fmt::Display,
{
    if min == max {
        min.to_string()
    } else {
        format!("{min}-{max}")
    }
}

fn report_field_range<T>(min: T, max: T) -> String
where
    T: std::fmt::Display,
{
    format!("{min}..{max}")
}

fn report_sample_list(samples: &[String]) -> String {
    if samples.is_empty() {
        return "<none>".to_owned();
    }
    samples.join(", ")
}

fn markdown_table_cells(line: &str) -> Option<Vec<String>> {
    let line = line.trim();
    if !line.starts_with('|') || !line.ends_with('|') {
        return None;
    }
    let cells = line
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().replace('`', ""))
        .collect::<Vec<_>>();
    (!cells
        .iter()
        .all(|cell| cell.chars().all(|ch| ch == '-' || ch == ':')))
    .then_some(cells)
}

fn markdown_line_exists(markdown: &str, expected: &str) -> bool {
    markdown.lines().any(|line| line.trim_end() == expected)
}

fn markdown_section<'a>(markdown: &'a str, heading: &str) -> Option<&'a str> {
    let start = markdown.find(heading)?;
    let after_heading = start + heading.len();
    let remaining = &markdown[after_heading..];
    let next_heading = remaining.find("\n## ").or_else(|| remaining.find("\n# "));
    Some(match next_heading {
        Some(end) => &remaining[..end],
        None => remaining,
    })
}

fn report_replay_mutation_label(mutation: &ReplayMutation) -> String {
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

fn validate_corpus_hash_membership(
    label: &str,
    hash: &str,
    corpus_hashes: &[&str],
    gate_failures: &mut Vec<String>,
) {
    if !corpus_hashes.iter().any(|known| known == &hash) {
        gate_failures.push(format!(
            "{label} {hash} is not present in corpus transactions"
        ));
    }
}

fn validate_manifest_transaction_membership(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    corpus: Option<&StateFlowCorpus>,
    gate_failures: &mut Vec<String>,
) {
    let Some(corpus) = corpus else {
        return;
    };
    let corpus_hashes = corpus_transaction_hashes(corpus);
    if let Some(flow) =
        read_single_target_json_artifact::<StateFlowTx>(manifest_path, artifacts, "transaction")
    {
        if !corpus_hashes.iter().any(|hash| hash == &flow.query_hash) {
            gate_failures.push(format!(
                "transaction query hash {} is not present in corpus transactions",
                flow.query_hash
            ));
        }
    }
}

fn validate_manifest_replay_membership(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    corpus: Option<&StateFlowCorpus>,
    gate_failures: &mut Vec<String>,
) {
    let Some(corpus) = corpus else {
        return;
    };
    let corpus_hashes = corpus_transaction_hashes(corpus);
    let replays =
        read_target_json_artifacts::<StateFlowReplayDiff>(manifest_path, artifacts, "replay");
    if !replays.is_empty()
        && !replays
            .iter()
            .any(|replay| !matches!(replay.mutation, ReplayMutation::None))
    {
        gate_failures.push("replay artifacts must include at least one mutation".to_owned());
    }
    if !replays.is_empty()
        && !replays
            .iter()
            .any(|replay| replay_has_observable_diff(replay))
    {
        gate_failures.push("replay artifacts must include an observable diff".to_owned());
    }
    for replay in replays {
        if !matches!(replay.mutation, ReplayMutation::None) && !replay.diff.input_changed {
            gate_failures.push(format!(
                "mutated replay {} must report inputChanged true",
                replay.source_query_hash
            ));
        }
        if !corpus_hashes
            .iter()
            .any(|hash| hash == &replay.source_query_hash)
        {
            gate_failures.push(format!(
                "replay source query hash {} is not present in corpus transactions",
                replay.source_query_hash
            ));
        }
    }
}

fn replay_has_observable_diff(replay: &StateFlowReplayDiff) -> bool {
    replay.diff.state_changed == Some(true)
        || replay.diff.code_hash_changed == Some(true)
        || replay.diff.data_hash_changed == Some(true)
        || replay.diff.balance_delta_diff.is_some_and(|diff| diff != 0)
        || replay.diff.exit_code_changed == Some(true)
        || replay
            .diff
            .outbound_count_delta
            .is_some_and(|diff| diff != 0)
        || replay.diff.action_count_delta.is_some_and(|diff| diff != 0)
        || replay.diff.c5_changed == Some(true)
}

fn corpus_transaction_hashes(corpus: &StateFlowCorpus) -> Vec<&str> {
    corpus
        .transactions
        .iter()
        .map(|transaction| transaction.query_hash.as_str())
        .collect()
}

fn read_single_target_json_artifact<T>(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    kind: &str,
) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let matches = artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .copied()
        .collect::<Vec<_>>();
    let [artifact] = matches.as_slice() else {
        return None;
    };
    let path = resolve_manifest_artifact_path(manifest_path, &artifact.path);
    let json = fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

fn read_single_target_text_artifact(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    kind: &str,
) -> Option<String> {
    let matches = artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .copied()
        .collect::<Vec<_>>();
    let [artifact] = matches.as_slice() else {
        return None;
    };
    let path = resolve_manifest_artifact_path(manifest_path, &artifact.path);
    fs::read_to_string(path).ok()
}

fn read_target_json_artifacts<T>(
    manifest_path: &Path,
    artifacts: &[&SmokeArtifactManifestEntry],
    kind: &str,
) -> Vec<T>
where
    T: for<'de> Deserialize<'de>,
{
    artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .filter_map(|artifact| {
            let path = resolve_manifest_artifact_path(manifest_path, &artifact.path);
            let json = fs::read_to_string(path).ok()?;
            serde_json::from_str(&json).ok()
        })
        .collect()
}

fn validate_target_text_field(
    actual_label: &str,
    actual: &str,
    expected_label: &str,
    expected: &str,
    gate_failures: &mut Vec<String>,
) {
    if actual != expected {
        gate_failures.push(format!(
            "{actual_label} {actual} does not match {expected_label} {expected}"
        ));
    }
}

fn validate_target_usize_field(
    actual_label: &str,
    actual: usize,
    expected_label: &str,
    expected: usize,
    gate_failures: &mut Vec<String>,
) {
    if actual != expected {
        gate_failures.push(format!(
            "{actual_label} {actual} does not match {expected_label} {expected}"
        ));
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
        write_sample_validation_artifacts(temp_dir.path());

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

        let validation = fs::read_to_string(temp_dir.path().join("validation.json"))
            .expect("validation artifact should be written");
        let validation_json: serde_json::Value =
            serde_json::from_str(&validation).expect("validation should be valid JSON");
        assert_eq!(
            validation_json["kind"],
            "stateFlowArtifactManifestValidation"
        );
        assert_eq!(validation_json["passed"], true);
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
        assert_eq!(json["artifacts"].as_array().unwrap().len(), 8);
        assert_eq!(
            json["artifacts"],
            serde_json::json!([
                {"kind": "runSummary", "path": "summary.json", "targetId": null},
                {"kind": "corpus", "path": "target-a/corpus.json", "targetId": "target-a"},
                {"kind": "schema", "path": "target-a/schema.json", "targetId": "target-a"},
                {"kind": "transaction", "path": "target-a/transaction-0.json", "targetId": "target-a"},
                {"kind": "replay", "path": "target-a/replay.json", "targetId": "target-a"},
                {"kind": "replay", "path": "target-a/replay-query-id.json", "targetId": "target-a"},
                {"kind": "report", "path": "target-a/report.md", "targetId": "target-a"},
                {"kind": "validation", "path": "validation.json", "targetId": null}
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
        assert_eq!(json["artifacts"].as_array().unwrap().len(), 6);
        assert_eq!(json["artifacts"][5]["kind"], "validation");
        assert_eq!(json["artifacts"][5]["path"], "validation.json");
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
        write_sample_validation_artifacts(temp_dir.path());
        let manifest = sample_validation_manifest();

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
        write_sample_validation_artifacts(temp_dir.path());
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
    fn artifact_manifest_validation_rejects_invalid_corpus_json() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        fs::write(temp_dir.path().join("target-a/corpus.json"), "{}")
            .expect("invalid corpus should be written");
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("invalid corpus artifact target-a/corpus.json")
            })
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_schema_content_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "other-addr",
                "transactionCount": 2,
                "stateMachine": {"edges": []},
                "auditSignals": [],
                "opcodeCandidates": []
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: schema address other-addr does not match summary address addr",
                )
            }),
            "expected schema content mismatch failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_schema_metric_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "transactionCount": 2,
                "stateMachine": {"edges": []},
                "auditSignals": [],
                "opcodeCandidates": []
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: schema opcode candidate count 0 does not match summary opcode candidate count 1",
                )
            }),
            "expected schema metric mismatch failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_schema_evidence_outside_corpus() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "transactionCount": 2,
                "opcodeCandidates": [{
                    "opcode": "0x00000001",
                    "count": 2,
                    "examples": ["tx-a"],
                    "evidence": [{
                        "txHash": "foreign-tx",
                        "inboundBodyHash": "body",
                        "inboundBodyBits": 32,
                        "inboundBodyRefs": 0,
                        "fromStatus": "active",
                        "toStatus": "active",
                        "preDataHash": null,
                        "postDataHash": null,
                        "preCodeHash": null,
                        "postCodeHash": null,
                        "outboundKinds": [],
                        "outActionKinds": []
                    }],
                    "inboundBody": {
                        "minBits": 32,
                        "maxBits": 32,
                        "minRefs": 0,
                        "maxRefs": 0,
                        "bodyHashes": []
                    },
                    "stateTransitions": [],
                    "outboundEffects": [],
                    "outActions": [],
                    "confidence": "medium",
                    "unknownFields": []
                }]
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: schema evidence tx hash foreign-tx is not present in corpus transactions",
                )
            }),
            "expected schema evidence membership failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_target_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown("other-addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report target line \"- Address: `addr`\" is missing")
            }),
            "expected report target mismatch failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_opcode_candidate_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_wrong_opcode_candidate("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure
                    .contains("target-a: report opcode candidate count 2 for 0x00000001 is missing")
            }),
            "expected report opcode count failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report opcode candidate body bits 32 for 0x00000001 is missing",
                )
            }),
            "expected report opcode body bits failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report opcode candidate evidence tx-a, tx-b for 0x00000001 is missing")
            }),
            "expected report opcode evidence failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_accepts_report_opcode_candidate_range_format() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        let schema_path = temp_dir.path().join("target-a/schema.json");
        let mut schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&schema_path).expect("schema should exist"))
                .expect("schema should parse");
        schema["opcodeCandidates"][0]["inboundBody"]["maxBits"] = serde_json::json!(40);
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &schema.to_string(),
        );
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_opcode_candidate_range("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(
            validation.passed,
            "expected renderer range format to pass, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_message_body_field_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        let schema_path = temp_dir.path().join("target-a/schema.json");
        let mut schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&schema_path).expect("schema should exist"))
                .expect("schema should parse");
        schema["opcodeCandidates"][0]["inboundBody"]["fieldCandidates"] = serde_json::json!([{
            "name": "query_id",
            "bitOffset": 32,
            "minBits": 64,
            "maxBits": 64,
            "minRefs": 0,
            "maxRefs": 0,
            "kind": "uint64",
            "presentCount": 2,
            "valueSamples": ["0x7"],
            "confidence": "high"
        }]);
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &schema.to_string(),
        );
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_wrong_message_body_field("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report message body field offset 32 for query_id is missing",
                )
            }),
            "expected report message body offset failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report message body field bits 64..64 for query_id is missing",
                )
            }),
            "expected report message body bits failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report message body field samples 0x7 for query_id is missing",
                )
            }),
            "expected report message body samples failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_replay_probe_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        let schema_path = temp_dir.path().join("target-a/schema.json");
        let mut schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&schema_path).expect("schema should exist"))
                .expect("schema should parse");
        schema["opcodeCandidates"][0]["replayProbes"] = serde_json::json!([{
            "fieldName": "query_id",
            "bitOffset": 32,
            "bits": 64,
            "value": "0x6",
            "mutation": {"type": "flipBodyBit", "bit": 0},
            "cliArg": "--flip-body-bit 0",
            "confidence": "high",
            "evidence": ["tx-a"]
        }]);
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &schema.to_string(),
        );
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_wrong_replay_probe("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report replay probe confidence high for --flip-body-bit 0 is missing",
                )
            }),
            "expected report replay probe confidence failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report replay probe evidence tx-a for --flip-body-bit 0 is missing",
                )
            }),
            "expected report replay probe evidence failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_storage_field_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        let schema_path = temp_dir.path().join("target-a/schema.json");
        let mut schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&schema_path).expect("schema should exist"))
                .expect("schema should parse");
        schema["opcodeCandidates"][0]["storage"] = serde_json::json!({
            "balanceDeltaMin": 0,
            "balanceDeltaMax": 0,
            "dataHashChangedCount": 1,
            "codeHashChangedCount": 0,
            "fields": [{
                "name": "data_word_0",
                "cellPath": "data",
                "bitOffset": 0,
                "minBits": 32,
                "maxBits": 32,
                "minRefs": 0,
                "maxRefs": 0,
                "kind": "uint32",
                "presentCount": 2,
                "valueSamples": ["0xdeadbeef"],
                "confidence": "medium"
            }],
            "postDataHashes": ["data"],
            "postCodeHashes": []
        });
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &schema.to_string(),
        );
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_wrong_storage_field("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure
                    .contains("target-a: report storage field cell data for data_word_0 is missing")
            }),
            "expected report storage cell failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report storage field samples 0xdeadbeef for data_word_0 is missing",
                )
            }),
            "expected report storage samples failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_missing_schema_evidence() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_without_schema_evidence("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report schema evidence tx hash tx-a is missing")
            }),
            "expected report schema evidence failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_schema_evidence_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_wrong_schema_evidence("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report schema evidence body hash body for tx tx-a is missing",
                )
            }),
            "expected report schema evidence body failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report schema evidence state none -> active for tx tx-a is missing",
                )
            }),
            "expected report schema evidence state failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_schema_evidence_data_code_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        let schema_path = temp_dir.path().join("target-a/schema.json");
        let mut schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&schema_path).expect("schema should exist"))
                .expect("schema should parse");
        schema["opcodeCandidates"][0]["evidence"][0]["preDataHash"] = serde_json::json!("old-data");
        schema["opcodeCandidates"][0]["evidence"][0]["postDataHash"] =
            serde_json::json!("new-data");
        schema["opcodeCandidates"][0]["evidence"][0]["preCodeHash"] = serde_json::json!("old-code");
        schema["opcodeCandidates"][0]["evidence"][0]["postCodeHash"] =
            serde_json::json!("new-code");
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &schema.to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report schema evidence data hash old-data -> new-data for tx tx-a is missing",
                )
            }),
            "expected report schema evidence data hash failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report schema evidence code hash old-code -> new-code for tx tx-a is missing",
                )
            }),
            "expected report schema evidence code hash failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_missing_replay_mutation() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_without_replay_mutation("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report replay mutation \"flip body bit 0\" for tx tx-a is missing",
                )
            }),
            "expected report replay mutation failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_replay_diff_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_with_wrong_replay_diff("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure
                    .contains("target-a: report replay input changed true for tx tx-a is missing")
            }),
            "expected report replay input-changed failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_report_missing_schema_deliverables() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "transactionCount": 2,
                "stateMachine": {
                    "edges": [{
                        "fromStatus": "none",
                        "toStatus": "active",
                        "opcode": "0x00000001",
                        "count": 2,
                        "examples": ["tx-a", "tx-b"]
                    }]
                },
                "auditSignals": [{
                    "kind": "unknown-fields",
                    "severity": "info",
                    "description": "Unknown fields remain.",
                    "evidence": ["tx-a"]
                }],
                "opcodeCandidates": [{
                    "opcode": "0x00000001",
                    "count": 2,
                    "examples": ["tx-a", "tx-b"],
                    "evidence": [{
                        "txHash": "tx-a",
                        "inboundBodyHash": "body",
                        "inboundBodyBits": 32,
                        "inboundBodyRefs": 0,
                        "fromStatus": "none",
                        "toStatus": "active",
                        "preDataHash": null,
                        "postDataHash": null,
                        "preCodeHash": null,
                        "postCodeHash": null,
                        "outboundKinds": ["internal"],
                        "outActionKinds": ["send-message"]
                    }],
                    "inboundBody": {
                        "minBits": 32,
                        "maxBits": 96,
                        "minRefs": 0,
                        "maxRefs": 0,
                        "bodyHashes": ["body"],
                        "fieldCandidates": [{
                            "name": "query_id",
                            "bitOffset": 32,
                            "minBits": 64,
                            "maxBits": 64,
                            "minRefs": 0,
                            "maxRefs": 0,
                            "kind": "uint64",
                            "presentCount": 2,
                            "valueSamples": ["0x7"],
                            "confidence": "high"
                        }]
                    },
                    "replayProbes": [{
                        "fieldName": "query_id",
                        "bitOffset": 32,
                        "bits": 64,
                        "value": "0x6",
                        "mutation": {"type": "flipBodyBit", "bit": 0},
                        "cliArg": "--flip-body-bit 0",
                        "confidence": "high",
                        "evidence": ["tx-a"]
                    }],
                    "storage": {
                        "balanceDeltaMin": 0,
                        "balanceDeltaMax": 0,
                        "dataHashChangedCount": 1,
                        "codeHashChangedCount": 0,
                        "fields": [{
                            "name": "data_word_0",
                            "cellPath": "data",
                            "bitOffset": 0,
                            "minBits": 32,
                            "maxBits": 32,
                            "minRefs": 0,
                            "maxRefs": 0,
                            "kind": "uint32",
                            "presentCount": 2,
                            "valueSamples": ["0xdeadbeef"],
                            "confidence": "medium"
                        }],
                        "postDataHashes": ["data"],
                        "postCodeHashes": []
                    },
                    "stateTransitions": [{
                        "fromStatus": "none",
                        "toStatus": "active",
                        "count": 2
                    }],
                    "outboundEffects": [{
                        "kind": "internal",
                        "count": 1,
                        "txHashes": ["tx-a"],
                        "modes": [],
                        "destinations": ["dst"],
                        "valueNanotonsMin": "11",
                        "valueNanotonsMax": "11",
                        "bodyShape": {
                            "minBits": 40,
                            "maxBits": 40,
                            "minRefs": 1,
                            "maxRefs": 1
                        },
                        "codeShape": null,
                        "libraryHashes": []
                    }],
                    "outActions": [{
                        "kind": "send-message",
                        "count": 1,
                        "txHashes": ["tx-a"],
                        "modes": ["64"],
                        "destinations": ["dst"],
                        "valueNanotonsMin": "7",
                        "valueNanotonsMax": "7",
                        "bodyShape": {
                            "minBits": 32,
                            "maxBits": 32,
                            "minRefs": 0,
                            "maxRefs": 0
                        },
                        "codeShape": null,
                        "libraryHashes": []
                    }],
                    "confidence": "medium",
                    "unknownFields": ["message body field names require TL-B recovery"]
                }]
            })
            .to_string(),
        );
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/report.md",
            &sample_report_markdown_without_schema_deliverables("addr"),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report message body field query_id is missing")
            }),
            "expected report message body field failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report storage field data_word_0 is missing")
            }),
            "expected report storage field failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report unknown field message body field names require TL-B recovery is missing",
                )
            }),
            "expected report unknown field failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report outbound effect outbound internal is missing")
            }),
            "expected report outbound effect failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: report state edge none --> active: 0x00000001 (2) is missing",
                )
            }),
            "expected report state edge failure, got {:?}",
            validation.gate_failures
        );
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: report risk \"Unknown fields remain.\" is missing")
            }),
            "expected report risk failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_schema_probe_without_replay() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "transactionCount": 2,
                "stateMachine": {
                    "edges": [{
                        "fromStatus": "none",
                        "toStatus": "active",
                        "opcode": "0x00000001",
                        "count": 2,
                        "examples": ["tx-a", "tx-b"]
                    }]
                },
                "auditSignals": [{
                    "kind": "unknown-fields",
                    "severity": "info",
                    "description": "Unknown fields remain.",
                    "evidence": ["tx-a"]
                }],
                "opcodeCandidates": [{
                    "opcode": "0x00000001",
                    "count": 2,
                    "examples": ["tx-a", "tx-b"],
                    "evidence": [{
                        "txHash": "tx-a",
                        "inboundBodyHash": "body",
                        "inboundBodyBits": 32,
                        "inboundBodyRefs": 0,
                        "fromStatus": "none",
                        "toStatus": "active",
                        "preDataHash": null,
                        "postDataHash": null,
                        "preCodeHash": null,
                        "postCodeHash": null,
                        "outboundKinds": [],
                        "outActionKinds": []
                    }],
                    "inboundBody": {
                        "minBits": 32,
                        "maxBits": 32,
                        "minRefs": 0,
                        "maxRefs": 0,
                        "bodyHashes": []
                    },
                    "replayProbes": [{
                        "fieldName": "query_id",
                        "bitOffset": 32,
                        "bits": 64,
                        "value": "42",
                        "mutation": {
                            "type": "setBodyUint",
                            "bitOffset": 32,
                            "bits": 64,
                            "value": "42"
                        },
                        "cliArg": "--set-body-uint 32:64:42",
                        "confidence": "high",
                        "evidence": ["tx-a"]
                    }],
                    "stateTransitions": [],
                    "outboundEffects": [],
                    "outActions": [],
                    "confidence": "medium",
                    "unknownFields": []
                }]
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: schema replay probe --set-body-uint 32:64:42 has no matching replay artifact",
                )
            }),
            "expected schema replay probe artifact failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_schema_probe_replayed_from_wrong_source() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "transactionCount": 2,
                "stateMachine": {
                    "edges": [{
                        "fromStatus": "none",
                        "toStatus": "active",
                        "opcode": "0x00000001",
                        "count": 2,
                        "examples": ["tx-a", "tx-b"]
                    }]
                },
                "auditSignals": [{
                    "kind": "unknown-fields",
                    "severity": "info",
                    "description": "Unknown fields remain.",
                    "evidence": ["tx-a"]
                }],
                "opcodeCandidates": [{
                    "opcode": "0x00000001",
                    "count": 2,
                    "examples": ["tx-a", "tx-b"],
                    "evidence": [{
                        "txHash": "tx-a",
                        "inboundBodyHash": "body",
                        "inboundBodyBits": 32,
                        "inboundBodyRefs": 0,
                        "fromStatus": "none",
                        "toStatus": "active",
                        "preDataHash": null,
                        "postDataHash": null,
                        "preCodeHash": null,
                        "postCodeHash": null,
                        "outboundKinds": [],
                        "outActionKinds": []
                    }],
                    "inboundBody": {
                        "minBits": 32,
                        "maxBits": 32,
                        "minRefs": 0,
                        "maxRefs": 0,
                        "bodyHashes": []
                    },
                    "replayProbes": [{
                        "fieldName": "query_id",
                        "bitOffset": 32,
                        "bits": 64,
                        "value": "42",
                        "mutation": {
                            "type": "setBodyUint",
                            "bitOffset": 32,
                            "bits": 64,
                            "value": "42"
                        },
                        "cliArg": "--set-body-uint 32:64:42",
                        "confidence": "high",
                        "evidence": ["tx-b"]
                    }],
                    "stateTransitions": [],
                    "outboundEffects": [],
                    "outActions": [],
                    "confidence": "medium",
                    "unknownFields": []
                }]
            })
            .to_string(),
        );
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/replay.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "sourceQueryHash": "tx-a",
                "mutation": {
                    "type": "setBodyUint",
                    "bitOffset": 32,
                    "bits": 64,
                    "value": "42"
                },
                "ignoreChksig": false,
                "baseline": sample_replay_observation_json(true),
                "replay": sample_replay_observation_json(true),
                "diff": {
                    "replayAccepted": true,
                    "inputChanged": true,
                    "stateChanged": false,
                    "codeHashChanged": false,
                    "dataHashChanged": false,
                    "balanceDeltaDiff": 0,
                    "exitCodeChanged": false,
                    "outboundCountDelta": 0,
                    "actionCountDelta": 0,
                    "c5Changed": true
                }
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: schema replay probe --set-body-uint 32:64:42 has no replay artifact for evidence source",
                )
            }),
            "expected schema replay probe source failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_stale_validation_artifact() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "validation.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "kind": "stateFlowArtifactManifestValidation",
                "manifest": "artifacts.json",
                "targetCount": 99,
                "absolutePathCount": 0,
                "expectedAbsolutePathCount": 0,
                "passed": true,
                "gateFailures": [],
                "targets": []
            })
            .to_string(),
        );
        let mut manifest = sample_validation_manifest();
        manifest
            .artifacts
            .push(super::SmokeArtifactManifestEntry::new(
                "validation",
                "validation.json",
                None,
            ));

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure
                    .contains("validation target count 99 does not match expected target count 1")
            }),
            "expected stale validation artifact failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_corpus_count_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/corpus.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "requestedLimit": 2,
                "sourceTxCount": 2,
                "retracedCount": 2,
                "failureCount": 0,
                "opcodeSummary": [],
                "transactions": [sample_state_flow_json("tx-a")],
                "failures": []
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: corpus retraced count 2 does not match transaction list length 1",
                )
            }),
            "expected corpus count mismatch failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_replay_outside_corpus() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/replay.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "sourceQueryHash": "foreign-tx",
                "mutation": {"type": "none"},
                "ignoreChksig": false,
                "baseline": sample_replay_observation_json(true),
                "replay": sample_replay_observation_json(true),
                "diff": {
                    "replayAccepted": true,
                    "inputChanged": false,
                    "stateChanged": false,
                    "codeHashChanged": false,
                    "dataHashChanged": false,
                    "balanceDeltaDiff": 0,
                    "exitCodeChanged": false,
                    "outboundCountDelta": 0,
                    "actionCountDelta": 0,
                    "c5Changed": false
                }
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: replay source query hash foreign-tx is not present in corpus transactions",
                )
            }),
            "expected replay corpus membership failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_unmutated_replay_bundle() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/replay.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "sourceQueryHash": "tx-a",
                "mutation": {"type": "none"},
                "ignoreChksig": false,
                "baseline": sample_replay_observation_json(true),
                "replay": sample_replay_observation_json(true),
                "diff": {
                    "replayAccepted": true,
                    "inputChanged": false,
                    "stateChanged": false,
                    "codeHashChanged": false,
                    "dataHashChanged": false,
                    "balanceDeltaDiff": 0,
                    "exitCodeChanged": false,
                    "outboundCountDelta": 0,
                    "actionCountDelta": 0,
                    "c5Changed": false
                }
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: replay artifacts must include at least one mutation")
            }),
            "expected unmutated replay bundle failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_mutated_replay_without_input_diff() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/replay.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "sourceQueryHash": "tx-a",
                "mutation": {"type": "flipBodyBit", "bit": 0},
                "ignoreChksig": false,
                "baseline": sample_replay_observation_json(true),
                "replay": sample_replay_observation_json(true),
                "diff": {
                    "replayAccepted": true,
                    "inputChanged": false,
                    "stateChanged": false,
                    "codeHashChanged": false,
                    "dataHashChanged": false,
                    "balanceDeltaDiff": 0,
                    "exitCodeChanged": false,
                    "outboundCountDelta": 0,
                    "actionCountDelta": 0,
                    "c5Changed": false
                }
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: mutated replay tx-a must report inputChanged true")
            }),
            "expected mutated replay input diff failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_input_only_replay_diff() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        write_sample_validation_artifact(
            temp_dir.path(),
            "target-a/replay.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "sourceQueryHash": "tx-a",
                "mutation": {"type": "flipBodyBit", "bit": 0},
                "ignoreChksig": false,
                "baseline": sample_replay_observation_json(true),
                "replay": sample_replay_observation_json(true),
                "diff": {
                    "replayAccepted": true,
                    "inputChanged": true,
                    "stateChanged": false,
                    "codeHashChanged": false,
                    "dataHashChanged": false,
                    "balanceDeltaDiff": 0,
                    "exitCodeChanged": false,
                    "outboundCountDelta": 0,
                    "actionCountDelta": 0,
                    "c5Changed": false
                }
            })
            .to_string(),
        );
        let manifest = sample_validation_manifest();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains("target-a: replay artifacts must include an observable diff")
            }),
            "expected input-only replay diff failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_summary_path_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        fs::copy(
            temp_dir.path().join("target-a/corpus.json"),
            temp_dir.path().join("target-a/alternate-corpus.json"),
        )
        .expect("alternate corpus should be written");
        let mut manifest = sample_validation_manifest();
        manifest
            .artifacts
            .iter_mut()
            .find(|artifact| {
                artifact.target_id.as_deref() == Some("target-a") && artifact.kind == "corpus"
            })
            .expect("corpus artifact should exist")
            .path = "target-a/alternate-corpus.json".to_owned();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "target-a: corpus artifact path target-a/alternate-corpus.json does not match summary path target-a/corpus.json",
                )
            }),
            "expected summary path mismatch failure, got {:?}",
            validation.gate_failures
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_run_summary_entry_mismatch() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        write_sample_validation_artifacts(temp_dir.path());
        fs::copy(
            temp_dir.path().join("summary.json"),
            temp_dir.path().join("alternate-summary.json"),
        )
        .expect("alternate summary should be written");
        let mut manifest = sample_validation_manifest();
        manifest.summary = "alternate-summary.json".to_owned();

        let validation = super::validate_artifact_manifest_bundle(
            &manifest,
            &temp_dir.path().join("artifacts.json"),
            None,
        )
        .expect("manifest validation should run");

        assert!(!validation.passed);
        assert!(
            validation.gate_failures.iter().any(|failure| {
                failure.contains(
                    "runSummary artifact path summary.json does not match manifest summary alternate-summary.json",
                )
            }),
            "expected run summary entry mismatch failure, got {:?}",
            validation.gate_failures
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

    fn sample_validation_manifest() -> super::SmokeArtifactManifest {
        serde_json::from_value(serde_json::json!({
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
        .expect("artifact manifest should deserialize")
    }

    fn write_sample_validation_artifacts(out_dir: &Path) {
        let summary = sample_smoke_summary().with_paths_relative_to(Path::new("out"));
        write_sample_validation_artifact(
            out_dir,
            "summary.json",
            &serde_json::to_string(&summary).expect("summary should serialize"),
        );
        write_sample_validation_artifact(
            out_dir,
            "target-a/corpus.json",
            &sample_replay_corpus_json(),
        );
        write_sample_validation_artifact(
            out_dir,
            "target-a/schema.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "network": "mainnet",
                "address": "addr",
                "transactionCount": 2,
                "stateMachine": {
                    "edges": [{
                        "fromStatus": "none",
                        "toStatus": "active",
                        "opcode": "0x00000001",
                        "count": 2,
                        "examples": ["tx-a", "tx-b"]
                    }]
                },
                "auditSignals": [{
                    "kind": "unknown-fields",
                    "severity": "info",
                    "description": "Unknown fields remain.",
                    "evidence": ["tx-a"]
                }],
                "opcodeCandidates": [{
                    "opcode": "0x00000001",
                    "count": 2,
                    "examples": ["tx-a", "tx-b"],
                    "evidence": [{
                        "txHash": "tx-a",
                        "inboundBodyHash": "body",
                        "inboundBodyBits": 32,
                        "inboundBodyRefs": 0,
                        "fromStatus": "none",
                        "toStatus": "active",
                        "preDataHash": null,
                        "postDataHash": null,
                        "preCodeHash": null,
                        "postCodeHash": null,
                        "outboundKinds": [],
                        "outActionKinds": []
                    }],
                    "inboundBody": {
                        "minBits": 32,
                        "maxBits": 32,
                        "minRefs": 0,
                        "maxRefs": 0,
                        "bodyHashes": []
                    },
                    "stateTransitions": [],
                    "outboundEffects": [],
                    "outActions": [],
                    "confidence": "medium",
                    "unknownFields": []
                }]
            })
            .to_string(),
        );
        write_sample_validation_artifact(
            out_dir,
            "target-a/transaction-0.json",
            &sample_state_flow_json("tx-a").to_string(),
        );
        write_sample_validation_artifact(
            out_dir,
            "target-a/replay.json",
            &serde_json::json!({
                "schemaVersion": 1,
                "sourceQueryHash": "tx-a",
                "mutation": {"type": "flipBodyBit", "bit": 0},
                "ignoreChksig": false,
                "baseline": sample_replay_observation_json(true),
                "replay": sample_replay_observation_json(true),
                "diff": {
                    "replayAccepted": true,
                    "inputChanged": true,
                    "stateChanged": false,
                    "codeHashChanged": false,
                    "dataHashChanged": false,
                    "balanceDeltaDiff": 0,
                    "exitCodeChanged": false,
                    "outboundCountDelta": 0,
                    "actionCountDelta": 0,
                    "c5Changed": true
                }
            })
            .to_string(),
        );
        write_sample_validation_artifact(
            out_dir,
            "target-a/report.md",
            &sample_report_markdown("addr"),
        );
    }

    fn write_sample_validation_artifact(out_dir: &Path, path: &str, contents: &str) {
        let path = out_dir.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("parent dir should be created");
        fs::write(path, contents).expect("artifact should be written");
    }

    fn sample_report_markdown(address: &str) -> String {
        sample_report_markdown_inner(address, true, "flip body bit 0", true, true)
    }

    fn sample_report_markdown_without_schema_evidence(address: &str) -> String {
        sample_report_markdown_inner(address, false, "flip body bit 0", true, true)
    }

    fn sample_report_markdown_with_wrong_opcode_candidate(address: &str) -> String {
        sample_report_markdown(address).replace(
            "| `0x00000001` | 2 | medium | 32 | 0 | balance 0; data hash changes 0; code hash changes 0 | none -> active (2) | none | none | tx-a, tx-b |",
            "| `0x00000001` | 9 | medium | 16 | 1 | balance 0 | none | none | none | foreign-tx |",
        )
    }

    fn sample_report_markdown_with_opcode_candidate_range(address: &str) -> String {
        sample_report_markdown(address).replace(
            "| `0x00000001` | 2 | medium | 32 | 0 | balance 0; data hash changes 0; code hash changes 0 | none -> active (2) | none | none | tx-a, tx-b |",
            "| `0x00000001` | 2 | medium | 32-40 | 0 | balance 0; data hash changes 0; code hash changes 0 | none -> active (2) | none | none | tx-a, tx-b |",
        )
    }

    fn sample_report_markdown_with_wrong_message_body_field(address: &str) -> String {
        sample_report_markdown(address).replace(
            "## Message Body Fields\n- No message body field candidates were inferred.",
            "## Message Body Fields\n| Opcode | Field | Offset | Bits | Refs | Kind | Samples | Confidence |\n| --- | --- | ---: | --- | --- | --- | --- | --- |\n| `0x00000001` | `query_id` | 0 | 32..32 | 1..1 | raw | `0xff` | high |",
        )
    }

    fn sample_report_markdown_with_wrong_replay_probe(address: &str) -> String {
        sample_report_markdown(address).replace(
            "## Replay Probes\n- No replay probe candidates were inferred.",
            "## Replay Probes\n| Opcode | Field | CLI mutation | Confidence | Evidence |\n| --- | --- | --- | --- | --- |\n| `0x00000001` | `query_id` | `--flip-body-bit 0` | low | `tx-b` |",
        )
    }

    fn sample_report_markdown_with_wrong_storage_field(address: &str) -> String {
        sample_report_markdown(address).replace(
            "## Storage Fields\n- No storage field candidates were inferred.",
            "## Storage Fields\n| Opcode | Field | Cell | Offset | Bits | Refs | Kind | Samples | Confidence |\n| --- | --- | --- | ---: | --- | --- | --- | --- | --- |\n| `0x00000001` | `data_word_0` | code | 8 | 16..16 | 1..1 | raw | `0xff` | medium |",
        )
    }

    fn sample_report_markdown_with_wrong_schema_evidence(address: &str) -> String {
        sample_report_markdown(address).replace(
            "| `0x00000001` | `tx-a` | `body` | 32/0 | none -> active | `<none>` -> `<none>` | `<none>` -> `<none>` | none | none |",
            "| `0x00000001` | `tx-a` | `wrong-body` | 16/1 | active -> none | n/a | n/a | outbound | action |",
        )
    }

    fn sample_report_markdown_without_replay_mutation(address: &str) -> String {
        sample_report_markdown_inner(address, true, "none", true, true)
    }

    fn sample_report_markdown_with_wrong_replay_diff(address: &str) -> String {
        sample_report_markdown(address).replace(
            "| `tx-a` | flip body bit 0 | true | true | false | false | 0 | 0 |",
            "| `tx-a` | flip body bit 0 | true | false | false | true | 99 | 99 |",
        )
    }

    fn sample_report_markdown_without_schema_deliverables(address: &str) -> String {
        sample_report_markdown_inner(address, true, "flip body bit 0", false, false)
    }

    fn sample_report_markdown_inner(
        address: &str,
        include_schema_evidence: bool,
        replay_mutation: &str,
        include_schema_summary_rows: bool,
        include_risk_point: bool,
    ) -> String {
        let opcode_candidate_row = if include_schema_summary_rows {
            "| `0x00000001` | 2 | medium | 32 | 0 | balance 0; data hash changes 0; code hash changes 0 | none -> active (2) | none | none | tx-a, tx-b |\n"
        } else {
            ""
        };
        let schema_evidence_row = if include_schema_evidence {
            "| `0x00000001` | `tx-a` | `body` | 32/0 | none -> active | `<none>` -> `<none>` | `<none>` -> `<none>` | none | none |\n"
        } else {
            ""
        };
        let state_edge = if include_schema_summary_rows {
            "    none --> active: 0x00000001 (2)\n"
        } else {
            ""
        };
        let risk_point = if include_risk_point {
            "- Unknown fields remain. Evidence: `tx-a`.\n"
        } else {
            "- No risk points were inferred from the provided artifacts.\n"
        };
        format!(
            "# TON State Flow Reverse Report\n\
             \n\
             ## Target\n\
             - Network: `mainnet`\n\
             - Address: `{address}`\n\
             - Source transactions: 2\n\
             - Retraced transactions: 2\n\
             - Replay failures while collecting: 0\n\
             \n\
             ## Opcode Candidates\n\
             | Opcode | Count | Confidence | Body bits | Body refs | Storage | State transitions | Outbound effects | Out actions | Evidence |\n\
             | --- | ---: | --- | --- | --- | --- | --- | --- | --- | --- |\n\
             {opcode_candidate_row}\
             \n\
             ## Schema Evidence\n\
             | Opcode | Tx | Body hash | Body bits/refs | State | Data hash | Code hash | Outbound | Actions |\n\
             | --- | --- | --- | ---: | --- | --- | --- | --- | --- |\n\
             {schema_evidence_row}\
             \n\
             ## Message Body Fields\n\
             - No message body field candidates were inferred.\n\
             \n\
             ## Replay Probes\n\
             - No replay probe candidates were inferred.\n\
             \n\
             ## Storage Fields\n\
             - No storage field candidates were inferred.\n\
             \n\
             ## Outbound Effects\n\
             - No outbound effect candidates were inferred.\n\
             \n\
             ## State Machine\n\
             ```mermaid\n\
             stateDiagram-v2\n\
             {state_edge}\
             ```\n\
             \n\
             ## Unknown Fields\n\
             - `0x00000001`:\n\
             \n\
             ## Replay Diffs\n\
             | Source tx | Mutation | Accepted | Input changed | State changed | Exit changed | Outbound delta | Action delta |\n\
             | --- | --- | --- | --- | --- | --- | ---: | ---: |\n\
             | `tx-a` | {replay_mutation} | true | true | false | false | 0 | 0 |\n\
             \n\
             ## Risk Points\n\
             {risk_point}"
        )
    }

    fn sample_replay_observation_json(accepted: bool) -> serde_json::Value {
        serde_json::json!({
            "accepted": accepted,
            "state": null,
            "inbound": sample_state_flow_json("tx-a")["inbound"].clone(),
            "outbound": [],
            "compute": null,
            "money": null,
            "c5": null,
            "outActions": [],
            "vmTrace": null,
            "executorTrace": null,
            "error": null
        })
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
