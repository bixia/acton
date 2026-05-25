export type StateFlowArtifact =
  | {readonly kind: "transaction"; readonly data: StateFlowTx}
  | {readonly kind: "corpus"; readonly data: StateFlowCorpus}
  | {readonly kind: "schema"; readonly data: StateFlowSchemaReport}
  | {readonly kind: "replay"; readonly data: StateFlowReplayDiff}
  | {readonly kind: "runSummary"; readonly data: StateFlowRunSummary}
  | {readonly kind: "artifactManifest"; readonly data: StateFlowArtifactManifest}
  | {readonly kind: "artifactValidation"; readonly data: StateFlowArtifactValidation}

export interface ArtifactSummary {
  readonly title: string
  readonly subtitle?: string
  readonly metrics: readonly SummaryMetric[]
  readonly sections: readonly SummarySection[]
}

export interface SummaryMetric {
  readonly label: string
  readonly value: string
}

export interface SummarySection {
  readonly title: string
  readonly rows: readonly SummaryRow[]
}

export interface SummaryRow {
  readonly label: string
  readonly value: string
  readonly detail?: string
}

export interface StateFlowTx {
  readonly schemaVersion: number
  readonly network: string
  readonly queryHash: string
  readonly transaction: {
    readonly lt: number
    readonly utime: number
    readonly account: string
    readonly stateUpdateHashOk: boolean
    readonly transactionBoc64: string
  }
  readonly replay: {
    readonly mcSeqno: number
    readonly randSeedHex: string
    readonly replayedPrevTxCount: number
    readonly blockConfigBoc64: string
    readonly libsBoc64?: string | null
  }
  readonly state: {
    readonly pre: ShardAccountSnapshot
    readonly post: ShardAccountSnapshot
  }
  readonly inbound: MessageArtifact
  readonly outbound: readonly MessageArtifact[]
  readonly compute: StateFlowCompute
  readonly money: MoneyFlow
  readonly c5?: CellArtifact | null
  readonly outActions: readonly ActionEffect[]
  readonly vmTrace: LogArtifact
  readonly executorTrace: LogArtifact
}

export interface StateFlowCorpus {
  readonly schemaVersion: number
  readonly network: string
  readonly address: string
  readonly requestedLimit: number
  readonly sourceTxCount: number
  readonly retracedCount: number
  readonly failureCount: number
  readonly opcodeSummary: readonly OpcodeSummary[]
  readonly transactions: readonly StateFlowTx[]
  readonly failures: readonly StateFlowFailure[]
}

export interface StateFlowSchemaReport {
  readonly schemaVersion: number
  readonly network: string
  readonly address: string
  readonly transactionCount: number
  readonly stateMachine?: StateMachineGraph | null
  readonly auditSignals?: readonly AuditSignal[] | null
  readonly opcodeCandidates: readonly OpcodeSchemaCandidate[]
}

export interface StateMachineGraph {
  readonly edges: readonly StateMachineEdge[]
}

export interface StateMachineEdge {
  readonly fromStatus: string
  readonly toStatus: string
  readonly opcode?: string | null
  readonly count: number
  readonly examples: readonly string[]
}

export interface AuditSignal {
  readonly kind: string
  readonly severity: string
  readonly description: string
  readonly evidence: readonly string[]
}

export interface StateFlowReplayDiff {
  readonly schemaVersion: number
  readonly sourceQueryHash: string
  readonly mutation: ReplayMutation
  readonly ignoreChksig: boolean
  readonly baseline: ReplayObservation
  readonly replay: ReplayObservation
  readonly diff: ReplayDiffSummary
}

export interface StateFlowRunSummary {
  readonly schemaVersion: number
  readonly targetCount: number
  readonly passed: boolean
  readonly absolutePathCount?: number
  readonly gateFailures: readonly string[]
  readonly targets: readonly StateFlowRunTargetSummary[]
}

export interface StateFlowRunTargetSummary {
  readonly id: string
  readonly network: string
  readonly address: string
  readonly sourceUrl?: string | null
  readonly collectLimit: number
  readonly sourceTxCount: number
  readonly retracedCount: number
  readonly failureCount: number
  readonly opcodeCandidateCount: number
  readonly stateEdgeCount: number
  readonly auditSignalCount: number
  readonly replayCount: number
  readonly passed: boolean
  readonly gateFailures: readonly string[]
  readonly outputDir: string
  readonly corpus: string
  readonly schema: string
  readonly transaction?: string | null
  readonly replay?: string | null
  readonly replays?: readonly string[] | null
  readonly report: string
}

export interface StateFlowArtifactManifest {
  readonly schemaVersion: number
  readonly kind: "stateFlowArtifactManifest"
  readonly summary: string
  readonly targetCount: number
  readonly absolutePathCount?: number
  readonly artifacts: readonly StateFlowArtifactManifestEntry[]
}

export interface StateFlowArtifactManifestEntry {
  readonly kind: string
  readonly path: string
  readonly targetId?: string | null
}

export interface StateFlowArtifactValidation {
  readonly schemaVersion: number
  readonly kind: "stateFlowArtifactManifestValidation"
  readonly manifest: string
  readonly targetCount: number
  readonly absolutePathCount: number
  readonly expectedAbsolutePathCount: number
  readonly passed: boolean
  readonly gateFailures: readonly string[]
  readonly targets: readonly StateFlowArtifactValidationTarget[]
}

