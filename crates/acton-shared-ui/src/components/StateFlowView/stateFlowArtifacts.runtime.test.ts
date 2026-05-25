import {parseStateFlowArtifact, summarizeStateFlowArtifact} from "./stateFlowArtifacts.ts"

const stateFlowTx = {
  schemaVersion: 1,
  network: "mainnet",
  queryHash: "tx-a",
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
    replayedPrevTxCount: 1,
    blockConfigBoc64: "config",
    libsBoc64: "libs",
  },
  state: {
    pre: {
      shardAccountBoc64: "pre",
      lastTransLt: 1,
      lastTransHash: "pre-hash",
      accountAddress: "account",
      status: "active",
      balanceNanotons: "100",
      codeHash: "code-a",
      dataHash: "data-a",
    },
    post: {
      shardAccountBoc64: "post",
      lastTransLt: 42,
      lastTransHash: "post-hash",
      accountAddress: "account",
      status: "frozen",
      balanceNanotons: "75",
      codeHash: "code-a",
      dataHash: "data-b",
    },
  },
  inbound: {
    direction: "inbound",
    kind: "internal",
    src: "sender",
    dst: "account",
    valueNanotons: "25",
    opcode: "0x00000001",
    messageBoc64: "msg-a",
    body: {boc64: "body-a", hash: "body-a", bits: 96, refs: 1},
  },
  outbound: [
    {
      direction: "outbound",
      index: 0,
      kind: "internal",
      src: "account",
      dst: "receiver",
      valueNanotons: "7",
      opcode: "0x00000002",
      messageBoc64: "out-msg",
      body: {boc64: "out-body", hash: "out-body", bits: 32, refs: 0},
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
    balanceBefore: 100,
    sentTotal: 7,
    totalFees: 1,
    balanceAfter: 75,
  },
  c5: {boc64: "c5", hash: "c5-hash", bits: 24, refs: 1},
  outActions: [
    {
      index: 0,
      kind: "send_msg",
      mode: "64",
      valueNanotons: "7",
      destination: "receiver",
      body: {boc64: "action-body", hash: "action-body", bits: 32, refs: 0},
    },
  ],
  vmTrace: {lineCount: 2, text: "vm step 1\nvm step 2"},
  executorTrace: {lineCount: 1, text: "executor accepted"},
}

const schema = {
  schemaVersion: 1,
  network: "mainnet",
  address: "account",
  transactionCount: 2,
  opcodeCandidates: [
    {
      opcode: "0x00000001",
      count: 2,
      examples: ["tx-a", "tx-b"],
      evidence: [
        {
          txHash: "tx-a",
          inboundBodyHash: "body-a",
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
      inboundBody: {
        minBits: 32,
        maxBits: 96,
        minRefs: 0,
        maxRefs: 1,
        bodyHashes: ["body-a", "body-b"],
        fieldCandidates: [
          {
            name: "opcode",
            bitOffset: 0,
            minBits: 32,
            maxBits: 32,
            minRefs: 0,
            maxRefs: 0,
            kind: "uint32",
            presentCount: 2,
            valueSamples: ["0x00000001"],
            confidence: "high",
          },
          {
            name: "query_id",
            bitOffset: 32,
            minBits: 64,
            maxBits: 64,
            minRefs: 0,
            maxRefs: 0,
            kind: "uint64",
            presentCount: 2,
            valueSamples: ["0x0000000000000007", "0x0000000000000008"],
            confidence: "high",
          },
        ],
      },
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
          evidence: ["tx-a"],
        },
      ],
      storage: {
        balanceDeltaMin: -2,
        balanceDeltaMax: 4,
        dataHashChangedCount: 1,
        codeHashChangedCount: 0,
        postDataShape: {minBits: 16, maxBits: 16, minRefs: 1, maxRefs: 1},
        postCodeShape: {minBits: 8, maxBits: 8, minRefs: 0, maxRefs: 0},
        fields: [
          {
            name: "data_word_0",
            cellPath: "data",
            bitOffset: 0,
            minBits: 32,
            maxBits: 32,
            minRefs: 0,
            maxRefs: 0,
            kind: "uint32",
            presentCount: 2,
            valueSamples: ["0xcafebabe", "0xdeadbeef"],
            confidence: "high",
          },
        ],
        postDataHashes: ["data-a", "data-b"],
        postCodeHashes: ["code-a"],
      },
      stateTransitions: [{fromStatus: "active", toStatus: "frozen", count: 2}],
      outboundEffects: [
        {
          kind: "internal",
          count: 1,
          txHashes: ["tx-a"],
          modes: [],
          destinations: ["out-dst"],
          valueNanotonsMin: "11",
          valueNanotonsMax: "11",
          bodyShape: {minBits: 40, maxBits: 40, minRefs: 1, maxRefs: 1},
          codeShape: undefined,
          libraryHashes: [],
        },
      ],
      outActions: [
        {
          kind: "send-message",
          count: 1,
          txHashes: ["tx-a"],
          modes: ["64"],
          destinations: ["action-dst"],
          valueNanotonsMin: "7",
          valueNanotonsMax: "7",
          bodyShape: {minBits: 32, maxBits: 32, minRefs: 0, maxRefs: 0},
          codeShape: undefined,
          libraryHashes: [],
        },
      ],
      confidence: "low",
      unknownFields: ["message body field names require TL-B recovery"],
    },
  ],
}

