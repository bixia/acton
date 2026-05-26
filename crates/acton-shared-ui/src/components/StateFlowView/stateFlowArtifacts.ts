export type StateFlowArtifact =
  | {readonly kind: "transaction"; readonly data: StateFlowTx}
  | {readonly kind: "retrace"; readonly data: StateFlowTx}
  | {readonly kind: "corpus"; readonly data: StateFlowCorpus}
  | {readonly kind: "schema"; readonly data: StateFlowSchemaReport}
  | {readonly kind: "replay"; readonly data: StateFlowReplayDiff}
  | {readonly kind: "runSummary"; readonly data: StateFlowRunSummary}
  | {readonly kind: "artifactManifest"; readonly data: StateFlowArtifactManifest}
  | {readonly kind: "artifactValidation"; readonly data: StateFlowArtifactValidation}
  | {readonly kind: "report"; readonly data: StateFlowReport}

export const STATE_FLOW_ARTIFACT_FILE_ACCEPT =
  "application/json,text/markdown,text/plain,.json,.md,.txt"

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

export interface ParseStateFlowArtifactOptions {
  readonly artifactKind?: string | null
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
  readonly artifactManifest?: string | null
  readonly validation?: string | null
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
  readonly retrace?: string | null
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
  readonly capabilityCount?: number | null
  readonly capabilityPassedCount?: number | null
  readonly capabilityFailedCount?: number | null
  readonly targets: readonly StateFlowArtifactValidationTarget[]
}

export interface StateFlowArtifactValidationTarget {
  readonly id: string
  readonly artifactCount: number
  readonly replayCount: number
  readonly passed: boolean
  readonly gateFailures: readonly string[]
  readonly capabilityCount?: number | null
  readonly capabilityPassedCount?: number | null
  readonly capabilityFailedCount?: number | null
  readonly capabilityChecks?: readonly StateFlowArtifactCapabilityCheck[] | null
}

export interface StateFlowArtifactCapabilityCheck {
  readonly id: string
  readonly label: string
  readonly passed: boolean
  readonly evidence: readonly string[]
}

export interface StateFlowReport {
  readonly markdown: string
  readonly title: string
  readonly lineCount: number
  readonly sections: readonly StateFlowReportSection[]
}

export interface StateFlowReportSection {
  readonly title: string
  readonly body: string
  readonly lineCount: number
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

export function parseStateFlowArtifact(
  raw: string,
  options: ParseStateFlowArtifactOptions = {},
): StateFlowArtifact {
  const report = parseReportMarkdown(raw)
  if (report) {
    return {kind: "report", data: report}
  }

  let parsed: unknown
  try {
    parsed = JSON.parse(raw) as unknown
  } catch {
    throw new Error("Unsupported StateFlow artifact shape")
  }

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
    return {
      kind: options.artifactKind === "retrace" ? "retrace" : "transaction",
      data: parsed as unknown as StateFlowTx,
    }
  }

  throw new Error("Unsupported StateFlow artifact shape")
}

export function parseStateFlowArtifactFromSource(
  raw: string,
  sourceName?: string | null,
): StateFlowArtifact {
  return parseStateFlowArtifact(raw, {
    artifactKind: artifactKindFromSourceName(sourceName),
  })
}