export interface StateFlowArtifactValidationTarget {
  readonly id: string
  readonly artifactCount: number
  readonly replayCount: number
  readonly passed: boolean
  readonly gateFailures: readonly string[]
}

export interface ShardAccountSnapshot {
  readonly shardAccountBoc64: string
  readonly lastTransLt: number
  readonly lastTransHash: string
  readonly accountAddress?: string | null
  readonly status: string
  readonly balanceNanotons: string
  readonly codeHash?: string | null
  readonly dataHash?: string | null
  readonly codeCell?: CellShape | null
  readonly dataCell?: CellShape | null
  readonly frozenHash?: string | null
}

export interface MessageArtifact {
  readonly direction: "inbound" | "outbound"
  readonly index?: number | null
  readonly kind: string
  readonly src?: string | null
  readonly dst?: string | null
  readonly valueNanotons?: string | null
  readonly bounced?: boolean | null
  readonly bounce?: boolean | null
  readonly opcode?: string | null
  readonly messageBoc64: string
  readonly body: CellArtifact
}

export interface CellArtifact {
  readonly boc64: string
  readonly hash: string
  readonly bits: number
  readonly refs: number
}

export interface CellShape {
  readonly boc64?: string | null
  readonly hash: string
  readonly bits: number
  readonly refs: number
}

export interface StateFlowCompute {
  readonly skipped: boolean
  readonly success?: boolean | null
  readonly exitCode?: number | null
  readonly vmSteps?: number | null
  readonly gasUsed?: number | null
  readonly gasFees?: number | null
}

export interface MoneyFlow {
  readonly balanceBefore: number
  readonly sentTotal: number
  readonly totalFees: number
  readonly balanceAfter: number
}

export interface ActionEffect {
  readonly index: number
  readonly kind: string
  readonly mode?: string | null
  readonly valueNanotons?: string | null
  readonly destination?: string | null
  readonly body?: CellArtifact | null
  readonly code?: CellArtifact | null
  readonly library?: unknown
}

export interface LogArtifact {
  readonly lineCount: number
  readonly text: string
}

export interface OpcodeSummary {
  readonly opcode?: string | null
  readonly count: number
  readonly txHashes: readonly string[]
}

export interface StateFlowFailure {
  readonly hash: string
  readonly lt: number
  readonly error: string
}

export interface OpcodeSchemaCandidate {
  readonly opcode?: string | null
  readonly count: number
  readonly examples: readonly string[]
  readonly evidence?: readonly SchemaEvidence[]
  readonly inboundBody: {
    readonly minBits: number
    readonly maxBits: number
    readonly minRefs: number
    readonly maxRefs: number
    readonly bodyHashes: readonly string[]
    readonly fieldCandidates?: readonly BodyFieldCandidate[] | null
  }
  readonly replayProbes?: readonly ReplayProbeCandidate[] | null
  readonly storage?: StorageShapeCandidate | null
  readonly stateTransitions: readonly StateTransitionCandidate[]
  readonly outboundEffects: readonly EffectCandidate[]
  readonly outActions: readonly EffectCandidate[]
  readonly confidence: string
  readonly unknownFields: readonly string[]
}

export interface ReplayProbeCandidate {
  readonly fieldName: string
  readonly bitOffset: number
  readonly bits: number
  readonly value: string
  readonly mutation: ReplayMutation
  readonly cliArg: string
  readonly confidence: string
  readonly evidence: readonly string[]
}

export interface BodyFieldCandidate {
  readonly name: string
  readonly bitOffset: number
  readonly minBits: number
  readonly maxBits: number
  readonly minRefs: number
  readonly maxRefs: number
  readonly kind: string
  readonly presentCount: number
  readonly valueSamples: readonly string[]
  readonly confidence: string
}

export interface SchemaEvidence {
  readonly txHash: string
  readonly inboundBodyHash: string
  readonly inboundBodyBits: number
  readonly inboundBodyRefs: number
  readonly fromStatus: string
  readonly toStatus: string
  readonly preDataHash?: string | null
  readonly postDataHash?: string | null
  readonly preCodeHash?: string | null
  readonly postCodeHash?: string | null
  readonly outboundKinds: readonly string[]
  readonly outActionKinds: readonly string[]
}

export interface StorageShapeCandidate {
  readonly balanceDeltaMin: number
  readonly balanceDeltaMax: number
  readonly dataHashChangedCount: number
  readonly codeHashChangedCount: number
  readonly postDataShape?: CellShapeRange | null
  readonly postCodeShape?: CellShapeRange | null
  readonly fields?: readonly StorageFieldCandidate[] | null
  readonly postDataHashes: readonly string[]
  readonly postCodeHashes: readonly string[]
}

export interface StorageFieldCandidate {
  readonly name: string
  readonly cellPath: string
  readonly bitOffset: number
  readonly minBits: number
  readonly maxBits: number
  readonly minRefs: number
  readonly maxRefs: number
  readonly kind: string
  readonly presentCount: number
  readonly valueSamples: readonly string[]
  readonly confidence: string
}

export interface CellShapeRange {
  readonly minBits: number
  readonly maxBits: number
  readonly minRefs: number
  readonly maxRefs: number
}

export interface StateTransitionCandidate {
  readonly fromStatus: string
  readonly toStatus: string
  readonly count: number
}