const replay = {
  schemaVersion: 1,
  sourceQueryHash: "tx-a",
  mutation: {type: "flipBodyBit", bit: 32},
  ignoreChksig: true,
  baseline: {
    accepted: true,
    inbound: {
      direction: "inbound",
      kind: "internal",
      opcode: "0x00000001",
      messageBoc64: "msg-a",
      body: {boc64: "body-a", hash: "body-a", bits: 32, refs: 0},
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
    outActions: [],
  },
  replay: {
    accepted: true,
    inbound: {
      direction: "inbound",
      kind: "internal",
      opcode: "0x00000001",
      messageBoc64: "msg-b",
      body: {boc64: "body-b", hash: "body-b", bits: 32, refs: 0},
    },
    outbound: [],
    compute: {
      skipped: false,
      success: false,
      exitCode: 7,
      vmSteps: 13,
      gasUsed: 5,
      gasFees: 6,
    },
    outActions: [],
  },
  diff: {
    replayAccepted: true,
    inputChanged: true,
    stateChanged: true,
    codeHashChanged: false,
    dataHashChanged: true,
    balanceDeltaDiff: 4,
    exitCodeChanged: false,
    outboundCountDelta: 1,
    actionCountDelta: 0,
    c5Changed: false,
  },
}

const setBodyUintReplay = {
  ...replay,
  mutation: {type: "setBodyUint", bitOffset: 32, bits: 64, value: "42"},
}

const runSummary = {
  schemaVersion: 1,
  targetCount: 2,
  passed: false,
  absolutePathCount: 0,
  gateFailures: ["target-b: replays 0"],
  targets: [
    {
      id: "target-a",
      network: "mainnet",
      address: "addr-a",
      sourceUrl: undefined,
      collectLimit: 2,
      sourceTxCount: 2,
      retracedCount: 2,
      failureCount: 0,
      opcodeCandidateCount: 1,
      stateEdgeCount: 1,
      auditSignalCount: 3,
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
    {
      id: "target-b",
      network: "mainnet",
      address: "addr-b",
      sourceUrl: "https://tonviewer.com/addr-b",
      collectLimit: 2,
      sourceTxCount: 2,
      retracedCount: 2,
      failureCount: 0,
      opcodeCandidateCount: 1,
      stateEdgeCount: 1,
      auditSignalCount: 2,
      replayCount: 0,
      passed: false,
      gateFailures: ["replays 0"],
      outputDir: "out/target-b",
      corpus: "out/target-b/corpus.json",
      schema: "out/target-b/schema.json",
      transaction: undefined,
      replay: undefined,
      replays: [],
      report: "out/target-b/report.md",
    },
  ],
}

const artifactManifest = {
  schemaVersion: 1,
  kind: "stateFlowArtifactManifest",
  summary: "out/summary.json",
  targetCount: 2,
  absolutePathCount: 0,
  artifacts: [
    {kind: "runSummary", path: "out/summary.json", targetId: undefined},
    {kind: "corpus", path: "out/target-a/corpus.json", targetId: "target-a"},
    {kind: "schema", path: "out/target-a/schema.json", targetId: "target-a"},
    {kind: "transaction", path: "out/target-a/transaction-0.json", targetId: "target-a"},
    {kind: "replay", path: "out/target-a/replay.json", targetId: "target-a"},
    {kind: "replay", path: "out/target-a/replay-probe-query-id-32-64.json", targetId: "target-a"},
    {kind: "report", path: "out/target-a/report.md", targetId: "target-a"},
    {kind: "corpus", path: "out/target-b/corpus.json", targetId: "target-b"},
    {kind: "schema", path: "out/target-b/schema.json", targetId: "target-b"},
    {kind: "report", path: "out/target-b/report.md", targetId: "target-b"},
  ],
}

const artifactValidation = {
  schemaVersion: 1,
  kind: "stateFlowArtifactManifestValidation",
  manifest: "out/artifacts.json",
  targetCount: 2,
  absolutePathCount: 0,
  expectedAbsolutePathCount: 0,
  passed: true,
  gateFailures: [],
  targets: [
    {
      id: "target-a",
      artifactCount: 6,
      replayCount: 2,
      passed: true,
      gateFailures: [],
    },
    {
      id: "target-b",
      artifactCount: 3,
      replayCount: 0,
      passed: false,
      gateFailures: ["missing replay artifact"],
    },
  ],
}

const reportMarkdown = `# TON State Flow Reverse Report

## Target
- Network: \`mainnet\`
- Address: \`addr-a\`
- Source transactions: 2
- Retraced transactions: 2
- Replay failures while collecting: 0

## Opcode Candidates
| Opcode | Count | Confidence |
| --- | ---: | --- |
| \`0x00000001\` | 2 | low |

## Schema Evidence
| Opcode | Tx | Body hash | Body bits/refs | State |
| --- | --- | --- | ---: | --- |
| \`0x00000001\` | \`tx-a\` | \`body-a\` | 32/0 | active -> frozen |

## Message Body Fields
| Opcode | Field | Offset | Bits | Refs | Kind | Samples | Confidence |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`query_id\` | 32 | 64..64 | 0..0 | uint64 | \`0x7\` | high |

## Replay Probes
| Opcode | Field | CLI mutation | Confidence | Evidence |
| --- | --- | --- | --- | --- |
| \`0x00000001\` | \`query_id\` | \`--set-body-uint 32:64:0x6\` | high | \`tx-a\` |

## Storage Fields
| Opcode | Field | Cell | Offset | Bits | Refs | Kind | Samples | Confidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`data_word_0\` | data | 0 | 32..32 | 0..0 | uint32 | \`0xdeadbeef\` | medium |

## Outbound Effects
| Opcode | Source | Kind | Count | Value | Modes | Destinations | Body | Code | Libraries | Evidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- | --- |
| \`0x00000001\` | outbound | internal | 1 | 11 | none | \`dst\` | 40/1 | n/a | none | \`tx-a\` |

## State Machine
\`\`\`mermaid
stateDiagram-v2
    active --> frozen: 0x00000001 (2)
\`\`\`

## Replay Diffs
| Source tx | Mutation | Accepted |
| --- | --- | --- |
| \`tx-a\` | flip body bit 32 | true |

## Unknown Fields
- \`0x00000001\`:
  - message body field names require TL-B recovery

## Risk Points
- Unknown fields remain for opcode 0x00000001.
`

const transactionSummary = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(stateFlowTx)),
)
const transactionStateRows = sectionRows(transactionSummary, "State")
assert(transactionStateRows[0]?.label === "pre", "expected pre-state row")
assert(transactionStateRows[0]?.value === "active", "expected pre-state status")
assert(
  transactionStateRows[0]?.detail?.includes("balance 100") === true,
  "expected pre-state balance",
)
assert(
  transactionStateRows[1]?.detail?.includes("data data-b") === true,
  "expected post-state data hash",
)
const inboundRows = sectionRows(transactionSummary, "Inbound Message")
assert(inboundRows[0]?.label === "internal", "expected inbound kind")
assert(inboundRows[0]?.value === "0x00000001", "expected inbound opcode")
assert(inboundRows[0]?.detail?.includes("body body-a 96/1") === true, "expected inbound body shape")
const outboundRows = sectionRows(transactionSummary, "Outbound Messages")
assert(outboundRows[0]?.label === "0 internal", "expected outbound index and kind")
assert(outboundRows[0]?.value === "0x00000002", "expected outbound opcode")
assert(outboundRows[0]?.detail?.includes("receiver") === true, "expected outbound destination")
const actionRows = sectionRows(transactionSummary, "Actions")
assert(actionRows[0]?.label === "c5", "expected c5 row")
assert(actionRows[0]?.value === "c5-hash", "expected c5 hash")
assert(actionRows[1]?.label === "0 send_msg", "expected action row")
assert(actionRows[1]?.detail?.includes("mode 64") === true, "expected action mode")
const traceRows = sectionRows(transactionSummary, "Traces")
assert(traceRows[0]?.label === "VM trace", "expected vm trace row")
assert(traceRows[0]?.detail === "vm step 1", "expected vm trace preview")
assert(traceRows[1]?.label === "Executor trace", "expected executor trace row")
assert(traceRows[1]?.detail === "executor accepted", "expected executor trace preview")

