import {
  parseStateFlowArtifact,
  parseStateFlowArtifactBundleFromSources,
  parseStateFlowArtifactFromSource,
  summarizeStateFlowArtifact,
  type StateFlowArtifact,
} from "./stateFlowArtifacts.ts"

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
      codeHash: "code-a",
      dataHash: "data-a",
    },
    post: {
      shardAccountBoc64: "post",
      lastTransLt: 42,
      lastTransHash: "post-hash",
      accountAddress: "account",
      status: "frozen",
      balanceNanotons: "8",
      codeHash: "code-a",
      dataHash: "data-b",
    },
  },
  inbound: {
    direction: "inbound",
    kind: "internal",
    src: "sender",
    dst: "account",
    valueNanotons: "2",
    opcode: "0x00000001",
    messageBoc64: "msg",
    body: {boc64: "body", hash: "body-hash", bits: 32, refs: 0},
  },
  outbound: [
    {
      direction: "outbound",
      index: 0,
      kind: "internal",
      src: "account",
      dst: "receiver",
      valueNanotons: "1",
      opcode: "0x00000002",
      messageBoc64: "out-msg",
      body: {boc64: "out-body", hash: "out-body", bits: 16, refs: 0},
    },
  ],
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
  c5: {boc64: "c5", hash: "c5-hash", bits: 8, refs: 1},
  outActions: [
    {
      index: 0,
      kind: "send_msg",
      mode: "64",
      valueNanotons: "1",
      destination: "receiver",
      body: {boc64: "action-body", hash: "action-body", bits: 16, refs: 0},
    },
  ],
  vmTrace: {lineCount: 2, text: "vm step 1"},
  executorTrace: {lineCount: 1, text: "executor accepted"},
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
    nodes: [
      {
        status: "active",
        transactionCount: 1,
        preCount: 1,
        postCount: 1,
        confidence: "low",
        examples: ["tx-hash"],
      },
    ],
    edges: [
      {
        fromStatus: "active",
        toStatus: "active",
        opcode: "0x00000001",
        count: 1,
        confidence: "low",
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
      evidence: [
        {
          txHash: "tx-hash",
          inboundBodyHash: "body-hash",
          inboundBodyBits: 32,
          inboundBodyRefs: 0,
          fromStatus: "active",
          toStatus: "frozen",
          preDataHash: "data-a",
          postDataHash: "data-b",
          preCodeHash: "code-a",
          postCodeHash: "code-a",
          outboundKinds: ["internal"],
          outActionKinds: ["send_msg"],
        },
      ],
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
      unknownFieldEvidence: [
        {
          marker: "message body field names require TL-B recovery",
          confidence: "high",
          evidence: ["tx-hash"],
        },
      ],
    },
  ],
}

const replay = {
  schemaVersion: 1,
  sourceQueryHash: "tx-hash",
  mutation: {type: "flipBodyBit", bit: 32},
  ignoreChksig: true,
  baseline: {
    accepted: true,
    inbound: stateFlowTx.inbound,
    outbound: [],
    compute: stateFlowTx.compute,
    outActions: [],
  },
  replay: {
    accepted: true,
    inbound: {
      ...stateFlowTx.inbound,
      body: {boc64: "body-b", hash: "body-b", bits: 32, refs: 0},
    },
    outbound: [],
    compute: {...stateFlowTx.compute, exitCode: 1},
    outActions: [],
  },
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
  artifactManifest: "out/artifacts.json",
  validation: "out/validation.json",
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
      unknownFieldCount: 1,
      replayRiskSignalCount: 2,
      replayCount: 1,
      passed: true,
      gateFailures: [],
      outputDir: "out/target-a",
      corpus: "out/target-a/corpus.json",
      schema: "out/target-a/schema.json",
      transaction: "out/target-a/transaction-0.json",
      retrace: "out/target-a/retrace.json",
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
  targets: [
    {
      id: "target-a",
      network: "mainnet",
      address: "account",
      protocol: "sample-protocol",
      category: "sample-category",
      contractType: "sample contract",
    },
  ],
  absolutePathCount: 0,
  artifacts: [
    {kind: "runSummary", path: "out/summary.json", targetId: undefined},
    {kind: "corpus", path: "out/target-a/corpus.json", targetId: "target-a"},
    {kind: "schema", path: "out/target-a/schema.json", targetId: "target-a"},
    {kind: "transaction", path: "out/target-a/transaction-0.json", targetId: "target-a"},
    {kind: "retrace", path: "out/target-a/retrace.json", targetId: "target-a"},
    {kind: "replay", path: "out/target-a/replay.json", targetId: "target-a"},
    {kind: "replay", path: "out/target-a/replay-probe-query-id-32-64.json", targetId: "target-a"},
    {kind: "report", path: "out/target-a/report.md", targetId: "target-a"},
  ],
}