export interface EffectCandidate {
  readonly kind: string
  readonly count: number
  readonly txHashes?: readonly string[] | null
  readonly modes?: readonly string[] | null
  readonly destinations?: readonly string[] | null
  readonly valueNanotonsMin?: string | null
  readonly valueNanotonsMax?: string | null
  readonly bodyShape?: CellShapeRange | null
  readonly codeShape?: CellShapeRange | null
  readonly libraryHashes?: readonly string[] | null
}

export type ReplayMutation =
  | {readonly type: "none"}
  | {readonly type: "flipBodyBit"; readonly bit: number}
  | {readonly type: "replaceBody"; readonly bodyBoc64: string}
  | {
      readonly type: "setBodyUint"
      readonly bitOffset: number
      readonly bits: number
      readonly value: string
    }

export interface ReplayObservation {
  readonly accepted: boolean
  readonly state?: ShardAccountSnapshot | null
  readonly inbound: MessageArtifact
  readonly outbound: readonly MessageArtifact[]
  readonly compute?: StateFlowCompute | null
  readonly money?: MoneyFlow | null
  readonly c5?: CellArtifact | null
  readonly outActions: readonly ActionEffect[]
  readonly vmTrace?: LogArtifact | null
  readonly executorTrace?: LogArtifact | null
  readonly error?: ReplayErrorArtifact | null
}

export interface ReplayErrorArtifact {
  readonly message: string
  readonly externalNotAccepted: boolean
  readonly vmExitCode?: number | null
}

export interface ReplayDiffSummary {
  readonly replayAccepted: boolean
  readonly inputChanged: boolean
  readonly stateChanged?: boolean | null
  readonly codeHashChanged?: boolean | null
  readonly dataHashChanged?: boolean | null
  readonly balanceDeltaDiff?: number | null
  readonly exitCodeChanged?: boolean | null
  readonly outboundCountDelta?: number | null
  readonly actionCountDelta?: number | null
  readonly c5Changed?: boolean | null
}

export function parseStateFlowArtifact(raw: string): StateFlowArtifact {
  const parsed = JSON.parse(raw) as unknown
  if (!isRecord(parsed)) {
    throw new Error("StateFlow artifact must be a JSON object")
  }

  if (Array.isArray(parsed.transactions) && Array.isArray(parsed.opcodeSummary)) {
    return {kind: "corpus", data: parsed as unknown as StateFlowCorpus}
  }
  if (Array.isArray(parsed.opcodeCandidates)) {
    return {kind: "schema", data: parsed as unknown as StateFlowSchemaReport}
  }
  if (isRecord(parsed.diff) && isRecord(parsed.baseline) && isRecord(parsed.replay)) {
    return {kind: "replay", data: parsed as unknown as StateFlowReplayDiff}
  }
  if (
    parsed.kind === "stateFlowArtifactManifest" &&
    typeof parsed.summary === "string" &&
    typeof parsed.targetCount === "number" &&
    Array.isArray(parsed.artifacts)
  ) {
    return {kind: "artifactManifest", data: parsed as unknown as StateFlowArtifactManifest}
  }
  if (
    parsed.kind === "stateFlowArtifactManifestValidation" &&
    typeof parsed.manifest === "string" &&
    typeof parsed.targetCount === "number" &&
    typeof parsed.passed === "boolean" &&
    Array.isArray(parsed.targets)
  ) {
    return {
      kind: "artifactValidation",
      data: parsed as unknown as StateFlowArtifactValidation,
    }
  }
  if (
    typeof parsed.targetCount === "number" &&
    typeof parsed.passed === "boolean" &&
    Array.isArray(parsed.gateFailures) &&
    Array.isArray(parsed.targets)
  ) {
    return {kind: "runSummary", data: parsed as unknown as StateFlowRunSummary}
  }
  if (
    typeof parsed.queryHash === "string" &&
    isRecord(parsed.transaction) &&
    isRecord(parsed.state)
  ) {
    return {kind: "transaction", data: parsed as unknown as StateFlowTx}
  }

  throw new Error("Unsupported StateFlow artifact shape")
}

export function summarizeStateFlowArtifact(artifact: StateFlowArtifact): ArtifactSummary {
  switch (artifact.kind) {
    case "transaction": {
      return summarizeTransaction(artifact.data)
    }
    case "corpus": {
      return summarizeCorpus(artifact.data)
    }
    case "schema": {
      return summarizeSchema(artifact.data)
    }
    case "replay": {
      return summarizeReplay(artifact.data)
    }
    case "runSummary": {
      return summarizeRunSummary(artifact.data)
    }
    case "artifactManifest": {
      return summarizeArtifactManifest(artifact.data)
    }
    case "artifactValidation": {
      return summarizeArtifactValidation(artifact.data)
    }
  }
}