const schemaSummary = summarizeStateFlowArtifact(parseStateFlowArtifact(JSON.stringify(schema)))
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("1 evidence row") === true,
  "expected candidate summary to include evidence row count",
)
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("2 body fields") === true,
  "expected candidate summary to include body field count",
)
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("1 storage field") === true,
  "expected candidate summary to include storage field count",
)
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("2 effects") === true,
  "expected candidate summary to include effect count",
)
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("1 replay probe") === true,
  "expected candidate summary to include replay probe count",
)
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("data 16/1") === true,
  "expected candidate summary to include storage data shape",
)
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("code 8/0") === true,
  "expected candidate summary to include storage code shape",
)
const stateMachineRows = sectionRows(schemaSummary, "State Machine")
assert(stateMachineRows[0]?.label === "0x00000001", "expected opcode on state machine row")
assert(stateMachineRows[0]?.value === "active -> frozen", "expected state transition row")
assert(
  stateMachineRows[0]?.detail === "2 observed transitions · examples tx-a, tx-b",
  "expected transition evidence count and examples",
)
const bodyFieldRows = sectionRows(schemaSummary, "Message Body Fields")
assert(bodyFieldRows[1]?.label === "0x00000001 query_id", "expected query_id body field row")
assert(bodyFieldRows[1]?.value === "uint64 @32", "expected query_id field offset")
assert(
  bodyFieldRows[1]?.detail?.includes("0x0000000000000007") === true,
  "expected query_id samples in body field row",
)
const storageFieldRows = sectionRows(schemaSummary, "Storage Fields")
assert(storageFieldRows[0]?.label === "0x00000001 data_word_0", "expected storage field row")
assert(storageFieldRows[0]?.value === "uint32 @data:0", "expected storage field offset")
assert(
  storageFieldRows[0]?.detail?.includes("0xdeadbeef") === true,
  "expected storage field samples in row",
)
const effectRows = sectionRows(schemaSummary, "Outbound Effects")
assert(effectRows[0]?.label === "0x00000001 outbound", "expected outbound effect row")
assert(effectRows[0]?.value === "internal x1", "expected outbound effect kind")
assert(effectRows[0]?.detail?.includes("out-dst") === true, "expected outbound destination")
assert(effectRows[1]?.label === "0x00000001 action", "expected action effect row")
assert(effectRows[1]?.value === "send-message x1", "expected action effect kind")
assert(effectRows[1]?.detail?.includes("64") === true, "expected action mode")
const schemaEvidenceRows = sectionRows(schemaSummary, "Schema Evidence")
assert(schemaEvidenceRows[0]?.label === "0x00000001 tx-a", "expected schema evidence row")
assert(schemaEvidenceRows[0]?.value === "active -> frozen", "expected schema evidence transition")
assert(
  schemaEvidenceRows[0]?.detail?.includes("body body-a 32/0") === true,
  "expected schema evidence body hash and shape",
)
assert(
  schemaEvidenceRows[0]?.detail?.includes("data data-a -> data-b") === true,
  "expected schema evidence data hash transition",
)
assert(
  schemaEvidenceRows[0]?.detail?.includes("out internal") === true,
  "expected schema evidence outbound kind",
)
const replayProbeRows = sectionRows(schemaSummary, "Replay Probes")
assert(replayProbeRows[0]?.label === "0x00000001 query_id", "expected replay probe row")
assert(
  replayProbeRows[0]?.value === "--set-body-uint 32:64:0x0000000000000006",
  "expected replay probe CLI arg",
)
assert(replayProbeRows[0]?.detail?.includes("tx-a") === true, "expected replay probe evidence")

