import {
  parseStateFlowArtifact,
  summarizeStateFlowArtifact,
  type StateFlowArtifact,
} from "./stateFlowArtifacts"
import {StateFlowArtifactView} from "./StateFlowArtifactView"
import {StateFlowArtifactWorkbench} from "./StateFlowArtifactWorkbench"

const stateFlowTx = {
  schemaVersion: 1,
  network: "mainnet",
  queryHash: "tx-hash",
  transaction: {
    lt: 42,
    utime: 1,
    account: "account",
    stateUpdateHashOk: true,
    transactionBoc64: "tx",
  },
  replay: {
    mcSeqno: 7,
    randSeedHex: "00",
    replayedPrevTxCount: 0,
    blockConfigBoc64: "config",
  },
  state: {
    pre: {
      shardAccountBoc64: "pre",
      lastTransLt: 1,
      lastTransHash: "pre-hash",
      accountAddress: "account",
      status: "active",
      balanceNanotons: "10",
    },
    post: {
      shardAccountBoc64: "post",
      lastTransLt: 42,
      lastTransHash: "post-hash",
      accountAddress: "account",
      status: "active",
      balanceNanotons: "8",
    },
  },
  inbound: {
    direction: "inbound",
    kind: "internal",
    opcode: "0x00000001",
    messageBoc64: "msg",
    body: {boc64: "body", hash: "body-hash", bits: 32, refs: 0},
  },
  outbound: [],
  compute: {
    skipped: false,
    success: true,
    exitCode: 0,
    vmSteps: 12,
    gasUsed: 3,
    gasFees: 4,
  },
  money: {
    balanceBefore: 10,
    sentTotal: 1,
    totalFees: 1,
    balanceAfter: 8,
  },
  outActions: [],
  vmTrace: {lineCount: 2, text: "vm"},
  executorTrace: {lineCount: 1, text: "executor"},
}

const corpus = {
  schemaVersion: 1,
  network: "mainnet",
  address: "account",
  requestedLimit: 1,
  sourceTxCount: 1,
  retracedCount: 1,
  failureCount: 0,
  opcodeSummary: [{opcode: "0x00000001", count: 1, txHashes: ["tx-hash"]}],
  transactions: [stateFlowTx],
  failures: [],
}

const schema = {
  schemaVersion: 1,
  network: "mainnet",
  address: "account",
  transactionCount: 1,
  stateMachine: {
    edges: [
      {
        fromStatus: "active",
        toStatus: "active",
        opcode: "0x00000001",
        count: 1,
        examples: ["tx-hash"],
      },
    ],
  },
  auditSignals: [
    {
      kind: "unknown-fields",
      severity: "medium",
      description: "Unknown fields remain for opcode 0x00000001.",
      evidence: ["tx-hash"],
    },
  ],
  opcodeCandidates: [
    {
      opcode: "0x00000001",
      count: 1,
      examples: ["tx-hash"],
      inboundBody: {minBits: 32, maxBits: 32, minRefs: 0, maxRefs: 0, bodyHashes: ["body-hash"]},
      replayProbes: [
        {
          fieldName: "query_id",
          bitOffset: 32,
          bits: 64,
          value: "0x0000000000000006",
          mutation: {
            type: "setBodyUint",
            bitOffset: 32,
            bits: 64,
            value: "0x0000000000000006",
          },
          cliArg: "--set-body-uint 32:64:0x0000000000000006",
          confidence: "high",
          evidence: ["tx-hash"],
        },
      ],
      storage: {
        balanceDeltaMin: -2,
        balanceDeltaMax: -2,
        dataHashChangedCount: 1,
        codeHashChangedCount: 0,
        postDataHashes: ["data-hash"],
        postCodeHashes: ["code-hash"],
      },
      stateTransitions: [{fromStatus: "active", toStatus: "active", count: 1}],
      outboundEffects: [],
      outActions: [],
      confidence: "medium",
      unknownFields: ["message body field names require TL-B recovery"],
    },
  ],
}

const replay = {
  schemaVersion: 1,
  sourceQueryHash: "tx-hash",
  mutation: {type: "flipBodyBit", bit: 32},
  ignoreChksig: true,
  baseline: {accepted: true, inbound: stateFlowTx.inbound, outbound: [], outActions: []},
  replay: {accepted: true, inbound: stateFlowTx.inbound, outbound: [], outActions: []},
  diff: {
    replayAccepted: true,
    inputChanged: true,
    stateChanged: true,
    codeHashChanged: false,
    dataHashChanged: false,
    balanceDeltaDiff: 0,
    exitCodeChanged: false,
    outboundCountDelta: 0,
    actionCountDelta: 0,
    c5Changed: false,
  },
}