function summarizeTransaction(tx: StateFlowTx): ArtifactSummary {
  return {
    title: "State Flow Transaction",
    subtitle: shortHash(tx.queryHash),
    metrics: [
      {label: "Network", value: tx.network},
      {label: "Opcode", value: tx.inbound.opcode ?? "<none>"},
      {label: "Exit", value: formatNullable(tx.compute.exitCode)},
      {label: "Actions", value: tx.outActions.length.toString()},
      {label: "Outbound", value: tx.outbound.length.toString()},
      {label: "State", value: `${tx.state.pre.status} -> ${tx.state.post.status}`},
    ],
    sections: [
      {
        title: "Evidence",
        rows: [
          {label: "Account", value: tx.transaction.account},
          {label: "LT", value: tx.transaction.lt.toString()},
          {label: "Masterchain seqno", value: tx.replay.mcSeqno.toString()},
          {label: "Prev tx replayed", value: tx.replay.replayedPrevTxCount.toString()},
        ],
      },
      {
        title: "State",
        rows: [snapshotRow("pre", tx.state.pre), snapshotRow("post", tx.state.post)],
      },
      {
        title: "Inbound Message",
        rows: [messageRow(tx.inbound)],
      },
      {
        title: "Outbound Messages",
        rows: tx.outbound.map(message => messageRow(message)),
      },
      {
        title: "Actions",
        rows: transactionActionRows(tx),
      },
      {
        title: "Traces",
        rows: [traceRow("VM trace", tx.vmTrace), traceRow("Executor trace", tx.executorTrace)],
      },
    ],
  }
}

function summarizeCorpus(corpus: StateFlowCorpus): ArtifactSummary {
  return {
    title: "State Flow Corpus",
    subtitle: corpus.address,
    metrics: [
      {label: "Network", value: corpus.network},
      {label: "Source", value: corpus.sourceTxCount.toString()},
      {label: "Retraced", value: corpus.retracedCount.toString()},
      {label: "Failures", value: corpus.failureCount.toString()},
    ],
    sections: [
      {
        title: "Opcodes",
        rows: corpus.opcodeSummary.map(opcode => ({
          label: opcode.opcode ?? "<none>",
          value: `${opcode.count} ${plural(opcode.count, "transaction")}`,
          detail: opcode.txHashes.map(hash => shortHash(hash)).join(", "),
        })),
      },
      {
        title: "State Transitions",
        rows: corpus.transactions.slice(0, 8).map(tx => ({
          label: tx.inbound.opcode ?? "<none>",
          value: `${tx.state.pre.status} -> ${tx.state.post.status}`,
          detail: shortHash(tx.queryHash),
        })),
      },
    ],
  }
}

function summarizeSchema(schema: StateFlowSchemaReport): ArtifactSummary {
  const stateEdges = stateMachineEdges(schema)
  const auditSignals = schemaAuditSignals(schema)
  const bodyFieldRows = schemaBodyFieldRows(schema)
  const storageFieldRows = schemaStorageFieldRows(schema)
  const effectRows = schemaEffectRows(schema)
  const replayProbeRows = schemaReplayProbeRows(schema)
  return {
    title: "State Flow Schema",
    subtitle: schema.address,
    metrics: [
      {label: "Network", value: schema.network},
      {label: "Transactions", value: schema.transactionCount.toString()},
      {label: "Candidates", value: schema.opcodeCandidates.length.toString()},
      {label: "Body Fields", value: bodyFieldRows.length.toString()},
      {label: "Storage Fields", value: storageFieldRows.length.toString()},
      {label: "Effects", value: effectRows.length.toString()},
      {label: "Replay Probes", value: replayProbeRows.length.toString()},
      {label: "State Edges", value: stateEdges.length.toString()},
      {label: "Audit Signals", value: auditSignals.length.toString()},
    ],
    sections: [
      {
        title: "Candidates",
        rows: schema.opcodeCandidates.map(candidate => ({
          label: candidate.opcode ?? "<none>",
          value: `${candidate.confidence} confidence`,
          detail: [
            `${candidate.count} ${plural(candidate.count, "transaction")}`,
            `body ${formatRange(candidate.inboundBody.minBits, candidate.inboundBody.maxBits)} bits`,
            `storage ${storageLabel(candidate.storage)}`,
            `${candidateEvidenceCount(candidate)} ${plural(candidateEvidenceCount(candidate), "evidence row")}`,
            `${candidateBodyFieldCount(candidate)} ${plural(candidateBodyFieldCount(candidate), "body field")}`,
            `${candidateStorageFieldCount(candidate)} ${plural(candidateStorageFieldCount(candidate), "storage field")}`,
            `${candidateEffectCount(candidate)} ${plural(candidateEffectCount(candidate), "effect")}`,
            `${candidateReplayProbeCount(candidate)} ${plural(candidateReplayProbeCount(candidate), "replay probe")}`,
            `${candidate.unknownFields.length} unknowns`,
          ].join(" · "),
        })),
      },
      ...(bodyFieldRows.length > 0
        ? [
            {
              title: "Message Body Fields",
              rows: bodyFieldRows,
            },
          ]
        : []),
      ...(storageFieldRows.length > 0
        ? [
            {
              title: "Storage Fields",
              rows: storageFieldRows,
            },
          ]
        : []),
      ...(effectRows.length > 0
        ? [
            {
              title: "Outbound Effects",
              rows: effectRows,
            },
          ]
        : []),
      ...(replayProbeRows.length > 0
        ? [
            {
              title: "Replay Probes",
              rows: replayProbeRows,
            },
          ]
        : []),
      ...(stateEdges.length > 0
        ? [
            {
              title: "State Machine",
              rows: stateEdges.map(edge => ({
                label: edge.opcode ?? "<none>",
                value: `${edge.fromStatus} -> ${edge.toStatus}`,
                detail: `${edge.count} ${plural(edge.count, "observed transition")}`,
              })),
            },
          ]
        : []),
      ...(auditSignals.length > 0
        ? [
            {
              title: "Risk Points",
              rows: auditSignals.map(signal => ({
                label: `${signal.severity} ${signal.kind}`,
                value: signal.description,
                detail: signal.evidence.map(hash => shortHash(hash)).join(", "),
              })),
            },
          ]
        : []),
    ],
  }
}