const schemaRiskRows = sectionRows(schemaSummary, "Risk Points")
assert(
  schemaRiskRows.some(row => row.value.includes("Unknown fields remain")),
  "expected schema unknown-field risk point",
)
assert(
  schemaRiskRows.some(row => row.value.includes("Low confidence")),
  "expected schema confidence risk point",
)

const replaySummary = summarizeStateFlowArtifact(parseStateFlowArtifact(JSON.stringify(replay)))
const replayObservationRows = sectionRows(replaySummary, "Replay Observations")
assert(replayObservationRows[0]?.label === "baseline", "expected baseline observation row")
assert(replayObservationRows[0]?.value === "accepted", "expected accepted baseline row")
assert(
  replayObservationRows[0]?.detail?.includes("body body-a") === true,
  "expected baseline body hash in replay observation",
)
assert(
  replayObservationRows[0]?.detail?.includes("exit 0") === true,
  "expected baseline exit code in replay observation",
)
assert(replayObservationRows[1]?.label === "replay", "expected replay observation row")
assert(
  replayObservationRows[1]?.detail?.includes("body body-b") === true,
  "expected replay body hash in replay observation",
)
assert(
  replayObservationRows[1]?.detail?.includes("exit 7") === true,
  "expected replay exit code in replay observation",
)
const replayDiffRows = sectionRows(replaySummary, "Replay Diff")
assert(
  replayDiffRows.some(row => row.label === "Input" && row.value === "changed"),
  "expected replay diff input row",
)
assert(
  replayDiffRows.some(row => row.label === "Data Hash" && row.value === "changed"),
  "expected replay diff data hash row",
)
assert(
  replayDiffRows.some(row => row.label === "Balance Delta" && row.value === "4"),
  "expected replay diff balance delta row",
)
const replayRiskRows = sectionRows(replaySummary, "Risk Points")
assert(
  replayRiskRows.some(row => row.value === "Mutation changed state"),
  "expected replay state-change risk point",
)
assert(
  replayRiskRows.some(row => row.value === "Mutation changed outbound/action counts"),
  "expected replay outbound/action risk point",
)
const setBodyUintReplaySummary = summarizeStateFlowArtifact(
  parseStateFlowArtifact(JSON.stringify(setBodyUintReplay)),
)
assert(
  setBodyUintReplaySummary.metrics.some(
    metric => metric.label === "Mutation" && metric.value === "set body uint 42 at 32:64",
  ),
  "expected setBodyUint replay mutation label",
)

