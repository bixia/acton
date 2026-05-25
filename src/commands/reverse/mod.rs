use anyhow::Context;
use clap::Subcommand;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use ton_retrace::Network;
use ton_stateflow::{
    ReplayMutation, StateFlowCorpus, StateFlowReplayDiff, StateFlowSchemaReport, StateFlowTx,
};

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