function summarizeReplay(replay: StateFlowReplayDiff): ArtifactSummary {
  return {
    title: "State Flow Replay Diff",
    subtitle: shortHash(replay.sourceQueryHash),
    metrics: [
      {label: "Mutation", value: mutationLabel(replay.mutation)},
      {label: "Accepted", value: yesNo(replay.diff.replayAccepted)},
      {label: "Input", value: changedLabel(replay.diff.inputChanged)},
      {label: "State", value: optionalChangedLabel(replay.diff.stateChanged)},
      {label: "Exit", value: optionalChangedLabel(replay.diff.exitCodeChanged)},
    ],
    sections: [
      {
        title: "Replay Observations",
        rows: [
          replayObservationRow("baseline", replay.baseline),
          replayObservationRow("replay", replay.replay),
        ],
      },
      {
        title: "Replay Diff",
        rows: replayDiffRows(replay.diff),
      },
      {
        title: "Risk Points",
        rows: replayRiskRows(replay),
      },
    ],
  }
}

function summarizeRunSummary(summary: StateFlowRunSummary): ArtifactSummary {
  return {
    title: "State Flow Run Summary",
    metrics: [
      {label: "Passed", value: yesNo(summary.passed)},
      {label: "Targets", value: summary.targetCount.toString()},
      {label: "Absolute Paths", value: formatNullable(summary.absolutePathCount)},
      {label: "Gate Failures", value: summary.gateFailures.length.toString()},
      {
        label: "Replays",
        value: summary.targets.reduce((count, target) => count + target.replayCount, 0).toString(),
      },
    ],
    sections: [
      {
        title: "Targets",
        rows: summary.targets.map(target => ({
          label: target.id,
          value: target.passed ? "passed" : "failed",
          detail: [
            target.network,
            `${target.retracedCount}/${target.sourceTxCount} retraced`,
            `${target.failureCount} failures`,
            `opcodes ${target.opcodeCandidateCount}`,
            `state edges ${target.stateEdgeCount}`,
            `audit signals ${target.auditSignalCount}`,
            `replay ${target.replayCount}`,
            `${targetReplayArtifactCount(target)} ${plural(targetReplayArtifactCount(target), "replay artifact")}`,
          ].join(" · "),
        })),
      },
      ...(summary.gateFailures.length > 0
        ? [
            {
              title: "Gate Failures",
              rows: summary.gateFailures.map(failure => formatGateFailureRow(failure)),
            },
          ]
        : []),
    ],
  }
}

function summarizeArtifactManifest(manifest: StateFlowArtifactManifest): ArtifactSummary {
  const targetRows = artifactManifestTargetRows(manifest)
  return {
    title: "State Flow Artifact Manifest",
    subtitle: manifest.summary,
    metrics: [
      {label: "Targets", value: manifest.targetCount.toString()},
      {label: "Artifacts", value: manifest.artifacts.length.toString()},
      {label: "Absolute Paths", value: formatNullable(manifest.absolutePathCount)},
    ],
    sections: [
      ...(targetRows.length > 0
        ? [
            {
              title: "Targets",
              rows: targetRows,
            },
          ]
        : []),
      {
        title: "Artifacts",
        rows: manifest.artifacts.map(artifact => ({
          label: artifact.kind,
          value: artifact.path,
          detail: artifact.targetId ?? undefined,
        })),
      },
    ],
  }
}

function summarizeArtifactValidation(validation: StateFlowArtifactValidation): ArtifactSummary {
  return {
    title: "State Flow Artifact Validation",
    subtitle: validation.manifest,
    metrics: [
      {label: "Passed", value: yesNo(validation.passed)},
      {label: "Targets", value: validation.targetCount.toString()},
      {label: "Gate Failures", value: validation.gateFailures.length.toString()},
      {label: "Absolute Paths", value: validation.absolutePathCount.toString()},
    ],
    sections: [
      {
        title: "Targets",
        rows: validation.targets.map(target => ({
          label: target.id,
          value: target.passed ? "passed" : "failed",
          detail: [
            `${target.artifactCount} ${plural(target.artifactCount, "artifact")}`,
            `${target.replayCount} ${plural(target.replayCount, "replay")}`,
          ].join(" · "),
        })),
      },
      ...(validation.gateFailures.length > 0
        ? [
            {
              title: "Gate Failures",
              rows: validation.gateFailures.map(failure => formatGateFailureRow(failure)),
            },
          ]
        : []),
    ],
  }
}

function artifactManifestTargetRows(manifest: StateFlowArtifactManifest): readonly SummaryRow[] {
  const artifactsByTarget = new Map<string, StateFlowArtifactManifestEntry[]>()
  for (const artifact of manifest.artifacts) {
    if (!artifact.targetId) {
      continue
    }
    const artifacts = artifactsByTarget.get(artifact.targetId) ?? []
    artifacts.push(artifact)
    artifactsByTarget.set(artifact.targetId, artifacts)
  }

  return [...artifactsByTarget.entries()].map(([targetId, artifacts]) => ({
    label: targetId,
    value: `${artifacts.length} ${plural(artifacts.length, "artifact")}`,
    detail: artifactKindCoverage(artifacts),
  }))
}