const runSummaryArtifact = parseStateFlowArtifact(JSON.stringify(runSummary))
assert(runSummaryArtifact.kind === "runSummary", "expected run summary artifact kind")
const runSummaryView = summarizeStateFlowArtifact(runSummaryArtifact)
assert(runSummaryView.title === "State Flow Run Summary", "expected run summary title")
assert(
  runSummaryView.metrics.some(metric => metric.label === "Passed" && metric.value === "no"),
  "expected failed run metric",
)
assert(
  runSummaryView.metrics.some(metric => metric.label === "Targets" && metric.value === "2"),
  "expected target count metric",
)
assert(
  runSummaryView.metrics.some(metric => metric.label === "Absolute Paths" && metric.value === "0"),
  "expected run summary absolute path count metric",
)
const targetRows = sectionRows(runSummaryView, "Targets")
assert(targetRows[0]?.label === "target-a", "expected first target row")
assert(targetRows[0]?.value === "passed", "expected passed target value")
assert(targetRows[0]?.detail?.includes("opcodes 1") === true, "expected target artifact detail")
assert(
  targetRows[0]?.detail?.includes("2 replay artifacts") === true,
  "expected target detail to include replay artifact count",
)
assert(targetRows[1]?.value === "failed", "expected failed target value")
const runArtifactRows = sectionRows(runSummaryView, "Target Artifacts")
assert(runArtifactRows[0]?.label === "target-a corpus", "expected target corpus artifact row")
assert(runArtifactRows[0]?.value === "out/target-a/corpus.json", "expected corpus artifact path")
assert(runArtifactRows[1]?.label === "target-a schema", "expected target schema artifact row")
assert(
  runArtifactRows.some(
    row =>
      row.label === "target-a replay 1" &&
      row.value === "out/target-a/replay-probe-query-id-32-64.json",
  ),
  "expected all replay artifact paths",
)
assert(
  runArtifactRows.some(
    row => row.label === "target-b report" && row.value === "out/target-b/report.md",
  ),
  "expected failed target report artifact path",
)
const gateRows = sectionRows(runSummaryView, "Gate Failures")
assert(gateRows[0]?.label === "target-b", "expected target id on gate failure row")
assert(gateRows[0]?.value === "replays 0", "expected gate failure reason")