const runSummary = {
  schemaVersion: 1,
  targetCount: 1,
  passed: true,
  absolutePathCount: 0,
  gateFailures: [],
  targets: [
    {
      id: "target-a",
      network: "mainnet",
      address: "account",
      sourceUrl: undefined,
      collectLimit: 1,
      sourceTxCount: 1,
      retracedCount: 1,
      failureCount: 0,
      opcodeCandidateCount: 1,
      stateEdgeCount: 1,
      auditSignalCount: 1,
      replayCount: 1,
      passed: true,
      gateFailures: [],
      outputDir: "out/target-a",
      corpus: "out/target-a/corpus.json",
      schema: "out/target-a/schema.json",
      transaction: "out/target-a/transaction-0.json",
      replay: "out/target-a/replay.json",
      replays: ["out/target-a/replay.json", "out/target-a/replay-probe-query-id-32-64.json"],
      report: "out/target-a/report.md",
    },
  ],
}

const artifactManifest = {
  schemaVersion: 1,
  kind: "stateFlowArtifactManifest",
  summary: "out/summary.json",
  targetCount: 1,
  absolutePathCount: 0,
  artifacts: [
    {kind: "runSummary", path: "out/summary.json", targetId: undefined},
    {kind: "corpus", path: "out/target-a/corpus.json", targetId: "target-a"},
    {kind: "schema", path: "out/target-a/schema.json", targetId: "target-a"},
    {kind: "transaction", path: "out/target-a/transaction-0.json", targetId: "target-a"},
    {kind: "replay", path: "out/target-a/replay.json", targetId: "target-a"},
    {kind: "replay", path: "out/target-a/replay-probe-query-id-32-64.json", targetId: "target-a"},
    {kind: "report", path: "out/target-a/report.md", targetId: "target-a"},
  ],
}

const artifacts: StateFlowArtifact[] = [
  parseStateFlowArtifact(JSON.stringify(stateFlowTx)),
  parseStateFlowArtifact(JSON.stringify(corpus)),
  parseStateFlowArtifact(JSON.stringify(schema)),
  parseStateFlowArtifact(JSON.stringify(replay)),
  parseStateFlowArtifact(JSON.stringify(runSummary)),
  parseStateFlowArtifact(JSON.stringify(artifactManifest)),
]

const artifactKinds: Array<StateFlowArtifact["kind"]> = [
  "transaction",
  "corpus",
  "schema",
  "replay",
  "runSummary",
  "artifactManifest",
]

for (const [index, kind] of artifactKinds.entries()) {
  assert(artifacts[index]?.kind === kind, `expected artifact ${index} to be ${kind}`)
}

const summary = summarizeStateFlowArtifact(parseStateFlowArtifact(JSON.stringify(corpus)))

assert(summary.title === "State Flow Corpus", "expected corpus summary title")
assert(
  summary.metrics.some(metric => metric.label === "Retraced" && metric.value === "1"),
  "expected retraced metric",
)
assert(summary.sections[0]?.rows[0]?.label === "0x00000001", "expected opcode row label")
assert(summary.sections[0]?.rows[0]?.value === "1 transaction", "expected opcode row value")
const schemaSummary = summarizeStateFlowArtifact(parseStateFlowArtifact(JSON.stringify(schema)))
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("storage balance -2") === true,
  "expected schema summary to include storage evidence",
)
assert(
  schemaSummary.metrics.some(metric => metric.label === "State Edges" && metric.value === "1"),
  "expected schema summary to include state machine edge count",
)
assert(
  schemaSummary.metrics.some(metric => metric.label === "Replay Probes" && metric.value === "1"),
  "expected schema summary to include replay probe count",
)
assert(
  schemaSummary.sections.some(section => section.title === "Replay Probes"),
  "expected schema summary to include replay probes",
)
assert(
  schemaSummary.sections.some(section => section.title === "Risk Points"),
  "expected schema summary to include risk points",
)
const runSummaryView = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(runSummary)),
)
assert(runSummaryView.title === "State Flow Run Summary", "expected run summary title")
assert(
  runSummaryView.sections[0]?.rows[0]?.detail?.includes("opcodes 1") === true,
  "expected run summary target detail",
)
const manifestView = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(artifactManifest)),
)
assert(manifestView.title === "State Flow Artifact Manifest", "expected artifact manifest title")
assert(
  manifestView.metrics.some(metric => metric.label === "Absolute Paths" && metric.value === "0"),
  "expected artifact manifest absolute path count",
)
assert(
  manifestView.sections
    .find(section => section.title === "Targets")
    ?.rows[0]?.detail?.includes("replay x2") === true,
  "expected artifact manifest target coverage",
)
assert(typeof StateFlowArtifactView === "function", "expected artifact view component export")
assert(typeof StateFlowArtifactWorkbench === "function", "expected workbench component export")

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(message)
  }
}