const artifactValidation = {
  schemaVersion: 1,
  kind: "stateFlowArtifactManifestValidation",
  manifest: "out/artifacts.json",
  targetCount: 2,
  absolutePathCount: 0,
  expectedAbsolutePathCount: 0,
  passed: false,
  gateFailures: [],
  capabilityCount: 2,
  capabilityPassedCount: 2,
  capabilityFailedCount: 0,
  targets: [
    {
      id: "target-a",
      artifactCount: 6,
      replayCount: 2,
      passed: true,
      gateFailures: [],
      capabilityCount: 2,
      capabilityPassedCount: 2,
      capabilityFailedCount: 0,
      capabilityChecks: [
        {
          id: "stateFlowTx",
          label: "StateFlowTx evidence JSON",
          passed: true,
          evidence: [
            "transaction:out/target-a/transaction-0.json",
            "pre/post state, inbound body/op, VM trace, executor logs, c5/actions validated",
          ],
        },
        {
          id: "replayDiff",
          label: "Replay diff",
          passed: true,
          evidence: [
            "replay:out/target-a/replay.json",
            "replay:out/target-a/replay-probe-query-id-32-64.json",
            "mutations, replay observations, and observable diffs validated",
          ],
        },
      ],
    },
    {
      id: "target-b",
      artifactCount: 3,
      replayCount: 0,
      passed: false,
      gateFailures: ["missing replay artifact"],
      capabilityCount: 0,
      capabilityPassedCount: 0,
      capabilityFailedCount: 0,
      capabilityChecks: [],
    },
  ],
}

const reportMarkdown = `# TON State Flow Reverse Report

## Target
- Network: \`mainnet\`
- Address: \`account\`
- Source transactions: 1
- Retraced transactions: 1
- Replay failures while collecting: 0

## Opcode Candidates
| Opcode | Count | Confidence |
| --- | ---: | --- |
| \`0x00000001\` | 1 | medium |

## Schema Evidence
| Opcode | Tx | Body hash | Body bits/refs | State |
| --- | --- | --- | ---: | --- |
| \`0x00000001\` | \`tx-hash\` | \`body-hash\` | 32/0 | active -> frozen |

## Message Body Fields
| Opcode | Field | Offset | Bits | Refs | Kind | Samples | Confidence |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`query_id\` | 32 | 64..64 | 0..0 | uint64 | \`0x7\` | high |

## Replay Probes
| Opcode | Field | CLI mutation | Confidence | Evidence |
| --- | --- | --- | --- | --- |
| \`0x00000001\` | \`query_id\` | \`--set-body-uint 32:64:0x6\` | high | \`tx-hash\` |

## Storage Fields
| Opcode | Field | Cell | Offset | Bits | Refs | Kind | Samples | Confidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`data_word_0\` | data | 0 | 32..32 | 0..0 | uint32 | \`0xdeadbeef\` | medium |

## Outbound Effects
| Opcode | Source | Kind | Count | Value | Modes | Destinations | Body | Code | Libraries | Evidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- | --- |
| \`0x00000001\` | outbound | internal | 1 | 11 | none | \`dst\` | 40/1 | n/a | none | \`tx-hash\` |

## State Machine
\`\`\`mermaid
stateDiagram-v2
    active --> frozen: 0x00000001 (1)
\`\`\`

## Replay Diffs
| Source tx | Mutation | Accepted |
| --- | --- | --- |
| \`tx-hash\` | flip body bit 32 | true |

## Unknown Fields
- \`0x00000001\`:
  - message body field names require TL-B recovery

## Risk Points
- Unknown fields remain for opcode 0x00000001.
`