const manifestArtifact = parseStateFlowArtifact(JSON.stringify(artifactManifest))
assert(manifestArtifact.kind === "artifactManifest", "expected artifact manifest kind")
const manifestSummary = summarizeStateFlowArtifact(manifestArtifact)
assert(manifestSummary.title === "State Flow Artifact Manifest", "expected manifest title")
assert(
  manifestSummary.metrics.some(metric => metric.label === "Artifacts" && metric.value === "10"),
  "expected manifest artifact count metric",
)
assert(
  manifestSummary.metrics.some(metric => metric.label === "Absolute Paths" && metric.value === "0"),
  "expected manifest absolute path count metric",
)
const manifestTargetRows = sectionRows(manifestSummary, "Targets")
assert(manifestTargetRows[0]?.label === "target-a", "expected first manifest target row")
assert(manifestTargetRows[0]?.value === "6 artifacts", "expected target-a artifact count")
assert(
  manifestTargetRows[0]?.detail?.includes("replay x2") === true,
  "expected target-a replay artifact coverage",
)
assert(manifestTargetRows[1]?.label === "target-b", "expected second manifest target row")
assert(manifestTargetRows[1]?.value === "3 artifacts", "expected target-b artifact count")
const manifestRows = sectionRows(manifestSummary, "Artifacts")
assert(manifestRows[0]?.label === "runSummary", "expected run summary manifest row")
assert(manifestRows[0]?.value === "out/summary.json", "expected run summary path")
assert(manifestRows[1]?.detail === "target-a", "expected target id in manifest detail")

