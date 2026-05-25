import {parseStateFlowArtifact, summarizeStateFlowArtifact} from "./stateFlowArtifacts.ts"

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
  stateMachineRows[0]?.detail === "2 observed transitions",
  "expected transition evidence count",
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
