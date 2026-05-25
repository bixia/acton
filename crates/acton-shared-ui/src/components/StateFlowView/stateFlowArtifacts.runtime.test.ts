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
      },
      storage: {
        balanceDeltaMin: -2,
        balanceDeltaMax: 4,
        dataHashChangedCount: 1,
        codeHashChangedCount: 0,
        postDataHashes: ["data-a", "data-b"],
        postCodeHashes: ["code-a"],
      },
      stateTransitions: [{fromStatus: "active", toStatus: "frozen", count: 2}],
      outboundEffects: [{kind: "internal", count: 1}],
      outActions: [{kind: "send_msg", count: 1}],
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

const schemaSummary = summarizeStateFlowArtifact(parseStateFlowArtifact(JSON.stringify(schema)))
assert(
  schemaSummary.sections[0]?.rows[0]?.detail?.includes("1 evidence row") === true,
  "expected candidate summary to include evidence row count",
)
const stateMachineRows = sectionRows(schemaSummary, "State Machine")
assert(stateMachineRows[0]?.label === "0x00000001", "expected opcode on state machine row")
assert(stateMachineRows[0]?.value === "active -> frozen", "expected state transition row")
assert(
  stateMachineRows[0]?.detail === "2 observed transitions",
  "expected transition evidence count",
)

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
