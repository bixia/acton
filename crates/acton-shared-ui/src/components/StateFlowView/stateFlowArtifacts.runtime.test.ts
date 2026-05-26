import {
  parseStateFlowArtifact,
  parseStateFlowArtifactFromSource,
  summarizeStateFlowArtifact,
  STATE_FLOW_ARTIFACT_FILE_ACCEPT,
} from "./stateFlowArtifacts.ts"

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
  opTable: {
    entries: [
      {
        opcode: "0x00000001",
        name: "op::0x00000001",
        sourceFunction: "recv_internal",
        transactionCount: 2,
        bodyMinBits: 32,
        bodyMaxBits: 96,
        bodyMinRefs: 0,
        bodyMaxRefs: 0,
        bodyFieldCount: 2,
        storageFieldCount: 1,
        outboundEffectCount: 1,
        outActionCount: 1,
        stateTransitionCount: 1,
        confidence: "medium",
        evidence: ["tx-a", "tx-b"],
        unknowns: ["message body field names require TL-B recovery"],
      },
    ],
  },
  messageSurface: {
    messages: [
      {
        opcode: "0x00000001",
        name: "op::0x00000001",
        sourceFunction: "recv_internal",
        transactionCount: 2,
        bodyMinBits: 32,
        bodyMaxBits: 96,
        bodyMinRefs: 0,
        bodyMaxRefs: 0,
        fields: [
          {
            name: "opcode",
            kind: "uint32",
            source: "body",
            bitOffset: 0,
            minBits: 32,
            maxBits: 32,
            minRefs: 0,
            maxRefs: 0,
            presentCount: 2,
            valueSamples: ["0x00000001"],
            confidence: "high",
          },
          {
            name: "query_id",
            kind: "uint64",
            source: "body",
            bitOffset: 32,
            minBits: 64,
            maxBits: 64,
            minRefs: 0,
            maxRefs: 0,
            presentCount: 2,
            valueSamples: ["0x0000000000000007", "0x0000000000000008"],
            confidence: "high",
          },
        ],
        unknowns: ["message body field names require TL-B recovery"],
        confidence: "medium",
        evidence: ["tx-a", "tx-b"],
      },
    ],
  },
  replaySurface: {
    probes: [
      {
        opcode: "0x00000001",
        opName: "op::0x00000001",
        fieldName: "query_id",
        fieldKind: "uint64",
        source: "body",
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
  },
  effectSurface: {
    effects: [
      {
        opcode: "0x00000001",
        opName: "op::0x00000001",
        source: "outbound",
        kind: "internal",
        count: 1,
        modes: [],
        destinations: ["out-dst"],
        valueNanotonsMin: "11",
        valueNanotonsMax: "11",
        bodyShape: {minBits: 40, maxBits: 40, minRefs: 1, maxRefs: 1},
        codeShape: null,
        libraryHashes: [],
        confidence: "medium",
        evidence: ["tx-a"],
      },
      {
        opcode: "0x00000001",
        opName: "op::0x00000001",
        source: "action",
        kind: "send-message",
        count: 1,
        modes: ["64"],
        destinations: ["action-dst"],
        valueNanotonsMin: "7",
        valueNanotonsMax: "7",
        bodyShape: {minBits: 32, maxBits: 32, minRefs: 0, maxRefs: 0},
        codeShape: null,
        libraryHashes: [],
        confidence: "medium",
        evidence: ["tx-a"],
      },
    ],
  },
  storageLayout: {
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
        observationCount: 2,
        opcodes: ["0x00000001"],
        valueSamples: ["0xcafebabe", "0xdeadbeef"],
        confidence: "high",
        evidence: ["tx-a", "tx-b"],
      },
    ],
  },
  stateMachine: {
    nodes: [
      {
        status: "active",
        transactionCount: 2,
        preCount: 2,
        postCount: 0,
        confidence: "medium",
        examples: ["tx-a", "tx-b"],
      },
      {
        status: "frozen",
        transactionCount: 2,
        preCount: 0,
        postCount: 2,
        confidence: "medium",
        examples: ["tx-a", "tx-b"],
      },
    ],
    edges: [
      {
        fromStatus: "active",
        toStatus: "frozen",
        opcode: "0x00000001",
        count: 2,
        confidence: "medium",
        examples: ["tx-a", "tx-b"],
      },
    ],
  },
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
      methodSurface: {
        name: "op::0x00000001",
        sourceFunction: "recv_internal",
        opcode: "0x00000001",
        fields: [
          {
            name: "opcode",
            kind: "uint32",
            source: "body",
            bitOffset: 0,
            minBits: 32,
            maxBits: 32,
            minRefs: 0,
            maxRefs: 0,
            confidence: "high",
          },
          {
            name: "query_id",
            kind: "uint64",
            source: "body",
            bitOffset: 32,
            minBits: 64,
            maxBits: 64,
            minRefs: 0,
            maxRefs: 0,
            confidence: "high",
          },
        ],
        unknowns: ["message body field names require TL-B recovery"],
        confidence: "high",
        evidence: ["tx-a"],
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
      unknownFieldEvidence: [
        {
          marker: "message body field names require TL-B recovery",
          confidence: "medium",
          evidence: ["tx-a"],
        },
      ],
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
    exitCodeChanged: true,
    outboundCountDelta: 1,
    actionCountDelta: 0,
    c5Changed: true,
  },
  diffSurface: {
    changes: [
      {
        kind: "state",
        label: "Shard account state",
        baseline: "active",
        replay: "frozen",
        delta: null,
        severity: "high",
        evidence: ["tx-hash"],
      },
      {
        kind: "balanceDelta",
        label: "Balance delta",
        baseline: "-2",
        replay: "2",
        delta: "4",
        severity: "medium",
        evidence: ["tx-hash"],
      },
    ],
  },
  riskSignals: [
    {
      kind: "replay-state-change",
      severity: "high",
      description: "Mutation changed state",
      evidence: ["tx-hash"],
    },
    {
      kind: "replay-outbound-or-action-change",
      severity: "medium",
      description: "Mutation changed outbound/action counts",
      evidence: ["tx-hash"],
    },
    {
      kind: "replay-exit-code-change",
      severity: "medium",
      description: "Mutation changed exit code",
      evidence: ["tx-hash"],
    },
    {
      kind: "replay-c5-change",
      severity: "medium",
      description: "Mutation changed c5/action register",
      evidence: ["tx-hash"],
    },
  ],
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
  artifactManifest: "out/artifacts.json",
  validation: "out/validation.json",
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
      retrace: "out/target-a/retrace.json",
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
      retrace: undefined,
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
    {kind: "retrace", path: "out/target-a/retrace.json", targetId: "target-a"},
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