function artifactKindCoverage(artifacts: readonly StateFlowArtifactManifestEntry[]): string {
  const counts = new Map<string, number>()
  for (const artifact of artifacts) {
    counts.set(artifact.kind, (counts.get(artifact.kind) ?? 0) + 1)
  }
  return [...counts.entries()].map(([kind, count]) => `${kind} x${count}`).join(" · ")
}

function snapshotRow(label: string, snapshot: ShardAccountSnapshot): SummaryRow {
  return {
    label,
    value: snapshot.status,
    detail: [
      `balance ${snapshot.balanceNanotons}`,
      `lt ${snapshot.lastTransLt}`,
      `code ${formatHash(snapshot.codeHash)}`,
      `data ${formatHash(snapshot.dataHash)}`,
    ].join(" · "),
  }
}

function messageRow(message: MessageArtifact): SummaryRow {
  const indexedKind =
    message.index === null || message.index === undefined
      ? message.kind
      : `${message.index} ${message.kind}`
  return {
    label: indexedKind,
    value: message.opcode ?? "<none>",
    detail: [
      `src ${message.src ?? "n/a"}`,
      `dst ${message.dst ?? "n/a"}`,
      `value ${message.valueNanotons ?? "n/a"}`,
      `body ${shortHash(message.body.hash)} ${formatCellShape(message.body)}`,
    ].join(" · "),
  }
}

function transactionActionRows(tx: StateFlowTx): readonly SummaryRow[] {
  const rows: SummaryRow[] = []
  if (tx.c5) {
    rows.push({
      label: "c5",
      value: shortHash(tx.c5.hash),
      detail: formatCellShape(tx.c5),
    })
  }

  rows.push(
    ...tx.outActions.map(action => ({
      label: `${action.index} ${action.kind}`,
      value: action.destination ?? action.valueNanotons ?? "n/a",
      detail: [
        action.mode ? `mode ${action.mode}` : undefined,
        action.valueNanotons ? `value ${action.valueNanotons}` : undefined,
        action.body
          ? `body ${shortHash(action.body.hash)} ${formatCellShape(action.body)}`
          : undefined,
        action.code
          ? `code ${shortHash(action.code.hash)} ${formatCellShape(action.code)}`
          : undefined,
      ]
        .filter((value): value is string => value !== undefined)
        .join(" · "),
    })),
  )

  return rows
}

function traceRow(label: string, trace: LogArtifact): SummaryRow {
  return {
    label,
    value: `${trace.lineCount} ${plural(trace.lineCount, "line")}`,
    detail: firstLogLine(trace),
  }
}

function formatGateFailureRow(failure: string): SummaryRow {
  const separator = failure.indexOf(": ")
  if (separator === -1) {
    return {label: "run", value: failure}
  }

  return {
    label: failure.slice(0, separator),
    value: failure.slice(separator + 2),
  }
}

function targetReplayArtifactCount(target: StateFlowRunTargetSummary): number {
  return target.replays?.length ?? (target.replay ? 1 : 0)
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

function shortHash(value: string): string {
  return value.length <= 16 ? value : `${value.slice(0, 8)}...${value.slice(-6)}`
}

function formatRange(min: number, max: number): string {
  return min === max ? min.toString() : `${min}-${max}`
}

function formatFieldRange(min: number, max: number): string {
  return `${min}..${max}`
}

function storageLabel(storage: StorageShapeCandidate | null | undefined): string {
  if (!storage) {
    return "n/a"
  }
  const balance =
    storage.balanceDeltaMin === storage.balanceDeltaMax
      ? storage.balanceDeltaMin.toString()
      : `${storage.balanceDeltaMin}..${storage.balanceDeltaMax}`
  return [
    `balance ${balance}`,
    `data ${storage.dataHashChangedCount}`,
    `code ${storage.codeHashChangedCount}`,
    storage.postDataShape ? `data ${formatCellShapeRange(storage.postDataShape)}` : undefined,
    storage.postCodeShape ? `code ${formatCellShapeRange(storage.postCodeShape)}` : undefined,
  ]
    .filter(value => value !== undefined)
    .join(", ")
}

function formatCellShapeRange(shape: CellShapeRange): string {
  const bits = formatRange(shape.minBits, shape.maxBits)
  const refs = formatRange(shape.minRefs, shape.maxRefs)
  return `${bits}/${refs}`
}

function formatCellShape(cell: CellArtifact): string {
  return `${cell.bits}/${cell.refs}`
}

function formatHash(hash: string | null | undefined): string {
  return hash ? shortHash(hash) : "n/a"
}

function firstLogLine(trace: LogArtifact): string | undefined {
  return trace.text.split(/\r?\n/, 1)[0] || undefined
}

function candidateEvidenceCount(candidate: OpcodeSchemaCandidate): number {
  return candidate.evidence?.length ?? candidate.examples.length
}

function candidateBodyFieldCount(candidate: OpcodeSchemaCandidate): number {
  return candidate.inboundBody.fieldCandidates?.length ?? 0
}

function candidateStorageFieldCount(candidate: OpcodeSchemaCandidate): number {
  return candidate.storage?.fields?.length ?? 0
}

function candidateEffectCount(candidate: OpcodeSchemaCandidate): number {
  return candidate.outboundEffects.length + candidate.outActions.length
}

function candidateReplayProbeCount(candidate: OpcodeSchemaCandidate): number {
  return candidate.replayProbes?.length ?? 0
}

function schemaBodyFieldRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => {
    const opcode = candidate.opcode ?? "<none>"
    return (candidate.inboundBody.fieldCandidates ?? []).map(field => ({
      label: `${opcode} ${field.name}`,
      value: `${field.kind} @${field.bitOffset}`,
      detail: [
        `${formatFieldRange(field.minBits, field.maxBits)} bits`,
        `${formatFieldRange(field.minRefs, field.maxRefs)} refs`,
        `${field.presentCount} ${plural(field.presentCount, "observation")}`,
        field.confidence,
        field.valueSamples.join(", "),
      ]
        .filter(value => value.length > 0)
        .join(" · "),
    }))
  })
}

function schemaStorageFieldRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => {
    const opcode = candidate.opcode ?? "<none>"
    return (candidate.storage?.fields ?? []).map(field => ({
      label: `${opcode} ${field.name}`,
      value: `${field.kind} @${field.cellPath}:${field.bitOffset}`,
      detail: [
        `${formatFieldRange(field.minBits, field.maxBits)} bits`,
        `${formatFieldRange(field.minRefs, field.maxRefs)} refs`,
        `${field.presentCount} ${plural(field.presentCount, "observation")}`,
        field.confidence,
        field.valueSamples.join(", "),
      ]
        .filter(value => value.length > 0)
        .join(" · "),
    }))
  })
}

function schemaEffectRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => [
    ...candidate.outboundEffects.map(effect => effectRow(candidate.opcode, "outbound", effect)),
    ...candidate.outActions.map(effect => effectRow(candidate.opcode, "action", effect)),
  ])
}

function schemaReplayProbeRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => {
    const opcode = candidate.opcode ?? "<none>"
    return (candidate.replayProbes ?? []).map(probe => ({
      label: `${opcode} ${probe.fieldName}`,
      value: probe.cliArg,
      detail: [
        `${probe.bits} bits @${probe.bitOffset}`,
        probe.confidence,
        mutationLabel(probe.mutation),
        probe.evidence.map(hash => shortHash(hash)).join(", "),
      ]
        .filter(value => value.length > 0)
        .join(" · "),
    }))
  })
}

function effectRow(
  opcode: string | null | undefined,
  source: "outbound" | "action",
  effect: EffectCandidate,
): SummaryRow {
  return {
    label: `${opcode ?? "<none>"} ${source}`,
    value: `${effect.kind} x${effect.count}`,
    detail: [
      `value ${effectValueLabel(effect)}`,
      `body ${shapeLabel(effect.bodyShape)}`,
      `code ${shapeLabel(effect.codeShape)}`,
      listLabel("modes", effect.modes),
      listLabel("destinations", effect.destinations),
      listLabel("libraries", effect.libraryHashes),
      listLabel("evidence", effect.txHashes),
    ]
      .filter(value => value !== undefined)
      .join(" · "),
  }
}

function effectValueLabel(effect: EffectCandidate): string {
  if (!effect.valueNanotonsMin || !effect.valueNanotonsMax) {
    return "n/a"
  }
  return effect.valueNanotonsMin === effect.valueNanotonsMax
    ? effect.valueNanotonsMin
    : `${effect.valueNanotonsMin}..${effect.valueNanotonsMax}`
}

function shapeLabel(shape: CellShapeRange | null | undefined): string {
  return shape ? formatCellShapeRange(shape) : "n/a"
}

function listLabel(
  label: string,
  values: readonly string[] | null | undefined,
): string | undefined {
  if (!values || values.length === 0) {
    return undefined
  }
  return `${label} ${values.join(", ")}`
}

function stateMachineEdges(schema: StateFlowSchemaReport): readonly StateMachineEdge[] {
  const structuredEdges = schema.stateMachine?.edges ?? []
  if (structuredEdges.length > 0) {
    return structuredEdges
  }

  return schema.opcodeCandidates.flatMap(candidate =>
    candidate.stateTransitions.map(transition => ({
      fromStatus: transition.fromStatus,
      toStatus: transition.toStatus,
      opcode: candidate.opcode,
      count: transition.count,
      examples: candidate.examples,
    })),
  )
}

