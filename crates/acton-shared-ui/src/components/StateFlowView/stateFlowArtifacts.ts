export type StateFlowArtifact =
  | {readonly kind: "transaction"; readonly data: StateFlowTx}
  | {readonly kind: "corpus"; readonly data: StateFlowCorpus}
  | {readonly kind: "schema"; readonly data: StateFlowSchemaReport}
  | {readonly kind: "replay"; readonly data: StateFlowReplayDiff}

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

export interface ShardAccountSnapshot {
  readonly shardAccountBoc64: string
  readonly lastTransLt: number
  readonly lastTransHash: string
  readonly accountAddress?: string | null
  readonly status: string
  readonly balanceNanotons: string
  readonly codeHash?: string | null
  readonly dataHash?: string | null
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
  readonly inboundBody: {
    readonly minBits: number
    readonly maxBits: number
    readonly minRefs: number
    readonly maxRefs: number
    readonly bodyHashes: readonly string[]
  }
  readonly storage?: StorageShapeCandidate | null
  readonly stateTransitions: readonly StateTransitionCandidate[]
  readonly outboundEffects: readonly EffectCandidate[]
  readonly outActions: readonly EffectCandidate[]
  readonly confidence: string
  readonly unknownFields: readonly string[]
}

export interface StorageShapeCandidate {
  readonly balanceDeltaMin: number
  readonly balanceDeltaMax: number
  readonly dataHashChangedCount: number
  readonly codeHashChangedCount: number
  readonly postDataHashes: readonly string[]
  readonly postCodeHashes: readonly string[]
}

export interface StateTransitionCandidate {
  readonly fromStatus: string
  readonly toStatus: string
  readonly count: number
}

export interface EffectCandidate {
  readonly kind: string
  readonly count: number
}

export type ReplayMutation =
  | {readonly type: "none"}
  | {readonly type: "flipBodyBit"; readonly bit: number}
  | {readonly type: "replaceBody"; readonly bodyBoc64: string}

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
          {label: "VM trace", value: `${tx.vmTrace.lineCount} lines`},
          {label: "Executor trace", value: `${tx.executorTrace.lineCount} lines`},
        ],
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
  return {
    title: "State Flow Schema",
    subtitle: schema.address,
    metrics: [
      {label: "Network", value: schema.network},
      {label: "Transactions", value: schema.transactionCount.toString()},
      {label: "Candidates", value: schema.opcodeCandidates.length.toString()},
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
            `${candidate.unknownFields.length} unknowns`,
          ].join(" · "),
        })),
      },
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
        title: "Deltas",
        rows: [
          {label: "Outbound", value: formatNullable(replay.diff.outboundCountDelta)},
          {label: "Actions", value: formatNullable(replay.diff.actionCountDelta)},
          {label: "Balance", value: formatNullable(replay.diff.balanceDeltaDiff)},
          {label: "c5", value: optionalChangedLabel(replay.diff.c5Changed)},
        ],
      },
      {
        title: "Risk Points",
        rows: replayRiskRows(replay),
      },
    ],
  }
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

function storageLabel(storage: StorageShapeCandidate | null | undefined): string {
  if (!storage) {
    return "n/a"
  }
  const balance =
    storage.balanceDeltaMin === storage.balanceDeltaMax
      ? storage.balanceDeltaMin.toString()
      : `${storage.balanceDeltaMin}..${storage.balanceDeltaMax}`
  return `balance ${balance}, data ${storage.dataHashChangedCount}, code ${storage.codeHashChangedCount}`
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
  }
}

function plural(count: number, singular: string): string {
  return count === 1 ? singular : `${singular}s`
}