## Op Table
| Opcode | Name | Source function | Transactions | Body bits | Body refs | Body fields | Storage fields | Effects | State transitions | Confidence | Evidence | Unknowns |
| --- | --- | --- | ---: | --- | --- | ---: | ---: | --- | ---: | --- | --- | --- |
| \`0x00000001\` | \`op::0x00000001\` | recv_internal | 2 | 32..96 | 0..0 | 2 | 1 | outbound 1; actions 1 | 1 | medium | \`tx-a\`, \`tx-b\` | \`message body field names require TL-B recovery\` |

## Message Surface
| Opcode | Name | Source function | Transactions | Body bits | Body refs | Fields | Unknowns | Confidence | Evidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`op::0x00000001\` | recv_internal | 2 | 32..96 | 0..0 | \`opcode:uint32@body:0\`, \`query_id:uint64@body:32\` | \`message body field names require TL-B recovery\` | medium | \`tx-a\`, \`tx-b\` |

## Schema Evidence
| Opcode | Tx | Body hash | Body bits/refs | State |
| --- | --- | --- | ---: | --- |
| \`0x00000001\` | \`tx-a\` | \`body-a\` | 32/0 | active -> frozen |

## Runtime Evidence
| Tx | Opcode | Exit | VM steps | VM trace lines | Executor trace lines | C5 | Out actions | Outbound messages | State |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | --- |
| \`tx-a\` | \`0x00000001\` | 0 | 1004 | 4127 | 15 | 40/2 | 1 | 1 | active -> frozen |

## Message Body Fields
| Opcode | Field | Offset | Bits | Refs | Kind | Samples | Confidence |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`query_id\` | 32 | 64..64 | 0..0 | uint64 | \`0x7\` | high |

## Replay Probes
| Opcode | Field | CLI mutation | Confidence | Evidence |
| --- | --- | --- | --- | --- |
| \`0x00000001\` | \`query_id\` | \`--set-body-uint 32:64:0x6\` | high | \`tx-a\` |

## Replay Surface
| Opcode | Name | Field | Source | Kind | Offset | Bits | Mutation | CLI mutation | Confidence | Evidence |
| --- | --- | --- | --- | --- | ---: | ---: | --- | --- | --- | --- |
| \`0x00000001\` | \`op::0x00000001\` | \`query_id\` | body | uint64 | 32 | 64 | set body uint 0x6 at 32:64 | \`--set-body-uint 32:64:0x6\` | high | \`tx-a\` |

## Storage Fields
| Opcode | Field | Cell | Offset | Bits | Refs | Kind | Samples | Confidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`data_word_0\` | data | 0 | 32..32 | 0..0 | uint32 | \`0xdeadbeef\` | medium |

## Storage Layout
| Field | Cell | Offset | Bits | Refs | Kind | Observations | Opcodes | Samples | Confidence | Evidence |
| --- | --- | ---: | --- | --- | --- | ---: | --- | --- | --- | --- |
| \`data_word_0\` | data | 0 | 32..32 | 0..0 | uint32 | 2 | \`0x00000001\` | \`0xcafebabe\`, \`0xdeadbeef\` | high | \`tx-a\`, \`tx-b\` |

## Effect Surface
| Opcode | Name | Source | Kind | Count | Value | Modes | Destinations | Body | Code | Libraries | Confidence | Evidence |
| --- | --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- | --- | --- |
| \`0x00000001\` | \`op::0x00000001\` | outbound | internal | 1 | 11 | none | \`dst\` | 40/1 | n/a | none | medium | \`tx-a\` |

## Outbound Effects
| Opcode | Source | Kind | Count | Value | Modes | Destinations | Body | Code | Libraries | Evidence |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- | --- | --- |
| \`0x00000001\` | outbound | internal | 1 | 11 | none | \`dst\` | 40/1 | n/a | none | \`tx-a\` |

## State Machine
\`\`\`mermaid
stateDiagram-v2
    active --> frozen: 0x00000001 (2)
\`\`\`

## State Machine Nodes
| Status | Transactions | Pre | Post | Confidence | Evidence |
| --- | ---: | ---: | ---: | --- | --- |
| active | 2 | 2 | 0 | medium | \`tx-a\`, \`tx-b\` |
| frozen | 2 | 0 | 2 | medium | \`tx-a\`, \`tx-b\` |

## State Machine Evidence
| From | To | Opcode | Count | Confidence | Evidence |
| --- | --- | --- | ---: | --- | --- |
| active | frozen | \`0x00000001\` | 2 | medium | \`tx-a\`, \`tx-b\` |

## Replay Diffs
| Source tx | Mutation | Accepted | Input changed | State changed | Code changed | Data changed | Balance delta | Exit changed | Outbound delta | Action delta | C5 changed |
| --- | --- | --- | --- | --- | --- | --- | ---: | --- | ---: | ---: | --- |
| \`tx-a\` | flip body bit 32 | true | true | true | false | true | 4 | false | 0 | 1 | true |

## Replay Diff Surface
| Source tx | Mutation | Kind | Label | Baseline | Replay | Delta | Severity | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| \`tx-a\` | flip body bit 32 | state | Shard account state | active | frozen | n/a | high | \`tx-a\` |

## Unknown Fields
- \`0x00000001\`:
  - message body field names require TL-B recovery (confidence: medium; evidence: \`tx-a\`, \`tx-b\`)

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

const retraceArtifact = parseStateFlowArtifact(JSON.stringify(stateFlowTx), {
  artifactKind: "retrace",
})
assert(retraceArtifact.kind === "retrace", "expected retrace artifact kind")
const retraceSummary = summarizeStateFlowArtifact(retraceArtifact)
assert(retraceSummary.title === "State Flow Retrace", "expected retrace summary title")
assert(
  sectionRows(retraceSummary, "Traces")[0]?.detail === "vm step 1",
  "expected retrace summary to keep VM trace evidence",
)
const retraceFileArtifact = parseStateFlowArtifactFromSource(
  JSON.stringify(stateFlowTx),
  "retrace.json",
)
assert(retraceFileArtifact.kind === "retrace", "expected retrace file source to set artifact kind")
assert(
  summarizeStateFlowArtifact(retraceFileArtifact).title === "State Flow Retrace",
  "expected retrace file source to render as retrace",
)

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
const opTableRows = sectionRows(schemaSummary, "Op Table")
assert(opTableRows[0]?.label === "0x00000001", "expected op table row")
assert(opTableRows[0]?.value === "op::0x00000001", "expected op table name")
assert(
  opTableRows[0]?.detail ===
    "recv_internal · 2 transactions · body 32..96 bits/0..0 refs · fields body 2, storage 1 · effects outbound 1, actions 1 · transitions 1 · confidence medium · evidence tx-a, tx-b · unknowns message body field names require TL-B recovery",
  "expected op table detail",
)
const messageSurfaceRows = sectionRows(schemaSummary, "Message Surface")
assert(
  schemaSummary.metrics.some(metric => metric.label === "Message Surface" && metric.value === "1"),
  "expected schema message surface metric",
)
assert(messageSurfaceRows[0]?.label === "0x00000001 op::0x00000001", "expected message surface row")
assert(messageSurfaceRows[0]?.value === "recv_internal", "expected message surface source")
assert(
  messageSurfaceRows[0]?.detail ===
    "2 transactions · body 32..96 bits/0..0 refs · 2 fields · confidence medium · evidence tx-a, tx-b · unknowns message body field names require TL-B recovery · fields opcode:uint32@body:0, query_id:uint64@body:32",
  "expected message surface detail",
)
const stateMachineRows = sectionRows(schemaSummary, "State Machine")
assert(stateMachineRows[0]?.label === "0x00000001", "expected opcode on state machine row")
assert(stateMachineRows[0]?.value === "active -> frozen", "expected state transition row")
assert(
  stateMachineRows[0]?.detail ===
    "2 observed transitions · confidence medium · examples tx-a, tx-b",
  "expected transition evidence count, confidence, and examples",
)
const stateMachineNodeRows = sectionRows(schemaSummary, "State Machine Nodes")
assert(stateMachineNodeRows[0]?.label === "active", "expected active state node row")
assert(stateMachineNodeRows[0]?.value === "2 transactions", "expected state node transaction count")
assert(
  stateMachineNodeRows[0]?.detail === "pre 2 · post 0 · confidence medium · examples tx-a, tx-b",
  "expected state node counts, confidence, and examples",
)
const bodyFieldRows = sectionRows(schemaSummary, "Message Body Fields")
assert(bodyFieldRows[1]?.label === "0x00000001 query_id", "expected query_id body field row")
assert(bodyFieldRows[1]?.value === "uint64 @32", "expected query_id field offset")
assert(
  bodyFieldRows[1]?.detail?.includes("0x0000000000000007") === true,
  "expected query_id samples in body field row",
)
const methodSurfaceRows = sectionRows(schemaSummary, "Method Surface")
assert(methodSurfaceRows[0]?.label === "0x00000001 op::0x00000001", "expected method surface row")
assert(methodSurfaceRows[0]?.value === "recv_internal", "expected method surface source")
assert(
  methodSurfaceRows[0]?.detail?.includes("query_id:uint64@body:32") === true,
  "expected method surface ordered field detail",
)
const storageFieldRows = sectionRows(schemaSummary, "Storage Fields")
assert(storageFieldRows[0]?.label === "0x00000001 data_word_0", "expected storage field row")
assert(storageFieldRows[0]?.value === "uint32 @data:0", "expected storage field offset")
assert(
  storageFieldRows[0]?.detail?.includes("0xdeadbeef") === true,
  "expected storage field samples in row",
)
const storageLayoutRows = sectionRows(schemaSummary, "Storage Layout")
assert(storageLayoutRows[0]?.label === "data_word_0", "expected storage layout row")
assert(storageLayoutRows[0]?.value === "uint32 @data:0", "expected storage layout location")
assert(
  storageLayoutRows[0]?.detail ===
    "32..32 bits · 0..0 refs · 2 observations · opcodes 0x00000001 · confidence high · evidence tx-a, tx-b · 0xcafebabe, 0xdeadbeef",
  "expected storage layout detail",
)
const effectRows = sectionRows(schemaSummary, "Outbound Effects")
assert(effectRows[0]?.label === "0x00000001 outbound", "expected outbound effect row")
assert(effectRows[0]?.value === "internal x1", "expected outbound effect kind")
assert(effectRows[0]?.detail?.includes("out-dst") === true, "expected outbound destination")
assert(effectRows[1]?.label === "0x00000001 action", "expected action effect row")
assert(effectRows[1]?.value === "send-message x1", "expected action effect kind")
assert(effectRows[1]?.detail?.includes("64") === true, "expected action mode")
const effectSurfaceRows = sectionRows(schemaSummary, "Effect Surface")
assert(
  effectSurfaceRows[0]?.label === "0x00000001 outbound internal",
  "expected effect surface row",
)
assert(effectSurfaceRows[0]?.value === "1 effect", "expected effect surface count")
assert(
  effectSurfaceRows[0]?.detail ===
    "op::0x00000001 · value 11 · body 40/1 · code n/a · destinations out-dst · confidence medium · evidence tx-a",
  "expected effect surface detail",
)
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
const replaySurfaceRows = sectionRows(schemaSummary, "Replay Surface")
assert(
  schemaSummary.metrics.some(metric => metric.label === "Replay Surface" && metric.value === "1"),
  "expected schema replay surface metric",
)
assert(replaySurfaceRows[0]?.label === "0x00000001 query_id", "expected replay surface row")
assert(
  replaySurfaceRows[0]?.value === "--set-body-uint 32:64:0x0000000000000006",
  "expected replay surface CLI arg",
)
assert(
  replaySurfaceRows[0]?.detail ===
    "op::0x00000001 · body uint64 @32:64 · set body uint 0x0000000000000006 at 32:64 · confidence high · evidence tx-a",
  "expected replay surface detail",
)
const schemaUnknownRows = sectionRows(schemaSummary, "Unknown Fields")
assert(schemaUnknownRows[0]?.label === "0x00000001", "expected schema unknown opcode")
assert(
  schemaUnknownRows[0]?.value === "message body field names require TL-B recovery",
  "expected schema unknown field text",
)
assert(
  schemaUnknownRows[0]?.detail === "confidence medium · evidence tx-a",
  "expected schema unknown field confidence and evidence",
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
const replayDiffSurfaceRows = sectionRows(replaySummary, "Replay Diff Surface")
assert(replayDiffSurfaceRows[0]?.label === "state", "expected replay diff surface state row")
assert(
  replayDiffSurfaceRows[0]?.value === "Shard account state",
  "expected replay diff surface label",
)
assert(
  replayDiffSurfaceRows[0]?.detail ===
    "baseline active · replay frozen · severity high · evidence tx-hash",
  "expected replay diff surface detail",
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
assert(
  replayRiskRows.some(row => row.value === "Mutation changed exit code"),
  "expected replay exit-code risk point",
)
assert(
  replayRiskRows.some(row => row.value === "Mutation changed c5/action register"),
  "expected replay c5 risk point",
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
const runTargetSourceRows = sectionRows(runSummaryView, "Target Sources")
assert(runTargetSourceRows[0]?.label === "target-a", "expected first target source row")
assert(runTargetSourceRows[0]?.value === "addr-a", "expected first target source address")
assert(
  runTargetSourceRows[0]?.detail === "mainnet",
  "expected first target source network without source URL",
)
assert(runTargetSourceRows[1]?.label === "target-b", "expected second target source row")
assert(runTargetSourceRows[1]?.value === "addr-b", "expected second target source address")
assert(
  runTargetSourceRows[1]?.detail === "mainnet · https://tonviewer.com/addr-b",
  "expected second target source network and source URL",
)
const runArtifactRows = sectionRows(runSummaryView, "Target Artifacts")
const bundleArtifactRows = sectionRows(runSummaryView, "Bundle Artifacts")
assert(bundleArtifactRows[0]?.label === "Artifact Manifest", "expected manifest bundle row")
assert(bundleArtifactRows[0]?.value === "out/artifacts.json", "expected manifest bundle path")
assert(bundleArtifactRows[1]?.label === "Validation", "expected validation bundle row")
assert(bundleArtifactRows[1]?.value === "out/validation.json", "expected validation bundle path")
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
    row => row.label === "target-a retrace" && row.value === "out/target-a/retrace.json",
  ),
  "expected run summary to include retrace artifact path",
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
  manifestSummary.metrics.some(metric => metric.label === "Artifacts" && metric.value === "11"),
  "expected manifest artifact count metric",
)
assert(
  manifestSummary.metrics.some(metric => metric.label === "Absolute Paths" && metric.value === "0"),
  "expected manifest absolute path count metric",
)
const manifestTargetRows = sectionRows(manifestSummary, "Targets")
assert(manifestTargetRows[0]?.label === "target-a", "expected first manifest target row")
assert(manifestTargetRows[0]?.value === "7 artifacts", "expected target-a artifact count")
assert(
  manifestTargetRows[0]?.detail?.includes("replay x2") === true,
  "expected target-a replay artifact coverage",
)
assert(
  manifestTargetRows[0]?.detail?.includes("retrace x1") === true,
  "expected target-a retrace artifact coverage",
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
assert(
  validationSummary.metrics.some(
    metric => metric.label === "Capability Checks" && metric.value === "2",
  ),
  "expected artifact validation capability metric",
)
assert(
  validationSummary.metrics.some(
    metric => metric.label === "Capability Failures" && metric.value === "0",
  ),
  "expected artifact validation capability failure metric",
)
const validationTargetRows = sectionRows(validationSummary, "Targets")
assert(validationTargetRows[0]?.label === "target-a", "expected first validation target")
assert(validationTargetRows[1]?.value === "failed", "expected failed validation target")
const validationCapabilityRows = sectionRows(validationSummary, "Capability Checks")
assert(
  validationCapabilityRows.some(
    row =>
      row.label === "target-a Replay diff" &&
      row.value === "passed" &&
      row.detail?.includes("replay:out/target-a/replay.json") === true,
  ),
  "expected validation capability rows",
)
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
  STATE_FLOW_ARTIFACT_FILE_ACCEPT.includes(".md") &&
    STATE_FLOW_ARTIFACT_FILE_ACCEPT.includes("text/markdown"),
  "expected artifact file picker to accept report markdown files",
)
assert(
  reportSummary.metrics.some(metric => metric.label === "Sections" && metric.value === "20"),
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
const reportOpTableRows = sectionRows(reportSummary, "Op Table")
assert(reportOpTableRows[0]?.label === "0x00000001", "expected report op table row")
assert(reportOpTableRows[0]?.value === "op::0x00000001", "expected report op table name")
assert(
  reportOpTableRows[0]?.detail ===
    "recv_internal · 2 transactions · body 32..96 bits/0..0 refs · fields body 2, storage 1 · effects outbound 1; actions 1 · transitions 1 · confidence medium · evidence tx-a, tx-b · unknowns message body field names require TL-B recovery",
  "expected report op table detail",
)
const reportMessageSurfaceRows = sectionRows(reportSummary, "Message Surface")
assert(
  reportMessageSurfaceRows[0]?.label === "0x00000001 op::0x00000001",
  "expected report message surface row",
)
assert(
  reportMessageSurfaceRows[0]?.value === "recv_internal",
  "expected report message surface source",
)
assert(
  reportMessageSurfaceRows[0]?.detail ===
    "2 transactions · body 32..96 bits/0..0 refs · fields opcode:uint32@body:0, query_id:uint64@body:32 · confidence medium · evidence tx-a, tx-b · unknowns message body field names require TL-B recovery",
  "expected report message surface detail",
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
const reportRuntimeRows = sectionRows(reportSummary, "Runtime Evidence")
assert(reportRuntimeRows[0]?.label === "tx-a", "expected report runtime evidence tx row")
assert(reportRuntimeRows[0]?.value === "0x00000001", "expected report runtime opcode")
assert(
  reportRuntimeRows[0]?.detail?.includes("vm trace 4127 lines") === true,
  "expected report runtime VM trace detail",
)
assert(
  reportRuntimeRows[0]?.detail?.includes("executor trace 15 lines") === true,
  "expected report runtime executor trace detail",
)
assert(
  reportRuntimeRows[0]?.detail?.includes("c5 40/2") === true,
  "expected report runtime c5 detail",
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
const reportReplaySurfaceRows = sectionRows(reportSummary, "Replay Surface")
assert(
  reportReplaySurfaceRows[0]?.label === "0x00000001 query_id",
  "expected report replay surface row",
)
assert(
  reportReplaySurfaceRows[0]?.value === "--set-body-uint 32:64:0x6",
  "expected report replay surface CLI arg",
)
assert(
  reportReplaySurfaceRows[0]?.detail ===
    "op::0x00000001 · body uint64 @32:64 · mutation set body uint 0x6 at 32:64 · confidence high · evidence tx-a",
  "expected report replay surface detail",
)
const reportStorageRows = sectionRows(reportSummary, "Storage Fields")
assert(reportStorageRows[0]?.label === "0x00000001 data_word_0", "expected storage field row")
assert(reportStorageRows[0]?.value === "data @ 0", "expected storage field location")
assert(
  reportStorageRows[0]?.detail?.includes("sample 0xdeadbeef") === true,
  "expected storage field sample detail",
)
const reportStorageLayoutRows = sectionRows(reportSummary, "Storage Layout")
assert(reportStorageLayoutRows[0]?.label === "data_word_0", "expected report storage layout row")
assert(reportStorageLayoutRows[0]?.value === "uint32 @ data:0", "expected report layout field")
assert(
  reportStorageLayoutRows[0]?.detail ===
    "32..32 bits · 0..0 refs · 2 observations · opcodes 0x00000001 · confidence high · evidence tx-a, tx-b · samples 0xcafebabe, 0xdeadbeef",
  "expected report storage layout detail",
)
const reportEffectSurfaceRows = sectionRows(reportSummary, "Effect Surface")
assert(
  reportEffectSurfaceRows[0]?.label === "0x00000001 outbound internal",
  "expected report effect surface row",
)
assert(reportEffectSurfaceRows[0]?.value === "1 effect", "expected report effect count")
assert(
  reportEffectSurfaceRows[0]?.detail ===
    "op::0x00000001 · value 11 · body 40/1 · code n/a · modes none · destinations dst · libraries none · confidence medium · evidence tx-a",
  "expected report effect surface detail",
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
const reportStateMachineNodeRows = sectionRows(reportSummary, "State Machine Nodes")
assert(reportStateMachineNodeRows[0]?.label === "active", "expected report state node row")
assert(
  reportStateMachineNodeRows[0]?.value === "2 transactions",
  "expected report state node count",
)
assert(
  reportStateMachineNodeRows[0]?.detail ===
    "pre 2 · post 0 · confidence medium · evidence tx-a, tx-b",
  "expected report state node counts, confidence, and evidence",
)
const reportStateMachineEvidenceRows = sectionRows(reportSummary, "State Machine Evidence")
assert(
  reportStateMachineEvidenceRows[0]?.label === "active -> frozen",
  "expected report state machine evidence transition",
)
assert(
  reportStateMachineEvidenceRows[0]?.value === "0x00000001",
  "expected report state machine evidence opcode",
)
assert(
  reportStateMachineEvidenceRows[0]?.detail ===
    "2 transitions · confidence medium · evidence tx-a, tx-b",
  "expected report state machine evidence detail",
)
const reportReplayDiffRows = sectionRows(reportSummary, "Replay Diffs")
assert(reportReplayDiffRows[0]?.label === "tx-a", "expected report replay tx row")
assert(reportReplayDiffRows[0]?.value === "flip body bit 32", "expected report replay mutation")
assert(
  reportReplayDiffRows[0]?.detail?.includes("accepted true") === true,
  "expected report replay accepted detail",
)
assert(
  reportReplayDiffRows[0]?.detail?.includes("data true") === true,
  "expected report replay data hash detail",
)
assert(
  reportReplayDiffRows[0]?.detail?.includes("balance delta 4") === true,
  "expected report replay balance delta detail",
)
assert(
  reportReplayDiffRows[0]?.detail?.includes("c5 true") === true,
  "expected report replay c5 detail",
)
const reportReplayDiffSurfaceRows = sectionRows(reportSummary, "Replay Diff Surface")
assert(
  reportReplayDiffSurfaceRows[0]?.label === "state",
  "expected report replay diff surface kind",
)
assert(
  reportReplayDiffSurfaceRows[0]?.value === "Shard account state",
  "expected report replay diff surface label",
)
assert(
  reportReplayDiffSurfaceRows[0]?.detail ===
    "source tx-a · mutation flip body bit 32 · baseline active · replay frozen · delta n/a · severity high · evidence tx-a",
  "expected report replay diff surface detail",
)
const reportUnknownRows = sectionRows(reportSummary, "Unknown Fields")
assert(reportUnknownRows[0]?.label === "0x00000001", "expected report unknown opcode")
assert(
  reportUnknownRows[0]?.value === "message body field names require TL-B recovery",
  "expected report unknown field value",
)
assert(
  reportUnknownRows[0]?.detail === "confidence medium · evidence tx-a, tx-b",
  "expected report unknown field confidence and evidence detail",
)
const reportRiskRows = sectionRows(reportSummary, "Risk Points")
assert(reportRiskRows[0]?.label === "risk 1", "expected report risk row label")
assert(
  reportRiskRows[0]?.value === "Unknown fields remain for opcode 0x00000001.",
  "expected report risk point text",
)

const reportWithEscapedPipe = `# TON State Flow Reverse Report

## Opcode Candidates
| Opcode | Count | Confidence | Body bits | Body refs | Storage | State transitions | Outbound effects | Out actions | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| \`0x00000001\` | 1 | low | 32 | 0 | data shape 16/1 | active -> active | internal send\\|notify | send_msg | \`tx-a\` |
`
const escapedPipeReport = summarizeStateFlowArtifact(parseStateFlowArtifact(reportWithEscapedPipe))
const escapedPipeOpcodeRows = sectionRows(escapedPipeReport, "Opcode Candidates")
assert(
  escapedPipeOpcodeRows[0]?.detail?.includes("outbound internal send|notify") === true,
  "expected escaped table pipe to stay inside the outbound cell",
)
assert(
  escapedPipeOpcodeRows[0]?.detail?.includes("actions send_msg") === true,
  "expected columns after escaped pipe to remain aligned",
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