const validationArtifact = parseStateFlowArtifact(JSON.stringify(artifactValidation))
assert(validationArtifact.kind === "artifactValidation", "expected artifact validation kind")
const validationSummary = summarizeStateFlowArtifact(validationArtifact)
assert(
  validationSummary.title === "State Flow Artifact Validation",
  "expected artifact validation title",
)
assert(
  validationSummary.metrics.some(metric => metric.label === "Passed" && metric.value === "yes"),
  "expected artifact validation pass metric",
)
const validationTargetRows = sectionRows(validationSummary, "Targets")
assert(validationTargetRows[0]?.label === "target-a", "expected first validation target")
assert(validationTargetRows[1]?.value === "failed", "expected failed validation target")
const validationTargetGateRows = sectionRows(validationSummary, "Target Gate Failures")
assert(validationTargetGateRows[0]?.label === "target-b", "expected failed validation target id")
assert(
  validationTargetGateRows[0]?.value === "missing replay artifact",
  "expected validation target failure reason",
)
const reportArtifact = parseStateFlowArtifact(reportMarkdown)
assert(reportArtifact.kind === "report", "expected report markdown artifact kind")
const reportSummary = summarizeStateFlowArtifact(reportArtifact)
assert(reportSummary.title === "TON State Flow Reverse Report", "expected report title")
assert(
  reportSummary.metrics.some(metric => metric.label === "Sections" && metric.value === "11"),
  "expected report section count metric",
)
const reportTargetRows = sectionRows(reportSummary, "Target")
assert(reportTargetRows[0]?.label === "Network", "expected report target network row")
assert(reportTargetRows[0]?.value === "mainnet", "expected report target network")
const reportOpcodeRows = sectionRows(reportSummary, "Opcode Candidates")
assert(reportOpcodeRows[0]?.label === "0x00000001", "expected report opcode candidate row")
assert(reportOpcodeRows[0]?.value === "low confidence", "expected report opcode confidence")
assert(
  reportOpcodeRows[0]?.detail?.includes("2 transactions") === true,
  "expected report opcode count detail",
)
const reportSectionRows = sectionRows(reportSummary, "Report Sections")
assert(
  reportSectionRows.some(row => row.label === "Schema Evidence" && row.value === "3 lines"),
  "expected report schema evidence section summary",
)
assert(
  reportSectionRows.some(row => row.label === "Replay Diffs"),
  "expected report replay diffs section summary",
)
const reportSchemaEvidenceRows = sectionRows(reportSummary, "Schema Evidence")
assert(
  reportSchemaEvidenceRows[0]?.label === "0x00000001 tx-a",
  "expected report schema evidence row",
)
assert(
  reportSchemaEvidenceRows[0]?.value === "active -> frozen",
  "expected report schema evidence transition",
)
assert(
  reportSchemaEvidenceRows[0]?.detail?.includes("body body-a 32/0") === true,
  "expected report schema evidence body detail",
)
const reportMessageBodyRows = sectionRows(reportSummary, "Message Body Fields")
assert(reportMessageBodyRows[0]?.label === "0x00000001 query_id", "expected report body field row")
assert(reportMessageBodyRows[0]?.value === "uint64", "expected report body field kind")
assert(
  reportMessageBodyRows[0]?.detail?.includes("confidence high") === true,
  "expected report body field confidence detail",
)
const reportReplayProbeRows = sectionRows(reportSummary, "Replay Probes")
assert(reportReplayProbeRows[0]?.label === "0x00000001 query_id", "expected replay probe row")
assert(
  reportReplayProbeRows[0]?.value === "--set-body-uint 32:64:0x6",
  "expected replay probe mutation",
)
const reportStorageRows = sectionRows(reportSummary, "Storage Fields")
assert(reportStorageRows[0]?.label === "0x00000001 data_word_0", "expected storage field row")
assert(reportStorageRows[0]?.value === "data @ 0", "expected storage field location")
assert(
  reportStorageRows[0]?.detail?.includes("sample 0xdeadbeef") === true,
  "expected storage field sample detail",
)
const reportOutboundRows = sectionRows(reportSummary, "Outbound Effects")
assert(
  reportOutboundRows[0]?.label === "0x00000001 outbound internal",
  "expected outbound effect row",
)
assert(reportOutboundRows[0]?.value === "1 effect", "expected outbound effect count")
const reportStateMachineRows = sectionRows(reportSummary, "State Machine")
assert(
  reportStateMachineRows[0]?.label === "active -> frozen",
  "expected report state transition row",
)
assert(reportStateMachineRows[0]?.value === "0x00000001 (2)", "expected report state transition")
const reportReplayDiffRows = sectionRows(reportSummary, "Replay Diffs")
assert(reportReplayDiffRows[0]?.label === "tx-a", "expected report replay tx row")
assert(reportReplayDiffRows[0]?.value === "flip body bit 32", "expected report replay mutation")
assert(
  reportReplayDiffRows[0]?.detail?.includes("accepted true") === true,
  "expected report replay accepted detail",
)
const reportUnknownRows = sectionRows(reportSummary, "Unknown Fields")
assert(reportUnknownRows[0]?.label === "0x00000001", "expected report unknown opcode")
assert(
  reportUnknownRows[0]?.value === "message body field names require TL-B recovery",
  "expected report unknown field value",
)
const reportRiskRows = sectionRows(reportSummary, "Risk Points")
assert(reportRiskRows[0]?.label === "risk 1", "expected report risk row label")
assert(
  reportRiskRows[0]?.value === "Unknown fields remain for opcode 0x00000001.",
  "expected report risk point text",
)

function sectionRows(
  summary: ReturnType<typeof summarizeStateFlowArtifact>,
  title: string,
): ReturnType<typeof summarizeStateFlowArtifact>["sections"][number]["rows"] {
  const section = summary.sections.find(section => section.title === title)
  assert(section !== undefined, `expected ${title} section`)
  return section.rows
}

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(message)
  }
}