export function summarizeStateFlowArtifact(artifact: StateFlowArtifact): ArtifactSummary {
  switch (artifact.kind) {
    case "transaction": {
      return summarizeTransaction(artifact.data)
    }
    case "retrace": {
      return summarizeTransaction(artifact.data, "State Flow Retrace")
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
    case "report": {
      return summarizeReport(artifact.data)
    }
  }
}

function artifactKindFromSourceName(sourceName: string | null | undefined): string | undefined {
  const fileName = sourceName?.split(/[\\/]/).pop()?.toLowerCase()
  return fileName === "retrace.json" ? "retrace" : undefined
}

function summarizeTransaction(
  tx: StateFlowTx,
  title: "State Flow Transaction" | "State Flow Retrace" = "State Flow Transaction",
): ArtifactSummary {
  return {
    title,
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
  const evidenceRows = schemaEvidenceRows(schema)
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
      ...(evidenceRows.length > 0
        ? [
            {
              title: "Schema Evidence",
              rows: evidenceRows,
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
                detail: stateMachineEdgeDetail(edge),
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
  const artifactRows = runSummaryArtifactRows(summary)
  const bundleRows = runSummaryBundleRows(summary)
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
      ...(bundleRows.length > 0
        ? [
            {
              title: "Bundle Artifacts",
              rows: bundleRows,
            },
          ]
        : []),
      ...(artifactRows.length > 0
        ? [
            {
              title: "Target Artifacts",
              rows: artifactRows,
            },
          ]
        : []),
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
  const targetGateFailureRows = validationTargetGateFailureRows(validation)
  const capabilityRows = validationCapabilityRows(validation)
  const capabilityCounts = validationCapabilityCounts(validation, capabilityRows)
  return {
    title: "State Flow Artifact Validation",
    subtitle: validation.manifest,
    metrics: [
      {label: "Passed", value: yesNo(validation.passed)},
      {label: "Targets", value: validation.targetCount.toString()},
      {label: "Gate Failures", value: validation.gateFailures.length.toString()},
      {label: "Absolute Paths", value: validation.absolutePathCount.toString()},
      {label: "Capability Checks", value: capabilityCounts.total.toString()},
      {label: "Capability Failures", value: capabilityCounts.failed.toString()},
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
            targetCapabilityDetail(target),
          ].join(" · "),
        })),
      },
      ...(capabilityRows.length > 0
        ? [
            {
              title: "Capability Checks",
              rows: capabilityRows,
            },
          ]
        : []),
      ...(targetGateFailureRows.length > 0
        ? [
            {
              title: "Target Gate Failures",
              rows: targetGateFailureRows,
            },
          ]
        : []),
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

function summarizeReport(report: StateFlowReport): ArtifactSummary {
  const targetSection = report.sections.find(section => section.title === "Target")
  const opcodeCandidateRows = reportOpcodeCandidateRows(report)
  const schemaEvidenceRows = reportSchemaEvidenceRows(report)
  const runtimeEvidenceRows = reportRuntimeEvidenceRows(report)
  const messageBodyFieldRows = reportMessageBodyFieldRows(report)
  const replayProbeRows = reportReplayProbeRows(report)
  const storageFieldRows = reportStorageFieldRows(report)
  const outboundEffectRows = reportOutboundEffectRows(report)
  const stateMachineRows = reportStateMachineRows(report)
  const replayDiffRows = reportReplayDiffRows(report)
  const unknownFieldRows = reportUnknownFieldRows(report)
  const riskPointRows = reportRiskPointRows(report)
  return {
    title: report.title,
    metrics: [
      {label: "Lines", value: report.lineCount.toString()},
      {label: "Sections", value: report.sections.length.toString()},
    ],
    sections: [
      ...(targetSection
        ? [
            {
              title: "Target",
              rows: reportTargetRows(targetSection),
            },
          ]
        : []),
      ...(opcodeCandidateRows.length > 0
        ? [
            {
              title: "Opcode Candidates",
              rows: opcodeCandidateRows,
            },
          ]
        : []),
      ...(schemaEvidenceRows.length > 0
        ? [
            {
              title: "Schema Evidence",
              rows: schemaEvidenceRows,
            },
          ]
        : []),
      ...(runtimeEvidenceRows.length > 0
        ? [
            {
              title: "Runtime Evidence",
              rows: runtimeEvidenceRows,
            },
          ]
        : []),
      ...(messageBodyFieldRows.length > 0
        ? [
            {
              title: "Message Body Fields",
              rows: messageBodyFieldRows,
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
      ...(storageFieldRows.length > 0
        ? [
            {
              title: "Storage Fields",
              rows: storageFieldRows,
            },
          ]
        : []),
      ...(outboundEffectRows.length > 0
        ? [
            {
              title: "Outbound Effects",
              rows: outboundEffectRows,
            },
          ]
        : []),
      ...(stateMachineRows.length > 0
        ? [
            {
              title: "State Machine",
              rows: stateMachineRows,
            },
          ]
        : []),
      ...(replayDiffRows.length > 0
        ? [
            {
              title: "Replay Diffs",
              rows: replayDiffRows,
            },
          ]
        : []),
      ...(unknownFieldRows.length > 0
        ? [
            {
              title: "Unknown Fields",
              rows: unknownFieldRows,
            },
          ]
        : []),
      ...(riskPointRows.length > 0
        ? [
            {
              title: "Risk Points",
              rows: riskPointRows,
            },
          ]
        : []),
      {
        title: "Report Sections",
        rows: report.sections.map(section => ({
          label: section.title,
          value: `${section.lineCount} ${plural(section.lineCount, "line")}`,
          detail: reportSectionPreview(section),
        })),
      },
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

function validationTargetGateFailureRows(
  validation: StateFlowArtifactValidation,
): readonly SummaryRow[] {
  return validation.targets.flatMap(target =>
    target.gateFailures.map(failure => ({
      label: target.id,
      value: failure,
      detail: [
        `${target.artifactCount} ${plural(target.artifactCount, "artifact")}`,
        `${target.replayCount} ${plural(target.replayCount, "replay")}`,
      ].join(" · "),
    })),
  )
}

function validationCapabilityCounts(
  validation: StateFlowArtifactValidation,
  capabilityRows: readonly SummaryRow[],
): {readonly total: number; readonly passed: number; readonly failed: number} {
  const total =
    validation.capabilityCount ??
    validation.targets.reduce(
      (sum, target) => sum + (target.capabilityCount ?? target.capabilityChecks?.length ?? 0),
      0,
    ) ??
    capabilityRows.length
  const failed =
    validation.capabilityFailedCount ??
    validation.targets.reduce(
      (sum, target) =>
        sum +
        (target.capabilityFailedCount ??
          target.capabilityChecks?.filter(check => !check.passed).length ??
          0),
      0,
    )
  const passed =
    validation.capabilityPassedCount ??
    validation.targets.reduce(
      (sum, target) =>
        sum +
        (target.capabilityPassedCount ??
          target.capabilityChecks?.filter(check => check.passed).length ??
          0),
      0,
    )
  return {total, passed, failed}
}

function targetCapabilityDetail(target: StateFlowArtifactValidationTarget): string {
  const total = target.capabilityCount ?? target.capabilityChecks?.length
  const passed =
    target.capabilityPassedCount ?? target.capabilityChecks?.filter(check => check.passed).length
  if (total === undefined || passed === undefined) {
    return "capabilities n/a"
  }
  return `capabilities ${passed}/${total}`
}

function validationCapabilityRows(validation: StateFlowArtifactValidation): readonly SummaryRow[] {
  return validation.targets.flatMap(target =>
    (target.capabilityChecks ?? []).map(check => ({
      label: `${target.id} ${check.label}`,
      value: check.passed ? "passed" : "failed",
      detail: check.evidence.join(" · "),
    })),
  )
}

function artifactKindCoverage(artifacts: readonly StateFlowArtifactManifestEntry[]): string {
  const counts = new Map<string, number>()
  for (const artifact of artifacts) {
    counts.set(artifact.kind, (counts.get(artifact.kind) ?? 0) + 1)
  }
  return [...counts.entries()].map(([kind, count]) => `${kind} x${count}`).join(" · ")
}

function parseReportMarkdown(raw: string): StateFlowReport | undefined {
  const markdown = raw.replaceAll("\r\n", "\n").trimEnd()
  const lines = markdown.split("\n")
  const titleLine = lines.find(line => line.trim().length > 0)
  if (titleLine?.trim() !== "# TON State Flow Reverse Report") {
    return undefined
  }

  const sections: StateFlowReportSection[] = []
  let currentTitle: string | undefined
  let currentBody: string[] = []

  const finishSection = () => {
    if (!currentTitle) {
      return
    }
    sections.push({
      title: currentTitle,
      body: currentBody.join("\n").trim(),
      lineCount: nonEmptyLineCount(currentBody),
    })
  }

  for (const line of lines) {
    if (line.startsWith("## ")) {
      finishSection()
      currentTitle = line.slice(3).trim()
      currentBody = []
      continue
    }

    if (currentTitle) {
      currentBody.push(line)
    }
  }
  finishSection()

  return {
    markdown,
    title: titleLine.trim().slice(2).trim(),
    lineCount: nonEmptyLineCount(lines),
    sections,
  }
}

function reportTargetRows(section: StateFlowReportSection): readonly SummaryRow[] {
  return section.body
    .split("\n")
    .map(line => line.trim().match(/^-\s*([^:]+):\s*(.*)$/))
    .filter((match): match is RegExpMatchArray => match !== null)
    .map(match => ({
      label: stripMarkdownInline(match[1] ?? ""),
      value: stripMarkdownInline(match[2] ?? ""),
    }))
}

function reportOpcodeCandidateRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Opcode Candidates").map(row => {
    const count = rowValue(row, "Count")
    return {
      label: rowValue(row, "Opcode") || "<none>",
      value: `${rowValue(row, "Confidence") || "n/a"} confidence`,
      detail: [
        count.length > 0 ? `${count} ${plural(Number(count), "transaction")}` : undefined,
        tableValueLabel("body bits", rowValue(row, "Body bits")),
        tableValueLabel("body refs", rowValue(row, "Body refs")),
        tableValueLabel("storage", rowValue(row, "Storage")),
        tableValueLabel("state", rowValue(row, "State transitions")),
        tableValueLabel("outbound", rowValue(row, "Outbound effects")),
        tableValueLabel("actions", rowValue(row, "Out actions")),
        tableValueLabel("evidence", rowValue(row, "Evidence")),
      ]
        .filter((value): value is string => value !== undefined)
        .join(" · "),
    }
  })
}

function reportSchemaEvidenceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Schema Evidence").map(row => ({
    label: [rowValue(row, "Opcode"), rowValue(row, "Tx")]
      .filter(value => value.length > 0)
      .join(" "),
    value: rowValue(row, "State") || "n/a",
    detail: [
      `body ${rowValue(row, "Body hash") || "n/a"} ${rowValue(row, "Body bits/refs") || "n/a"}`,
      tableTransitionLabel("data", rowValue(row, "Data hash")),
      tableTransitionLabel("code", rowValue(row, "Code hash")),
      tableValueLabel("out", rowValue(row, "Outbound")),
      tableValueLabel("actions", rowValue(row, "Actions")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportRuntimeEvidenceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Runtime Evidence").map(row => ({
    label: rowValue(row, "Tx") || "n/a",
    value: rowValue(row, "Opcode") || "<none>",
    detail: [
      tableValueLabel("exit", rowValue(row, "Exit")),
      tableValueLabel("VM steps", rowValue(row, "VM steps")),
      tableLineCountLabel("vm trace", rowValue(row, "VM trace lines")),
      tableLineCountLabel("executor trace", rowValue(row, "Executor trace lines")),
      tableValueLabel("c5", rowValue(row, "C5")),
      tableValueLabel("out actions", rowValue(row, "Out actions")),
      tableValueLabel("outbound", rowValue(row, "Outbound messages")),
      tableValueLabel("state", rowValue(row, "State")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportMessageBodyFieldRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Message Body Fields").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Field"]),
    value: rowValue(row, "Kind") || "n/a",
    detail: [
      tableValueLabel("offset", rowValue(row, "Offset")),
      tableValueLabel("bits", rowValue(row, "Bits")),
      tableValueLabel("refs", rowValue(row, "Refs")),
      tableValueLabel("sample", rowValue(row, "Samples")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportReplayProbeRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Replay Probes").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Field"]),
    value: rowValue(row, "CLI mutation") || "n/a",
    detail: [
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportStorageFieldRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Storage Fields").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Field"]),
    value: tableLocationLabel(rowValue(row, "Cell"), rowValue(row, "Offset")),
    detail: [
      tableValueLabel("bits", rowValue(row, "Bits")),
      tableValueLabel("refs", rowValue(row, "Refs")),
      tableValueLabel("kind", rowValue(row, "Kind")),
      tableValueLabel("sample", rowValue(row, "Samples")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportOutboundEffectRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Outbound Effects").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Source", "Kind"]),
    value: tableCountLabel(rowValue(row, "Count"), "effect"),
    detail: [
      tableValueLabel("value", rowValue(row, "Value")),
      tableValueLabel("modes", rowValue(row, "Modes")),
      tableValueLabel("destinations", rowValue(row, "Destinations")),
      tableValueLabel("body", rowValue(row, "Body")),
      tableValueLabel("code", rowValue(row, "Code")),
      tableValueLabel("libraries", rowValue(row, "Libraries")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportStateMachineRows(report: StateFlowReport): readonly SummaryRow[] {
  const section = report.sections.find(section => section.title === "State Machine")
  if (!section) {
    return []
  }

  return section.body
    .split("\n")
    .map(line => line.match(/^\s*(.+?)\s+-->\s+(.+?):\s*(.+)$/))
    .filter((match): match is RegExpMatchArray => match !== null)
    .map(match => ({
      label: `${stripMarkdownInline(match[1] ?? "")} -> ${stripMarkdownInline(match[2] ?? "")}`,
      value: stripMarkdownInline(match[3] ?? ""),
    }))
}

function reportReplayDiffRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Replay Diffs").map(row => ({
    label: rowValue(row, "Source tx") || "n/a",
    value: rowValue(row, "Mutation") || "n/a",
    detail: [
      tableValueLabel("accepted", rowValue(row, "Accepted")),
      tableValueLabel("input", rowValue(row, "Input changed")),
      tableValueLabel("state", rowValue(row, "State changed")),
      tableValueLabel("code", rowValue(row, "Code changed")),
      tableValueLabel("data", rowValue(row, "Data changed")),
      tableValueLabel("balance delta", rowValue(row, "Balance delta")),
      tableValueLabel("exit", rowValue(row, "Exit changed")),
      tableValueLabel("outbound delta", rowValue(row, "Outbound delta")),
      tableValueLabel("action delta", rowValue(row, "Action delta")),
      tableValueLabel("c5", rowValue(row, "C5 changed")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportUnknownFieldRows(report: StateFlowReport): readonly SummaryRow[] {
  const section = report.sections.find(section => section.title === "Unknown Fields")
  if (!section) {
    return []
  }

  const rows: SummaryRow[] = []
  let opcode = "n/a"
  for (const line of section.body.split("\n")) {
    const topLevel = line.trim().match(/^-\s*`?([^`:]+)`?:\s*$/)
    if (topLevel) {
      opcode = stripMarkdownInline(topLevel[1] ?? "") || "n/a"
      continue
    }

    const nested = line.match(/^\s+-\s*(.+)$/)
    if (nested) {
      rows.push({
        label: opcode,
        value: stripMarkdownInline(nested[1] ?? ""),
      })
    }
  }

  return rows
}

function reportRiskPointRows(report: StateFlowReport): readonly SummaryRow[] {
  const section = report.sections.find(section => section.title === "Risk Points")
  if (!section) {
    return []
  }

  return section.body
    .split("\n")
    .map(line => line.trim().match(/^-\s*(.+)$/))
    .filter((match): match is RegExpMatchArray => match !== null)
    .map((match, index) => ({
      label: `risk ${index + 1}`,
      value: stripMarkdownInline(match[1] ?? ""),
    }))
}

function reportTableRows(
  report: StateFlowReport,
  sectionTitle: string,
): readonly Map<string, string>[] {
  const section = report.sections.find(section => section.title === sectionTitle)
  if (!section) {
    return []
  }

  const tableRows = section.body
    .split("\n")
    .map(line => line.trim())
    .filter(line => line.startsWith("|") && line.endsWith("|"))
    .map(line => parseMarkdownTableRow(line))
  const header = tableRows[0]
  if (!header) {
    return []
  }

  return tableRows
    .slice(1)
    .filter(values => isMarkdownTableDataRow(values))
    .map(values => {
      const row = new Map<string, string>()
      for (const [index, key] of header.entries()) {
        row.set(key, values[index] ?? "")
      }
      return row
    })
}

function parseMarkdownTableRow(line: string): readonly string[] {
  const cells: string[] = []
  let cell = ""
  let escaped = false

  for (const char of line.slice(1, -1)) {
    if (escaped) {
      cell += char === "|" ? "|" : `\\${char}`
      escaped = false
      continue
    }

    if (char === "\\") {
      escaped = true
      continue
    }

    if (char === "|") {
      cells.push(stripMarkdownInline(cell))
      cell = ""
      continue
    }

    cell += char
  }

  if (escaped) {
    cell += "\\"
  }
  cells.push(stripMarkdownInline(cell))
  return cells
}

function isMarkdownTableDataRow(values: readonly string[]): boolean {
  return values.some(value => !/^:?-{3,}:?$/.test(value))
}

function rowValue(row: ReadonlyMap<string, string>, key: string): string {
  return row.get(key)?.trim() ?? ""
}

function tableRowLabel(row: ReadonlyMap<string, string>, keys: readonly string[]): string {
  return keys
    .map(key => rowValue(row, key))
    .filter(value => value.length > 0)
    .join(" ")
}

function tableLocationLabel(cell: string, offset: string): string {
  if (cell.length > 0 && offset.length > 0) {
    return `${cell} @ ${offset}`
  }
  return cell || offset || "n/a"
}

function tableCountLabel(count: string, singular: string): string {
  const value = Number(count)
  if (Number.isFinite(value)) {
    return `${count} ${plural(value, singular)}`
  }
  return count || `0 ${plural(0, singular)}`
}

function tableValueLabel(label: string, value: string): string | undefined {
  return value.length > 0 ? `${label} ${value}` : undefined
}

function tableLineCountLabel(label: string, value: string): string | undefined {
  if (value.length === 0) {
    return undefined
  }
  const count = Number(value)
  if (Number.isFinite(count)) {
    return `${label} ${value} ${plural(count, "line")}`
  }
  return `${label} ${value}`
}

function tableTransitionLabel(label: string, value: string): string | undefined {
  return value.length > 0 && value !== "n/a" ? `${label} ${value}` : undefined
}

function reportSectionPreview(section: StateFlowReportSection): string | undefined {
  const line = section.body.split("\n").find(line => line.trim().length > 0)
  return line ? stripMarkdownInline(line.trim()) : undefined
}

function stripMarkdownInline(value: string): string {
  return value.replaceAll("`", "").trim()
}

function nonEmptyLineCount(lines: readonly string[]): number {
  return lines.filter(line => line.trim().length > 0).length
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

function runSummaryArtifactRows(summary: StateFlowRunSummary): readonly SummaryRow[] {
  return summary.targets.flatMap(target => targetArtifactRows(target))
}

function runSummaryBundleRows(summary: StateFlowRunSummary): readonly SummaryRow[] {
  return [
    ...(summary.artifactManifest
      ? [{label: "Artifact Manifest", value: summary.artifactManifest}]
      : []),
    ...(summary.validation ? [{label: "Validation", value: summary.validation}] : []),
  ]
}

function targetArtifactRows(target: StateFlowRunTargetSummary): readonly SummaryRow[] {
  const detail = `${target.network} ${target.address}`
  const replayPaths =
    target.replays && target.replays.length > 0 ? target.replays : replayPath(target)
  return [
    {label: `${target.id} corpus`, value: target.corpus, detail},
    {label: `${target.id} schema`, value: target.schema, detail},
    ...(target.transaction
      ? [{label: `${target.id} transaction`, value: target.transaction, detail}]
      : []),
    ...(target.retrace ? [{label: `${target.id} retrace`, value: target.retrace, detail}] : []),
    ...replayPaths.map((path, index) => ({
      label: `${target.id} replay ${index}`,
      value: path,
      detail,
    })),
    {label: `${target.id} report`, value: target.report, detail},
  ]
}

function replayPath(target: StateFlowRunTargetSummary): readonly string[] {
  return target.replay ? [target.replay] : []
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

function schemaEvidenceRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => {
    const opcode = candidate.opcode ?? "<none>"
    return (candidate.evidence ?? []).map(evidence => ({
      label: `${opcode} ${shortHash(evidence.txHash)}`,
      value: `${evidence.fromStatus} -> ${evidence.toStatus}`,
      detail: [
        `body ${shortHash(evidence.inboundBodyHash)} ${evidence.inboundBodyBits}/${evidence.inboundBodyRefs}`,
        `data ${formatHash(evidence.preDataHash)} -> ${formatHash(evidence.postDataHash)}`,
        `code ${formatHash(evidence.preCodeHash)} -> ${formatHash(evidence.postCodeHash)}`,
        listLabel("out", evidence.outboundKinds),
        listLabel("actions", evidence.outActionKinds),
      ]
        .filter((value): value is string => value !== undefined)
        .join(" · "),
    }))
  })
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

function stateMachineEdgeDetail(edge: StateMachineEdge): string {
  return [
    `${edge.count} ${plural(edge.count, "observed transition")}`,
    edge.examples.length > 0
      ? `examples ${edge.examples.map(hash => shortHash(hash)).join(", ")}`
      : undefined,
  ]
    .filter((value): value is string => value !== undefined)
    .join(" · ")
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