function schemaAuditSignals(schema: StateFlowSchemaReport): readonly AuditSignal[] {
  const structuredSignals = schema.auditSignals ?? []
  if (structuredSignals.length > 0) {
    return structuredSignals
  }

  return schema.opcodeCandidates.flatMap(candidate => {
    const opcode = candidate.opcode ?? "<none>"
    const signals: AuditSignal[] = []

    if (candidate.confidence === "low") {
      signals.push({
        kind: "low-confidence-schema",
        severity: "medium",
        description: `Low confidence schema candidate for opcode ${opcode}; body shape varied or evidence is sparse.`,
        evidence: candidate.examples,
      })
    }
    if (candidate.unknownFields.length > 0) {
      signals.push({
        kind: "unknown-fields",
        severity: "medium",
        description: `Unknown fields remain for opcode ${opcode}: ${candidate.unknownFields.join("; ")}.`,
        evidence: candidate.examples,
      })
    }
    if (candidate.storage && candidate.storage.dataHashChangedCount > 0) {
      signals.push({
        kind: "storage-data-hash-change",
        severity: "medium",
        description: `Opcode ${opcode} changed storage data hash in ${candidate.storage.dataHashChangedCount} observed transaction(s).`,
        evidence: candidate.examples,
      })
    }
    if (candidate.storage && candidate.storage.codeHashChangedCount > 0) {
      signals.push({
        kind: "storage-code-hash-change",
        severity: "high",
        description: `Opcode ${opcode} changed code hash in ${candidate.storage.codeHashChangedCount} observed transaction(s).`,
        evidence: candidate.examples,
      })
    }
    if (candidate.outboundEffects.length > 0 || candidate.outActions.length > 0) {
      signals.push({
        kind: "outbound-or-action-effects",
        severity: "medium",
        description: `Opcode ${opcode} produced outbound effects or c5 actions; payload fields still require TL-B recovery.`,
        evidence: candidate.examples,
      })
    }

    return signals
  })
}

function replayObservationRow(label: string, observation: ReplayObservation): SummaryRow {
  return {
    label,
    value: observation.accepted ? "accepted" : "rejected",
    detail: [
      `opcode ${observation.inbound.opcode ?? "<none>"}`,
      `body ${shortHash(observation.inbound.body.hash)}`,
      `outbound ${observation.outbound.length}`,
      `actions ${observation.outActions.length}`,
      `exit ${formatNullable(observation.compute?.exitCode)}`,
      observation.c5 ? `c5 ${shortHash(observation.c5.hash)}` : undefined,
      observation.error ? `error ${observation.error.message}` : undefined,
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }
}

function replayDiffRows(diff: ReplayDiffSummary): readonly SummaryRow[] {
  return [
    {label: "Input", value: changedLabel(diff.inputChanged)},
    {label: "Replay Accepted", value: yesNo(diff.replayAccepted)},
    {label: "State", value: optionalChangedLabel(diff.stateChanged)},
    {label: "Code Hash", value: optionalChangedLabel(diff.codeHashChanged)},
    {label: "Data Hash", value: optionalChangedLabel(diff.dataHashChanged)},
    {label: "Exit Code", value: optionalChangedLabel(diff.exitCodeChanged)},
    {label: "Balance Delta", value: formatNullable(diff.balanceDeltaDiff)},
    {label: "Outbound Delta", value: formatNullable(diff.outboundCountDelta)},
    {label: "Action Delta", value: formatNullable(diff.actionCountDelta)},
    {label: "c5", value: optionalChangedLabel(diff.c5Changed)},
  ]
}

function replayRiskRows(replay: StateFlowReplayDiff): readonly SummaryRow[] {
  const mutation = mutationLabel(replay.mutation)
  const rows: SummaryRow[] = []

  if (replay.diff.inputChanged && replay.diff.replayAccepted) {
    if (replay.diff.stateChanged === true) {
      rows.push({
        label: mutation,
        value: "Mutation changed state",
        detail: shortHash(replay.sourceQueryHash),
      })
    }
    if (
      replay.diff.outboundCountDelta !== undefined &&
      replay.diff.outboundCountDelta !== null &&
      replay.diff.outboundCountDelta !== 0
    ) {
      rows.push({
        label: mutation,
        value: "Mutation changed outbound/action counts",
        detail: `outbound delta ${replay.diff.outboundCountDelta}`,
      })
    }
    if (
      replay.diff.actionCountDelta !== undefined &&
      replay.diff.actionCountDelta !== null &&
      replay.diff.actionCountDelta !== 0
    ) {
      rows.push({
        label: mutation,
        value: "Mutation changed outbound/action counts",
        detail: `action delta ${replay.diff.actionCountDelta}`,
      })
    }
    if (replay.diff.dataHashChanged === true || replay.diff.codeHashChanged === true) {
      rows.push({
        label: mutation,
        value: "Mutation changed storage hashes",
        detail: `data ${optionalChangedLabel(replay.diff.dataHashChanged)}, code ${optionalChangedLabel(replay.diff.codeHashChanged)}`,
      })
    }
  } else if (replay.diff.inputChanged && !replay.diff.replayAccepted) {
    rows.push({
      label: mutation,
      value: "Mutation was rejected",
      detail: shortHash(replay.sourceQueryHash),
    })
  }

  return rows
}

function formatNullable(value: number | string | null | undefined): string {
  return value === null || value === undefined ? "n/a" : value.toString()
}

function yesNo(value: boolean): string {
  return value ? "yes" : "no"
}

function changedLabel(value: boolean): string {
  return value ? "changed" : "same"
}

function optionalChangedLabel(value: boolean | null | undefined): string {
  return value === null || value === undefined ? "n/a" : changedLabel(value)
}

function mutationLabel(mutation: ReplayMutation): string {
  switch (mutation.type) {
    case "none": {
      return "none"
    }
    case "flipBodyBit": {
      return `flip body bit ${mutation.bit}`
    }
    case "replaceBody": {
      return "replace body"
    }
    case "setBodyUint": {
      return `set body uint ${mutation.value} at ${mutation.bitOffset}:${mutation.bits}`
    }
  }
}

function plural(count: number, singular: string): string {
  return count === 1 ? singular : `${singular}s`
}