const artifacts: StateFlowArtifact[] = [
  parseStateFlowArtifact(JSON.stringify(stateFlowTx)),
  parseStateFlowArtifact(JSON.stringify(stateFlowTx), {artifactKind: "retrace"}),
  parseStateFlowArtifact(JSON.stringify(corpus)),
  parseStateFlowArtifact(JSON.stringify(schema)),
  parseStateFlowArtifact(JSON.stringify(replay)),
  parseStateFlowArtifact(JSON.stringify(runSummary)),
  parseStateFlowArtifact(JSON.stringify(artifactManifest)),
  parseStateFlowArtifact(JSON.stringify(artifactValidation)),
  parseStateFlowArtifact(reportMarkdown),
]

const artifactKinds: Array<StateFlowArtifact["kind"]> = [
  "transaction",
  "retrace",
  "corpus",
  "schema",
  "replay",
  "runSummary",
  "artifactManifest",
  "artifactValidation",
  "report",
]

for (const [index, kind] of artifactKinds.entries()) {
  assert(artifacts[index]?.kind === kind, `expected artifact ${index} to be ${kind}`)
}

const transactionSummary = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(stateFlowTx)),
)
assert(
  transactionSummary.sections
    .find(section => section.title === "State")
    ?.rows[1]?.detail?.includes("data data-b") === true,
  "expected transaction summary to include post-state data hash",
)
assert(
  transactionSummary.sections
    .find(section => section.title === "Inbound Message")
    ?.rows[0]?.detail?.includes("body body-hash 32/0") === true,
  "expected transaction summary to include inbound body shape",
)
assert(
  transactionSummary.sections
    .find(section => section.title === "Actions")
    ?.rows.some(row => row.label === "c5" && row.value === "c5-hash") === true,
  "expected transaction summary to include c5 action evidence",
)
const retraceSummary = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(stateFlowTx), {artifactKind: "retrace"}),
)
assert(retraceSummary.title === "State Flow Retrace", "expected retrace summary title")
assert(
  retraceSummary.sections
    .find(section => section.title === "Traces")
    ?.rows[0]?.detail?.includes("vm step") === true,
  "expected retrace summary to include VM trace evidence",
)
const retraceFileSummary = summarizeStateFlowArtifact(
  parseStateFlowArtifactFromSource(JSON.stringify(stateFlowTx), "retrace.json"),
)
assert(retraceFileSummary.title === "State Flow Retrace", "expected retrace filename hint")

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
  schemaSummary.sections
    .find(section => section.title === "Schema Evidence")
    ?.rows[0]?.detail?.includes("body body-hash 32/0") === true,
  "expected schema summary to include raw evidence body hash and shape",
)
assert(
  schemaSummary.metrics.some(metric => metric.label === "State Edges" && metric.value === "1"),
  "expected schema summary to include state machine edge count",
)
assert(
  schemaSummary.metrics.some(metric => metric.label === "State Nodes" && metric.value === "1"),
  "expected schema summary to include state machine node count",
)
assert(
  schemaSummary.sections
    .find(section => section.title === "State Machine Nodes")
    ?.rows[0]?.detail?.includes("pre 1 · post 1 · confidence low · examples tx-hash") === true,
  "expected schema state machine node row to include counts, confidence, and examples",
)
assert(
  schemaSummary.sections
    .find(section => section.title === "State Machine")
    ?.rows[0]?.detail?.includes("confidence low") === true,
  "expected schema state machine row to include persisted confidence",
)
assert(
  schemaSummary.sections
    .find(section => section.title === "State Machine")
    ?.rows[0]?.detail?.includes("examples tx-hash") === true,
  "expected schema state machine row to include evidence examples",
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
const replaySummary = summarizeStateFlowArtifact(parseStateFlowArtifact(JSON.stringify(replay)))
assert(
  replaySummary.sections
    .find(section => section.title === "Replay Observations")
    ?.rows[1]?.detail?.includes("body body-b") === true,
  "expected replay summary to include replay observation body hash",
)
assert(
  replaySummary.sections
    .find(section => section.title === "Replay Diff")
    ?.rows.some(row => row.label === "Input" && row.value === "changed") === true,
  "expected replay summary to include replay diff rows",
)
const runSummaryView = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(runSummary)),
)
assert(runSummaryView.title === "State Flow Run Summary", "expected run summary title")
assert(
  runSummaryView.sections[0]?.rows[0]?.detail?.includes("opcodes 1") === true,
  "expected run summary target detail",
)
assert(
  runSummaryView.sections
    .find(section => section.title === "Bundle Artifacts")
    ?.rows.some(row => row.label === "Validation" && row.value === "out/validation.json") === true,
  "expected run summary to include bundle artifact entrypoints",
)
assert(
  runSummaryView.sections
    .find(section => section.title === "Target Artifacts")
    ?.rows.some(
      row =>
        row.label === "target-a replay 1" &&
        row.value === "out/target-a/replay-probe-query-id-32-64.json",
    ) === true,
  "expected run summary to include replay artifact paths",
)
assert(
  runSummaryView.sections
    .find(section => section.title === "Target Artifacts")
    ?.rows.some(
      row => row.label === "target-a retrace" && row.value === "out/target-a/retrace.json",
    ) === true,
  "expected run summary to include retrace artifact path",
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
assert(
  manifestView.sections
    .find(section => section.title === "Targets")
    ?.rows[0]?.detail?.includes("retrace x1") === true,
  "expected artifact manifest retrace coverage",
)
const validationView = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(artifactValidation)),
)
assert(
  validationView.title === "State Flow Artifact Validation",
  "expected artifact validation title",
)
assert(
  validationView.metrics.some(metric => metric.label === "Passed" && metric.value === "no"),
  "expected artifact validation pass metric",
)
assert(
  validationView.metrics.some(
    metric => metric.label === "Capability Checks" && metric.value === "2",
  ),
  "expected artifact validation capability check metric",
)
assert(
  validationView.metrics.some(
    metric => metric.label === "Capability Failures" && metric.value === "0",
  ),
  "expected artifact validation capability failure metric",
)
assert(
  validationView.sections
    .find(section => section.title === "Capability Checks")
    ?.rows.some(
      row =>
        row.label === "target-a StateFlowTx evidence JSON" &&
        row.value === "passed" &&
        row.detail?.includes("transaction:out/target-a/transaction-0.json") === true,
    ) === true,
  "expected artifact validation capability check rows",
)
assert(
  validationView.sections
    .find(section => section.title === "Target Gate Failures")
    ?.rows.some(row => row.label === "target-b" && row.value === "missing replay artifact") ===
    true,
  "expected artifact validation target failure rows",
)
const bundleView = summarizeStateFlowArtifact(
  parseStateFlowArtifactBundleFromSources([
    {name: "out/artifacts.json", raw: JSON.stringify(artifactManifest)},
    {name: "out/summary.json", raw: JSON.stringify(runSummary)},
    {name: "out/validation.json", raw: JSON.stringify(artifactValidation)},
    {name: "out/target-a/corpus.json", raw: JSON.stringify(corpus)},
    {name: "out/target-a/schema.json", raw: JSON.stringify(schema)},
    {name: "out/target-a/transaction-0.json", raw: JSON.stringify(stateFlowTx)},
    {name: "out/target-a/retrace.json", raw: JSON.stringify(stateFlowTx)},
    {name: "out/target-a/replay.json", raw: JSON.stringify(replay)},
    {name: "out/target-a/replay-probe-query-id-32-64.json", raw: JSON.stringify(replay)},
    {name: "out/target-a/report.md", raw: reportMarkdown},
  ]),
)
assert(bundleView.title === "State Flow Artifact Bundle", "expected artifact bundle title")
assert(
  bundleView.metrics.some(metric => metric.label === "Targets" && metric.value === "2"),
  "expected artifact bundle target count",
)
assert(
  bundleView.metrics.some(metric => metric.label === "Missing" && metric.value === "0"),
  "expected artifact bundle to resolve all manifest entries",
)
assert(
  bundleView.sections
    .find(section => section.title === "Targets")
    ?.rows.some(
      row =>
        row.label === "target-a" &&
        row.value === "passed" &&
        row.detail?.includes("sample-protocol sample-category sample contract") === true &&
        row.detail?.includes("loaded 7/7") === true &&
        row.detail?.includes("unknown fields 1") === true &&
        row.detail?.includes("replay risks 2") === true &&
        row.detail?.includes("capabilities 2/2") === true,
    ) === true,
  "expected artifact bundle target rows to merge manifest, summary, and validation evidence",
)
const summaryOnlyBundleView = summarizeStateFlowArtifact(
  parseStateFlowArtifactBundleFromSources([
    {
      name: "out/artifacts.json",
      raw: JSON.stringify({
        ...artifactManifest,
        artifacts: [{kind: "runSummary", path: "out/summary.json", targetId: undefined}],
      }),
    },
    {name: "out/summary.json", raw: JSON.stringify(runSummary)},
  ]),
)
assert(
  summaryOnlyBundleView.sections
    .find(section => section.title === "Risk Matrix")
    ?.rows.some(
      row =>
        row.label === "sample-protocol / sample-category" &&
        row.detail?.includes("audit signals 1") === true &&
        row.detail?.includes("unknown fields 1") === true &&
        row.detail?.includes("replay risks 2") === true,
    ) === true,
  "expected artifact bundle risk matrix to use run summary unknown-field counts when schema artifacts are not loaded",
)
const legacySummary = {
  ...runSummary,
  targets: runSummary.targets.map(
    ({
      unknownFieldCount: _unknownFieldCount,
      replayRiskSignalCount: _replayRiskSignalCount,
      ...target
    }) => target,
  ),
}
const legacySummaryWithSchemaBundleView = summarizeStateFlowArtifact(
  parseStateFlowArtifactBundleFromSources([
    {name: "out/artifacts.json", raw: JSON.stringify(artifactManifest)},
    {name: "out/summary.json", raw: JSON.stringify(legacySummary)},
    {name: "out/validation.json", raw: JSON.stringify(artifactValidation)},
    {name: "out/target-a/corpus.json", raw: JSON.stringify(corpus)},
    {name: "out/target-a/schema.json", raw: JSON.stringify(schema)},
    {name: "out/target-a/transaction-0.json", raw: JSON.stringify(stateFlowTx)},
    {name: "out/target-a/retrace.json", raw: JSON.stringify(stateFlowTx)},
    {name: "out/target-a/replay.json", raw: JSON.stringify(replay)},
    {name: "out/target-a/replay-probe-query-id-32-64.json", raw: JSON.stringify(replay)},
    {name: "out/target-a/report.md", raw: reportMarkdown},
  ]),
)
assert(
  legacySummaryWithSchemaBundleView.sections
    .find(section => section.title === "Targets")
    ?.rows.some(
      row =>
        row.label === "target-a" &&
        row.detail?.includes("unknown fields 1") === true &&
        row.detail?.includes("replay risks 2") === true,
    ) === true,
  "expected artifact bundle target rows to compute risk counts from loaded artifacts when run summary lacks the fields",
)
assert(
  bundleView.sections
    .find(section => section.title === "Coverage Matrix")
    ?.rows.some(
      row =>
        row.label === "sample-protocol / sample-category" &&
        row.value === "1 target" &&
        row.detail?.includes("sample contract") === true &&
        row.detail?.includes("7 artifacts") === true &&
        row.detail?.includes("1 replay") === true &&
        row.detail?.includes("capabilities 2/2") === true,
    ) === true,
  "expected artifact bundle coverage matrix rows to group target evidence by protocol and category",
)
assert(
  bundleView.sections
    .find(section => section.title === "Risk Matrix")
    ?.rows.some(
      row =>
        row.label === "sample-protocol / sample-category" &&
        row.value === "1 target" &&
        row.detail?.includes("sample contract") === true &&
        row.detail?.includes("audit signals 1") === true &&
        row.detail?.includes("unknown fields 1") === true &&
        row.detail?.includes("replay risks 2") === true,
    ) === true,
  "expected artifact bundle risk matrix rows to group schema risk markers by protocol and category",
)
assert(
  bundleView.sections
    .find(section => section.title === "Loaded Artifacts")
    ?.rows.some(
      row => row.label === "target-a replay" && row.value === "out/target-a/replay.json",
    ) === true,
  "expected artifact bundle loaded rows to include target replay artifacts",
)
const schemaForTargetB = {
  ...schema,
  address: "account-b",
}
const basenameBundle = parseStateFlowArtifactBundleFromSources([
  {
    name: "artifacts.json",
    raw: JSON.stringify({
      ...artifactManifest,
      targets: [
        {id: "target-a", network: "mainnet", address: "account"},
        {id: "target-b", network: "mainnet", address: "account-b"},
      ],
      artifacts: [
        {kind: "schema", path: "target-a/schema.json", targetId: "target-a"},
        {kind: "schema", path: "target-b/schema.json", targetId: "target-b"},
      ],
    }),
  },
  {name: "schema.json", raw: JSON.stringify(schema)},
  {name: "schema.json", raw: JSON.stringify(schemaForTargetB)},
])
assert(
  basenameBundle.kind === "artifactBundle" &&
    basenameBundle.data.missingArtifacts.length === 0 &&
    basenameBundle.data.targets.every(target => target.loadedArtifacts.length === 1),
  "expected basename-only bundle sources to resolve duplicate target artifacts by target identity",
)
const reportView = summarizeStateFlowArtifact(parseStateFlowArtifact(reportMarkdown))
assert(reportView.title === "TON State Flow Reverse Report", "expected report summary title")
assert(
  reportView.metrics.some(metric => metric.label === "Sections" && metric.value === "11"),
  "expected report summary section count",
)
assert(
  reportView.sections
    .find(section => section.title === "Target")
    ?.rows.some(row => row.label === "Address" && row.value === "account") === true,
  "expected report target rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Opcode Candidates")
    ?.rows.some(row => row.label === "0x00000001" && row.value === "medium confidence") === true,
  "expected report opcode candidate table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Schema Evidence")
    ?.rows.some(row => row.label === "0x00000001 tx-hash" && row.value === "active -> frozen") ===
    true,
  "expected report schema evidence table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Message Body Fields")
    ?.rows.some(row => row.label === "0x00000001 query_id" && row.value === "uint64") === true,
  "expected report message body field table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Replay Probes")
    ?.rows.some(
      row => row.label === "0x00000001 query_id" && row.value === "--set-body-uint 32:64:0x6",
    ) === true,
  "expected report replay probe table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Storage Fields")
    ?.rows.some(row => row.label === "0x00000001 data_word_0" && row.value === "data @ 0") === true,
  "expected report storage field table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Outbound Effects")
    ?.rows.some(row => row.label === "0x00000001 outbound internal" && row.value === "1 effect") ===
    true,
  "expected report outbound effect table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "State Machine")
    ?.rows.some(row => row.label === "active -> frozen" && row.value === "0x00000001 (1)") === true,
  "expected report state machine rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Replay Diffs")
    ?.rows.some(row => row.label === "tx-hash" && row.value === "flip body bit 32") === true,
  "expected report replay diff table rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Unknown Fields")
    ?.rows.some(
      row =>
        row.label === "0x00000001" &&
        row.value === "message body field names require TL-B recovery",
    ) === true,
  "expected report unknown field rows",
)
assert(
  reportView.sections
    .find(section => section.title === "Risk Points")
    ?.rows.some(row => row.value === "Unknown fields remain for opcode 0x00000001.") === true,
  "expected report risk point rows",
)
function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(message)
  }
}
