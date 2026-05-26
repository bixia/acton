import {Address} from "@ton/core"

export type StateFlowArtifact =
  | {readonly kind: "transaction"; readonly data: StateFlowTx}
  | {readonly kind: "retrace"; readonly data: StateFlowTx}
  | {readonly kind: "corpus"; readonly data: StateFlowCorpus}
  | {readonly kind: "schema"; readonly data: StateFlowSchemaReport}
  | {readonly kind: "replay"; readonly data: StateFlowReplayDiff}
  | {readonly kind: "runSummary"; readonly data: StateFlowRunSummary}
  | {readonly kind: "artifactManifest"; readonly data: StateFlowArtifactManifest}
  | {readonly kind: "artifactValidation"; readonly data: StateFlowArtifactValidation}
  | {readonly kind: "artifactBundle"; readonly data: StateFlowArtifactBundle}
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

export interface StateFlowArtifactSource {
  readonly name: string
  readonly raw: string
}

export interface StateFlowArtifactBundle {
  readonly sourceCount: number
  readonly manifest?: StateFlowArtifactManifest
  readonly summary?: StateFlowRunSummary
  readonly validation?: StateFlowArtifactValidation
  readonly targets: readonly StateFlowArtifactBundleTarget[]
  readonly loadedArtifacts: readonly StateFlowArtifactBundleEntry[]
  readonly missingArtifacts: readonly StateFlowArtifactManifestEntry[]
  readonly parseFailures: readonly StateFlowArtifactBundleParseFailure[]
}

export interface StateFlowArtifactBundleTarget {
  readonly id: string
  readonly manifestTarget?: StateFlowArtifactManifestTarget
  readonly summaryTarget?: StateFlowRunTargetSummary
  readonly validationTarget?: StateFlowArtifactValidationTarget
  readonly manifestArtifacts: readonly StateFlowArtifactManifestEntry[]
  readonly loadedArtifacts: readonly StateFlowArtifactBundleEntry[]
  readonly missingArtifacts: readonly StateFlowArtifactManifestEntry[]
}

export interface StateFlowArtifactBundleEntry {
  readonly kind: string
  readonly path?: string
  readonly targetId?: string | null
  readonly sourceName: string
  readonly artifactKind: StateFlowArtifact["kind"]
  readonly opcodeCandidateCount?: number
  readonly stateEdgeCount?: number
  readonly auditSignalCount?: number
  readonly unknownFieldCount?: number
  readonly replayRiskSignalCount?: number
}

export interface StateFlowArtifactBundleParseFailure {
  readonly sourceName: string
  readonly error: string
}

interface ParsedStateFlowArtifactSource {
  readonly source: StateFlowArtifactSource
  readonly artifact: StateFlowArtifact
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
  readonly opTable?: OpTableCandidate | null
  readonly messageSurface?: MessageSurfaceCandidate | null
  readonly replaySurface?: ReplaySurfaceCandidate | null
  readonly effectSurface?: EffectSurfaceCandidate | null
  readonly storageLayout?: StorageLayoutCandidate | null
  readonly auditSignals?: readonly AuditSignal[] | null
  readonly opcodeCandidates: readonly OpcodeSchemaCandidate[]
}

export interface StateMachineGraph {
  readonly nodes?: readonly StateMachineNode[] | null
  readonly edges: readonly StateMachineEdge[]
}

export interface StateMachineNode {
  readonly status: string
  readonly transactionCount: number
  readonly preCount: number
  readonly postCount: number
  readonly confidence?: string | null
  readonly examples: readonly string[]
}

export interface StateMachineEdge {
  readonly fromStatus: string
  readonly toStatus: string
  readonly opcode?: string | null
  readonly count: number
  readonly confidence?: string | null
  readonly examples: readonly string[]
  readonly stateEvidence?: readonly StateMachineStateEvidence[] | null
}

export interface StateMachineStateEvidence {
  readonly txHash: string
  readonly preState: string
  readonly postState: string
}

export interface AuditSignal {
  readonly kind: string
  readonly severity: string
  readonly description: string
  readonly evidence: readonly string[]
}

export interface OpTableCandidate {
  readonly entries: readonly OpTableEntry[]
}

export interface OpTableEntry {
  readonly opcode?: string | null
  readonly name: string
  readonly sourceFunction: string
  readonly transactionCount: number
  readonly bodyMinBits: number
  readonly bodyMaxBits: number
  readonly bodyMinRefs: number
  readonly bodyMaxRefs: number
  readonly bodyFieldCount: number
  readonly storageFieldCount: number
  readonly outboundEffectCount: number
  readonly outActionCount: number
  readonly stateTransitionCount: number
  readonly confidence: string
  readonly evidence: readonly string[]
  readonly unknowns: readonly string[]
}

export interface MessageSurfaceCandidate {
  readonly messages: readonly MessageSurfaceMessage[]
}

export interface MessageSurfaceMessage {
  readonly opcode?: string | null
  readonly name: string
  readonly sourceFunction: string
  readonly transactionCount: number
  readonly bodyMinBits: number
  readonly bodyMaxBits: number
  readonly bodyMinRefs: number
  readonly bodyMaxRefs: number
  readonly fields: readonly MessageSurfaceField[]
  readonly unknowns: readonly string[]
  readonly confidence: string
  readonly evidence: readonly string[]
}

export interface MessageSurfaceField {
  readonly name: string
  readonly kind: string
  readonly source: string
  readonly bitOffset: number
  readonly minBits: number
  readonly maxBits: number
  readonly minRefs: number
  readonly maxRefs: number
  readonly presentCount: number
  readonly valueSamples: readonly string[]
  readonly confidence: string
  readonly valueEvidence?: readonly BodyFieldValueEvidence[] | null
}

export interface BodyFieldValueEvidence {
  readonly txHash: string
  readonly value: string
}

export interface ReplaySurfaceCandidate {
  readonly probes: readonly ReplaySurfaceProbe[]
}

export interface ReplaySurfaceProbe {
  readonly opcode?: string | null
  readonly opName: string
  readonly fieldName: string
  readonly fieldKind: string
  readonly source: string
  readonly bitOffset: number
  readonly bits: number
  readonly value: string
  readonly mutation: ReplayMutation
  readonly cliArg: string
  readonly confidence: string
  readonly evidence: readonly string[]
}

export interface EffectSurfaceCandidate {
  readonly effects: readonly EffectSurfaceEntry[]
}

export interface EffectSurfaceEntry {
  readonly opcode?: string | null
  readonly opName: string
  readonly source: string
  readonly kind: string
  readonly count: number
  readonly modes: readonly string[]
  readonly destinations: readonly string[]
  readonly valueNanotonsMin?: string | null
  readonly valueNanotonsMax?: string | null
  readonly bodyShape?: CellShapeRange | null
  readonly codeShape?: CellShapeRange | null
  readonly libraryHashes: readonly string[]
  readonly confidence: string
  readonly evidence: readonly string[]
}

export interface StorageLayoutCandidate {
  readonly fields: readonly StorageLayoutField[]
}

export interface StorageLayoutField {
  readonly name: string
  readonly cellPath: string
  readonly bitOffset: number
  readonly minBits: number
  readonly maxBits: number
  readonly minRefs: number
  readonly maxRefs: number
  readonly kind: string
  readonly observationCount: number
  readonly opcodes: readonly (string | null | undefined)[]
  readonly valueSamples: readonly string[]
  readonly confidence: string
  readonly evidence: readonly string[]
  readonly valueEvidence?: readonly StorageValueEvidence[] | null
}

export interface StorageValueEvidence {
  readonly txHash: string
  readonly opcode?: string | null
  readonly preValue: string
  readonly postValue: string
  readonly changed: boolean
}

export interface StateFlowReplayDiff {
  readonly schemaVersion: number
  readonly sourceQueryHash: string
  readonly mutation: ReplayMutation
  readonly ignoreChksig: boolean
  readonly baseline: ReplayObservation
  readonly replay: ReplayObservation
  readonly diff: ReplayDiffSummary
  readonly diffSurface?: ReplayDiffSurface | null
  readonly riskSignals?: readonly AuditSignal[] | null
}

export interface ReplayDiffSurface {
  readonly changes: readonly ReplayDiffChange[]
}

export interface ReplayDiffChange {
  readonly kind: string
  readonly label: string
  readonly baseline: string
  readonly replay: string
  readonly delta?: string | null
  readonly severity: string
  readonly evidence: readonly string[]
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
  readonly protocol?: string | null
  readonly category?: string | null
  readonly contractType?: string | null
  readonly sourceUrl?: string | null
  readonly notes?: string | null
  readonly collectLimit: number
  readonly sourceTxCount: number
  readonly retracedCount: number
  readonly failureCount: number
  readonly opcodeCandidateCount: number
  readonly stateEdgeCount: number
  readonly auditSignalCount: number
  readonly unknownFieldCount?: number | null
  readonly replayRiskSignalCount?: number | null
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
  readonly targets?: readonly StateFlowArtifactManifestTarget[] | null
  readonly absolutePathCount?: number
  readonly artifacts: readonly StateFlowArtifactManifestEntry[]
}

export interface StateFlowArtifactManifestTarget {
  readonly id: string
  readonly network?: string | null
  readonly address?: string | null
  readonly protocol?: string | null
  readonly category?: string | null
  readonly contractType?: string | null
  readonly sourceUrl?: string | null
  readonly notes?: string | null
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
  readonly network?: string | null
  readonly address?: string | null
  readonly protocol?: string | null
  readonly category?: string | null
  readonly contractType?: string | null
  readonly sourceUrl?: string | null
  readonly notes?: string | null
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
  readonly methodSurface?: MethodSurfaceCandidate | null
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
  readonly unknownFieldEvidence?: readonly UnknownFieldEvidence[] | null
}

export interface MethodSurfaceCandidate {
  readonly name: string
  readonly sourceFunction: string
  readonly opcode?: string | null
  readonly fields: readonly MethodSurfaceField[]
  readonly unknowns: readonly string[]
  readonly confidence: string
  readonly evidence: readonly string[]
}

export interface MethodSurfaceField {
  readonly name: string
  readonly kind: string
  readonly source: string
  readonly bitOffset: number
  readonly minBits: number
  readonly maxBits: number
  readonly minRefs: number
  readonly maxRefs: number
  readonly confidence: string
}

export interface UnknownFieldEvidence {
  readonly marker: string
  readonly confidence: string
  readonly evidence: readonly string[]
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
  readonly valueEvidence?: readonly BodyFieldValueEvidence[] | null
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

  if (
    parsed.kind === "stateFlowArtifactBundle" &&
    Array.isArray(parsed.sources) &&
    parsed.sources.every(source => isArtifactBundleSource(source))
  ) {
    return parseStateFlowArtifactBundleFromSources(parsed.sources)
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

export function parseStateFlowArtifactBundleFromSources(
  sources: readonly StateFlowArtifactSource[],
): StateFlowArtifact {
  if (sources.length === 0) {
    throw new Error("StateFlow artifact bundle is empty")
  }

  const parsedSources = sources.map(source => {
    try {
      return {
        source,
        artifact: parseStateFlowArtifactFromSource(source.raw, source.name),
        error: undefined,
      }
    } catch (error) {
      return {
        source,
        artifact: undefined,
        error: error instanceof Error ? error.message : String(error),
      }
    }
  })
  const artifacts = parsedSources.flatMap(parsed =>
    parsed.artifact ? [{source: parsed.source, artifact: parsed.artifact}] : [],
  )
  const parseFailures = parsedSources.flatMap(parsed =>
    parsed.error ? [{sourceName: parsed.source.name, error: parsed.error}] : [],
  )
  const manifest = artifacts.find(({artifact}) => artifact.kind === "artifactManifest")?.artifact
    .data as StateFlowArtifactManifest | undefined
  const summary = artifacts.find(({artifact}) => artifact.kind === "runSummary")?.artifact.data as
    | StateFlowRunSummary
    | undefined
  const validation = artifacts.find(({artifact}) => artifact.kind === "artifactValidation")
    ?.artifact.data as StateFlowArtifactValidation | undefined

  const loadedArtifacts: StateFlowArtifactBundleEntry[] = []
  const missingArtifacts: StateFlowArtifactManifestEntry[] = []
  if (manifest) {
    for (const manifestArtifact of manifest.artifacts) {
      const manifestTarget = manifest.targets?.find(
        target => target.id === manifestArtifact.targetId,
      )
      const sourceArtifact = findSourceArtifactForManifestArtifact(
        artifacts,
        manifestArtifact,
        manifestTarget,
      )
      if (!sourceArtifact) {
        missingArtifacts.push(manifestArtifact)
        continue
      }
      loadedArtifacts.push({
        kind: manifestArtifact.kind,
        path: manifestArtifact.path,
        targetId: manifestArtifact.targetId,
        sourceName: sourceArtifact.source.name,
        artifactKind: sourceArtifact.artifact.kind,
        ...stateFlowArtifactBundleCounts(sourceArtifact.artifact),
      })
    }
  } else {
    loadedArtifacts.push(
      ...artifacts.map(({source, artifact}) => ({
        kind: artifact.kind,
        sourceName: source.name,
        artifactKind: artifact.kind,
        ...stateFlowArtifactBundleCounts(artifact),
      })),
    )
  }

  const targetIds = new Set<string>()
  for (const target of manifest?.targets ?? []) {
    targetIds.add(target.id)
  }
  for (const artifact of manifest?.artifacts ?? []) {
    if (artifact.targetId) {
      targetIds.add(artifact.targetId)
    }
  }
  for (const target of summary?.targets ?? []) {
    targetIds.add(target.id)
  }
  for (const target of validation?.targets ?? []) {
    targetIds.add(target.id)
  }

  return {
    kind: "artifactBundle",
    data: {
      sourceCount: sources.length,
      manifest,
      summary,
      validation,
      targets: [...targetIds].map(targetId => {
        const manifestTarget = manifest?.targets?.find(target => target.id === targetId)
        const summaryTarget = summary?.targets.find(target => target.id === targetId)
        const validationTarget = validation?.targets.find(target => target.id === targetId)
        const manifestArtifacts =
          manifest?.artifacts.filter(artifact => artifact.targetId === targetId) ?? []
        return {
          id: targetId,
          manifestTarget,
          summaryTarget,
          validationTarget,
          manifestArtifacts,
          loadedArtifacts: loadedArtifacts.filter(artifact => artifact.targetId === targetId),
          missingArtifacts: missingArtifacts.filter(artifact => artifact.targetId === targetId),
        }
      }),
      loadedArtifacts,
      missingArtifacts,
      parseFailures,
    },
  }
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
    case "artifactBundle": {
      return summarizeArtifactBundle(artifact.data)
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

function isArtifactBundleSource(value: unknown): value is StateFlowArtifactSource {
  return isRecord(value) && typeof value.name === "string" && typeof value.raw === "string"
}

function findSourceArtifactForManifestArtifact(
  artifacts: readonly ParsedStateFlowArtifactSource[],
  manifestArtifact: StateFlowArtifactManifestEntry,
  manifestTarget: StateFlowArtifactManifestTarget | undefined,
): ParsedStateFlowArtifactSource | undefined {
  const manifestPath = manifestArtifact.path
  const normalizedPath = normalizeArtifactPath(manifestPath)
  const exact = artifacts.find(({source}) => normalizeArtifactPath(source.name) === normalizedPath)
  if (exact) {
    return exact
  }

  const suffix = artifacts.find(({source}) =>
    normalizeArtifactPath(source.name).endsWith(`/${normalizedPath}`),
  )
  if (suffix) {
    return suffix
  }

  const baseName = artifactPathBaseName(normalizedPath)
  const baseNameMatches = artifacts.filter(
    ({source}) => artifactPathBaseName(normalizeArtifactPath(source.name)) === baseName,
  )
  if (baseNameMatches.length === 1) {
    return baseNameMatches[0]
  }

  const targetCandidates = baseNameMatches.length > 0 ? baseNameMatches : artifacts
  return targetCandidates.find(
    ({artifact}) =>
      artifactMatchesManifestKind(artifact.kind, manifestArtifact.kind) &&
      artifactTargetMatchesManifestTarget(artifact, manifestTarget),
  )
}

function normalizeArtifactPath(path: string): string {
  return path.replaceAll("\\", "/").replace(/^\.\/+/, "")
}

function artifactPathBaseName(path: string): string {
  return path.split("/").pop() ?? path
}

function artifactMatchesManifestKind(
  artifactKind: StateFlowArtifact["kind"],
  manifestKind: string,
): boolean {
  if (artifactKind === "artifactValidation") {
    return manifestKind === "validation"
  }
  if (artifactKind === "artifactManifest") {
    return manifestKind === "manifest"
  }
  if (artifactKind === "artifactBundle") {
    return false
  }
  return artifactKind === manifestKind
}

function artifactTargetMatchesManifestTarget(
  artifact: StateFlowArtifact,
  manifestTarget: StateFlowArtifactManifestTarget | undefined,
): boolean {
  if (!manifestTarget?.address) {
    return false
  }
  const identity = artifactTargetIdentity(artifact)
  if (
    !identity?.address ||
    normalizeArtifactAddress(identity.address) !== normalizeArtifactAddress(manifestTarget.address)
  ) {
    return false
  }
  return !manifestTarget.network || !identity.network || identity.network === manifestTarget.network
}

function normalizeArtifactAddress(address: string): string {
  try {
    return Address.parse(address).toRawString()
  } catch {
    return address
  }
}

function artifactTargetIdentity(
  artifact: StateFlowArtifact,
): Pick<StateFlowArtifactManifestTarget, "network" | "address"> | undefined {
  switch (artifact.kind) {
    case "corpus":
    case "schema": {
      return {
        network: artifact.data.network,
        address: artifact.data.address,
      }
    }
    case "transaction":
    case "retrace": {
      return {
        network: artifact.data.network,
        address:
          artifact.data.transaction.account ??
          artifact.data.state.post.accountAddress ??
          artifact.data.inbound.dst,
      }
    }
    case "replay": {
      return {
        address:
          artifact.data.baseline.state?.accountAddress ??
          artifact.data.replay.state?.accountAddress ??
          artifact.data.baseline.inbound.dst ??
          artifact.data.replay.inbound.dst,
      }
    }
    case "report": {
      return {
        network: reportTargetValue(artifact.data, "Network"),
        address: reportTargetValue(artifact.data, "Address"),
      }
    }
    default: {
      return undefined
    }
  }
}

function reportTargetValue(report: StateFlowReport, label: string): string | undefined {
  const section = report.sections.find(section => section.title === "Target")
  return reportTargetRows(section ?? {title: "Target", body: "", lineCount: 0}).find(
    row => row.label === label,
  )?.value
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
  const stateNodes = stateMachineNodes(schema, stateEdges)
  const auditSignals = schemaAuditSignals(schema)
  const opTableRows = schemaOpTableRows(schema)
  const messageSurfaceRows = schemaMessageSurfaceRows(schema)
  const bodyFieldRows = schemaBodyFieldRows(schema)
  const methodSurfaceRows = schemaMethodSurfaceRows(schema)
  const storageFieldRows = schemaStorageFieldRows(schema)
  const storageLayoutRows = schemaStorageLayoutRows(schema)
  const effectSurfaceRows = schemaEffectSurfaceRows(schema)
  const effectRows = schemaEffectRows(schema)
  const evidenceRows = schemaEvidenceRows(schema)
  const replayProbeRows = schemaReplayProbeRows(schema)
  const replaySurfaceRows = schemaReplaySurfaceRows(schema)
  const unknownFieldRows = schemaUnknownFieldRows(schema)
  return {
    title: "State Flow Schema",
    subtitle: schema.address,
    metrics: [
      {label: "Network", value: schema.network},
      {label: "Transactions", value: schema.transactionCount.toString()},
      {label: "Ops", value: opTableRows.length.toString()},
      {label: "Message Surface", value: messageSurfaceRows.length.toString()},
      {label: "Candidates", value: schema.opcodeCandidates.length.toString()},
      {label: "Body Fields", value: bodyFieldRows.length.toString()},
      {label: "Storage Fields", value: storageFieldRows.length.toString()},
      {label: "Storage Layout Fields", value: storageLayoutRows.length.toString()},
      {label: "Effect Surface", value: effectSurfaceRows.length.toString()},
      {label: "Effects", value: effectRows.length.toString()},
      {label: "Replay Probes", value: replayProbeRows.length.toString()},
      {label: "Replay Surface", value: replaySurfaceRows.length.toString()},
      {label: "State Nodes", value: stateNodes.length.toString()},
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
      ...(opTableRows.length > 0
        ? [
            {
              title: "Op Table",
              rows: opTableRows,
            },
          ]
        : []),
      ...(messageSurfaceRows.length > 0
        ? [
            {
              title: "Message Surface",
              rows: messageSurfaceRows,
            },
          ]
        : []),
      ...(bodyFieldRows.length > 0
        ? [
            {
              title: "Message Body Fields",
              rows: bodyFieldRows,
            },
          ]
        : []),
      ...(methodSurfaceRows.length > 0
        ? [
            {
              title: "Method Surface",
              rows: methodSurfaceRows,
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
      ...(storageLayoutRows.length > 0
        ? [
            {
              title: "Storage Layout",
              rows: storageLayoutRows,
            },
          ]
        : []),
      ...(effectSurfaceRows.length > 0
        ? [
            {
              title: "Effect Surface",
              rows: effectSurfaceRows,
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
      ...(replaySurfaceRows.length > 0
        ? [
            {
              title: "Replay Surface",
              rows: replaySurfaceRows,
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
      ...(stateNodes.length > 0
        ? [
            {
              title: "State Machine Nodes",
              rows: stateNodes.map(node => ({
                label: node.status,
                value: `${node.transactionCount} ${plural(node.transactionCount, "transaction")}`,
                detail: stateMachineNodeDetail(node),
              })),
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
  const diffSurfaceRows = replayDiffSurfaceRows(replay)
  return {
    title: "State Flow Replay Diff",
    subtitle: shortHash(replay.sourceQueryHash),
    metrics: [
      {label: "Mutation", value: mutationLabel(replay.mutation)},
      {label: "Accepted", value: yesNo(replay.diff.replayAccepted)},
      {label: "Input", value: changedLabel(replay.diff.inputChanged)},
      {label: "State", value: optionalChangedLabel(replay.diff.stateChanged)},
      {label: "Exit", value: optionalChangedLabel(replay.diff.exitCodeChanged)},
      {label: "Diff Surface", value: diffSurfaceRows.length.toString()},
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
      ...(diffSurfaceRows.length > 0
        ? [
            {
              title: "Replay Diff Surface",
              rows: diffSurfaceRows,
            },
          ]
        : []),
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
  const targetSourceRows = runSummaryTargetSourceRows(summary)
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
            `unknown fields ${target.unknownFieldCount ?? 0}`,
            `replay risks ${target.replayRiskSignalCount ?? 0}`,
            `replay ${target.replayCount}`,
            `${targetReplayArtifactCount(target)} ${plural(targetReplayArtifactCount(target), "replay artifact")}`,
          ].join(" · "),
        })),
      },
      ...(targetSourceRows.length > 0
        ? [
            {
              title: "Target Sources",
              rows: targetSourceRows,
            },
          ]
        : []),
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
            targetSourceDetail(target),
            `${target.artifactCount} ${plural(target.artifactCount, "artifact")}`,
            `${target.replayCount} ${plural(target.replayCount, "replay")}`,
            targetCapabilityDetail(target),
          ]
            .filter((value): value is string => value !== undefined && value.length > 0)
            .join(" · "),
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

function summarizeArtifactBundle(bundle: StateFlowArtifactBundle): ArtifactSummary {
  const capabilityRows = bundle.validation ? validationCapabilityRows(bundle.validation) : []
  const capabilityCounts = bundle.validation
    ? validationCapabilityCounts(bundle.validation, capabilityRows)
    : {total: 0, passed: 0, failed: 0}
  const coverageRows = bundleCoverageRows(bundle)
  const riskRows = bundleRiskRows(bundle)
  return {
    title: "State Flow Artifact Bundle",
    subtitle: bundle.manifest?.summary,
    metrics: [
      {label: "Sources", value: bundle.sourceCount.toString()},
      {label: "Targets", value: bundle.targets.length.toString()},
      {label: "Loaded", value: bundle.loadedArtifacts.length.toString()},
      {label: "Missing", value: bundle.missingArtifacts.length.toString()},
      {label: "Parse Failures", value: bundle.parseFailures.length.toString()},
      {label: "Capability Checks", value: capabilityCounts.total.toString()},
    ],
    sections: [
      {
        title: "Targets",
        rows: bundle.targets.map(target => ({
          label: target.id,
          value: bundleTargetStatus(target),
          detail: [
            targetSourceDetail(target.manifestTarget ?? target.validationTarget),
            bundleTargetSummaryDetail(target),
            bundleTargetCapabilityDetail(target),
          ]
            .filter((value): value is string => value !== undefined && value.length > 0)
            .join(" · "),
        })),
      },
      ...(coverageRows.length > 0
        ? [
            {
              title: "Coverage Matrix",
              rows: coverageRows,
            },
          ]
        : []),
      ...(riskRows.length > 0
        ? [
            {
              title: "Risk Matrix",
              rows: riskRows,
            },
          ]
        : []),
      {
        title: "Loaded Artifacts",
        rows: bundle.loadedArtifacts.map(artifact => ({
          label: [artifact.targetId ?? "bundle", artifact.kind].join(" "),
          value: artifact.path ?? artifact.sourceName,
          detail: artifact.artifactKind,
        })),
      },
      ...(bundle.missingArtifacts.length > 0
        ? [
            {
              title: "Missing Artifacts",
              rows: bundle.missingArtifacts.map(artifact => ({
                label: [artifact.targetId ?? "bundle", artifact.kind].join(" "),
                value: artifact.path,
              })),
            },
          ]
        : []),
      ...(capabilityRows.length > 0
        ? [
            {
              title: "Capability Checks",
              rows: capabilityRows,
            },
          ]
        : []),
      ...(bundle.parseFailures.length > 0
        ? [
            {
              title: "Parse Failures",
              rows: bundle.parseFailures.map(failure => ({
                label: failure.sourceName,
                value: failure.error,
              })),
            },
          ]
        : []),
    ],
  }
}

function bundleCoverageRows(bundle: StateFlowArtifactBundle): readonly SummaryRow[] {
  const coverage = new Map<
    string,
    {
      readonly protocol: string
      readonly category: string
      readonly contractTypes: Set<string>
      targetCount: number
      artifactCount: number
      replayCount: number
      capabilityCount: number
      capabilityPassedCount: number
    }
  >()
  for (const target of bundle.targets) {
    const context = target.manifestTarget ?? target.validationTarget ?? target.summaryTarget
    const protocol = context?.protocol
    const category = context?.category
    if (!protocol || !category) {
      continue
    }
    const key = `${protocol}\u0000${category}`
    const entry = coverage.get(key) ?? {
      protocol,
      category,
      contractTypes: new Set<string>(),
      targetCount: 0,
      artifactCount: 0,
      replayCount: 0,
      capabilityCount: 0,
      capabilityPassedCount: 0,
    }
    entry.targetCount += 1
    entry.artifactCount += target.loadedArtifacts.length
    entry.replayCount +=
      target.summaryTarget?.replayCount ?? target.validationTarget?.replayCount ?? 0
    entry.capabilityCount +=
      target.validationTarget?.capabilityCount ??
      target.validationTarget?.capabilityChecks?.length ??
      0
    entry.capabilityPassedCount +=
      target.validationTarget?.capabilityPassedCount ??
      target.validationTarget?.capabilityChecks?.filter(check => check.passed).length ??
      0
    if (context.contractType) {
      entry.contractTypes.add(context.contractType)
    }
    coverage.set(key, entry)
  }

  return [...coverage.values()]
    .sort((left, right) =>
      `${left.protocol}/${left.category}`.localeCompare(`${right.protocol}/${right.category}`),
    )
    .map(entry => ({
      label: `${entry.protocol} / ${entry.category}`,
      value: `${entry.targetCount} ${plural(entry.targetCount, "target")}`,
      detail: [
        [...entry.contractTypes].sort().join(", "),
        `${entry.artifactCount} ${plural(entry.artifactCount, "artifact")}`,
        `${entry.replayCount} ${plural(entry.replayCount, "replay")}`,
        `capabilities ${entry.capabilityPassedCount}/${entry.capabilityCount}`,
      ]
        .filter(value => value.length > 0)
        .join(" · "),
    }))
}

function bundleRiskRows(bundle: StateFlowArtifactBundle): readonly SummaryRow[] {
  const risks = new Map<
    string,
    {
      readonly protocol: string
      readonly category: string
      readonly contractTypes: Set<string>
      targetCount: number
      riskTargetCount: number
      auditSignalCount: number
      unknownFieldCount: number
      replayRiskSignalCount: number
    }
  >()
  for (const target of bundle.targets) {
    const context = target.manifestTarget ?? target.validationTarget ?? target.summaryTarget
    const protocol = context?.protocol
    const category = context?.category
    if (!protocol || !category) {
      continue
    }
    const key = `${protocol}\u0000${category}`
    const entry = risks.get(key) ?? {
      protocol,
      category,
      contractTypes: new Set<string>(),
      targetCount: 0,
      riskTargetCount: 0,
      auditSignalCount: 0,
      unknownFieldCount: 0,
      replayRiskSignalCount: 0,
    }
    const targetRisk = bundleTargetRiskCounts(target)
    entry.targetCount += 1
    if (
      targetRisk.auditSignalCount > 0 ||
      targetRisk.unknownFieldCount > 0 ||
      targetRisk.replayRiskSignalCount > 0
    ) {
      entry.riskTargetCount += 1
    }
    entry.auditSignalCount += targetRisk.auditSignalCount
    entry.unknownFieldCount += targetRisk.unknownFieldCount
    entry.replayRiskSignalCount += targetRisk.replayRiskSignalCount
    if (context.contractType) {
      entry.contractTypes.add(context.contractType)
    }
    risks.set(key, entry)
  }

  return [...risks.values()]
    .sort((left, right) =>
      `${left.protocol}/${left.category}`.localeCompare(`${right.protocol}/${right.category}`),
    )
    .map(entry => ({
      label: `${entry.protocol} / ${entry.category}`,
      value: `${entry.targetCount} ${plural(entry.targetCount, "target")}`,
      detail: [
        [...entry.contractTypes].sort().join(", "),
        `${entry.riskTargetCount} ${plural(entry.riskTargetCount, "risk target")}`,
        `audit signals ${entry.auditSignalCount}`,
        `unknown fields ${entry.unknownFieldCount}`,
        `replay risks ${entry.replayRiskSignalCount}`,
      ]
        .filter(value => value.length > 0)
        .join(" · "),
    }))
}

function bundleTargetRiskCounts(target: StateFlowArtifactBundleTarget): {
  readonly auditSignalCount: number
  readonly unknownFieldCount: number
  readonly replayRiskSignalCount: number
} {
  const schemaArtifacts = target.loadedArtifacts.filter(
    artifact => artifact.artifactKind === "schema",
  )
  const schemaAuditSignalCount = schemaArtifacts.reduce(
    (count, artifact) => count + (artifact.auditSignalCount ?? 0),
    0,
  )
  const schemaUnknownFieldCount = schemaArtifacts.reduce(
    (count, artifact) => count + (artifact.unknownFieldCount ?? 0),
    0,
  )
  const replayArtifacts = target.loadedArtifacts.filter(
    artifact => artifact.artifactKind === "replay",
  )
  const replayRiskSignalCount = replayArtifacts.reduce(
    (count, artifact) => count + (artifact.replayRiskSignalCount ?? 0),
    0,
  )
  return {
    auditSignalCount:
      schemaArtifacts.length > 0
        ? schemaAuditSignalCount
        : (target.summaryTarget?.auditSignalCount ?? 0),
    unknownFieldCount:
      schemaArtifacts.length > 0
        ? schemaUnknownFieldCount
        : (target.summaryTarget?.unknownFieldCount ?? 0),
    replayRiskSignalCount:
      replayArtifacts.length > 0
        ? replayRiskSignalCount
        : (target.summaryTarget?.replayRiskSignalCount ?? 0),
  }
}

function bundleTargetAuditCounts(target: StateFlowArtifactBundleTarget): {
  readonly opcodeCandidateCount: number
  readonly stateEdgeCount: number
} {
  const schemaArtifacts = target.loadedArtifacts.filter(
    artifact => artifact.artifactKind === "schema",
  )
  const opcodeCandidateCount = schemaArtifacts.reduce(
    (count, artifact) => count + (artifact.opcodeCandidateCount ?? 0),
    0,
  )
  const stateEdgeCount = schemaArtifacts.reduce(
    (count, artifact) => count + (artifact.stateEdgeCount ?? 0),
    0,
  )
  return {
    opcodeCandidateCount:
      schemaArtifacts.length > 0
        ? opcodeCandidateCount
        : (target.summaryTarget?.opcodeCandidateCount ?? 0),
    stateEdgeCount:
      schemaArtifacts.length > 0 ? stateEdgeCount : (target.summaryTarget?.stateEdgeCount ?? 0),
  }
}

function stateFlowArtifactBundleCounts(artifact: StateFlowArtifact): {
  readonly opcodeCandidateCount?: number
  readonly stateEdgeCount?: number
  readonly auditSignalCount?: number
  readonly unknownFieldCount?: number
  readonly replayRiskSignalCount?: number
} {
  if (artifact.kind === "schema") {
    return {
      opcodeCandidateCount: artifact.data.opcodeCandidates.length,
      stateEdgeCount: stateMachineEdges(artifact.data).length,
      auditSignalCount: schemaAuditSignals(artifact.data).length,
      unknownFieldCount: schemaUnknownFieldCount(artifact.data),
    }
  }
  if (artifact.kind === "replay") {
    return {
      replayRiskSignalCount: replayRiskSignalCount(artifact.data),
    }
  }

  return {}
}

function schemaUnknownFieldCount(schema: StateFlowSchemaReport): number {
  return schema.opcodeCandidates.reduce(
    (count, candidate) => count + candidateUnknownFieldEvidence(candidate).length,
    0,
  )
}

function summarizeReport(report: StateFlowReport): ArtifactSummary {
  const targetSection = report.sections.find(section => section.title === "Target")
  const opTableRows = reportOpTableRows(report)
  const messageSurfaceRows = reportMessageSurfaceRows(report)
  const opcodeCandidateRows = reportOpcodeCandidateRows(report)
  const schemaEvidenceRows = reportSchemaEvidenceRows(report)
  const runtimeEvidenceRows = reportRuntimeEvidenceRows(report)
  const messageBodyFieldRows = reportMessageBodyFieldRows(report)
  const replayProbeRows = reportReplayProbeRows(report)
  const replaySurfaceRows = reportReplaySurfaceRows(report)
  const storageFieldRows = reportStorageFieldRows(report)
  const storageLayoutRows = reportStorageLayoutRows(report)
  const effectSurfaceRows = reportEffectSurfaceRows(report)
  const outboundEffectRows = reportOutboundEffectRows(report)
  const stateMachineRows = reportStateMachineRows(report)
  const stateMachineNodeRows = reportStateMachineNodeRows(report)
  const stateMachineEvidenceRows = reportStateMachineEvidenceRows(report)
  const replayDiffRows = reportReplayDiffRows(report)
  const replayDiffSurfaceRows = reportReplayDiffSurfaceRows(report)
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
      ...(opTableRows.length > 0
        ? [
            {
              title: "Op Table",
              rows: opTableRows,
            },
          ]
        : []),
      ...(messageSurfaceRows.length > 0
        ? [
            {
              title: "Message Surface",
              rows: messageSurfaceRows,
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
      ...(replaySurfaceRows.length > 0
        ? [
            {
              title: "Replay Surface",
              rows: replaySurfaceRows,
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
      ...(storageLayoutRows.length > 0
        ? [
            {
              title: "Storage Layout",
              rows: storageLayoutRows,
            },
          ]
        : []),
      ...(effectSurfaceRows.length > 0
        ? [
            {
              title: "Effect Surface",
              rows: effectSurfaceRows,
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
      ...(stateMachineNodeRows.length > 0
        ? [
            {
              title: "State Machine Nodes",
              rows: stateMachineNodeRows,
            },
          ]
        : []),
      ...(stateMachineEvidenceRows.length > 0
        ? [
            {
              title: "State Machine Evidence",
              rows: stateMachineEvidenceRows,
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
      ...(replayDiffSurfaceRows.length > 0
        ? [
            {
              title: "Replay Diff Surface",
              rows: replayDiffSurfaceRows,
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
  const contextByTarget = new Map((manifest.targets ?? []).map(target => [target.id, target]))
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
    detail: [targetSourceDetail(contextByTarget.get(targetId)), artifactKindCoverage(artifacts)]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
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

function bundleTargetStatus(target: StateFlowArtifactBundleTarget): string {
  const passed = target.validationTarget?.passed ?? target.summaryTarget?.passed
  if (passed === undefined) {
    return target.missingArtifacts.length === 0 ? "loaded" : "partial"
  }
  return passed ? "passed" : "failed"
}

function bundleTargetSummaryDetail(target: StateFlowArtifactBundleTarget): string {
  const riskCounts = bundleTargetRiskCounts(target)
  const auditCounts = bundleTargetAuditCounts(target)
  const parts = [
    target.manifestArtifacts.length > 0
      ? `loaded ${target.loadedArtifacts.length}/${target.manifestArtifacts.length}`
      : `${target.loadedArtifacts.length} loaded`,
  ]
  if (target.summaryTarget) {
    parts.push(
      `${target.summaryTarget.retracedCount}/${target.summaryTarget.sourceTxCount} retraced`,
      `${target.summaryTarget.replayCount} ${plural(target.summaryTarget.replayCount, "replay")}`,
      `opcodes ${auditCounts.opcodeCandidateCount}`,
      `state edges ${auditCounts.stateEdgeCount}`,
      `audit signals ${riskCounts.auditSignalCount}`,
      `unknown fields ${riskCounts.unknownFieldCount}`,
      `replay risks ${riskCounts.replayRiskSignalCount}`,
      `${target.summaryTarget.failureCount} failures`,
    )
  }
  if (target.missingArtifacts.length > 0) {
    parts.push(`${target.missingArtifacts.length} missing`)
  }
  return parts.join(" · ")
}

function bundleTargetCapabilityDetail(target: StateFlowArtifactBundleTarget): string | undefined {
  if (!target.validationTarget) {
    return undefined
  }
  return targetCapabilityDetail(target.validationTarget)
}

function targetSourceDetail(
  target:
    | Pick<
        StateFlowArtifactManifestTarget,
        "network" | "address" | "protocol" | "category" | "contractType" | "sourceUrl" | "notes"
      >
    | undefined,
): string | undefined {
  if (!target) {
    return undefined
  }
  const location = [target.network ?? undefined, target.address ?? undefined]
    .filter((value): value is string => value !== undefined && value.length > 0)
    .join(" ")
  const targetType = [
    target.protocol ?? undefined,
    target.category ?? undefined,
    target.contractType ?? undefined,
  ]
    .filter((value): value is string => value !== undefined && value.length > 0)
    .join(" ")
  return [
    location || undefined,
    targetType || undefined,
    target.sourceUrl ?? undefined,
    target.notes ?? undefined,
  ]
    .filter((value): value is string => value !== undefined && value.length > 0)
    .join(" · ")
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
      tableValueLabel("values", rowValue(row, "Value evidence")),
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

function reportReplaySurfaceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Replay Surface").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Field"]),
    value: rowValue(row, "CLI mutation") || "n/a",
    detail: [
      rowValue(row, "Name"),
      `${rowValue(row, "Source") || "n/a"} ${rowValue(row, "Kind") || "n/a"} @${
        rowValue(row, "Offset") || "n/a"
      }:${rowValue(row, "Bits") || "n/a"}`,
      tableValueLabel("mutation", rowValue(row, "Mutation")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
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

function reportOpTableRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Op Table").map(row => ({
    label: rowValue(row, "Opcode") || "<none>",
    value: rowValue(row, "Name") || "n/a",
    detail: [
      rowValue(row, "Source function"),
      tableCountLabel(rowValue(row, "Transactions"), "transaction"),
      `body ${rowValue(row, "Body bits") || "n/a"} bits/${rowValue(row, "Body refs") || "n/a"} refs`,
      `fields body ${rowValue(row, "Body fields") || "n/a"}, storage ${
        rowValue(row, "Storage fields") || "n/a"
      }`,
      tableValueLabel("effects", rowValue(row, "Effects")),
      tableValueLabel("transitions", rowValue(row, "State transitions")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
      tableValueLabel("unknowns", rowValue(row, "Unknowns")),
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }))
}

function reportMessageSurfaceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Message Surface").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Name"]),
    value: rowValue(row, "Source function") || "n/a",
    detail: [
      tableCountLabel(rowValue(row, "Transactions"), "transaction"),
      `body ${rowValue(row, "Body bits") || "n/a"} bits/${rowValue(row, "Body refs") || "n/a"} refs`,
      tableValueLabel("fields", rowValue(row, "Fields")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
      tableValueLabel("unknowns", rowValue(row, "Unknowns")),
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }))
}

function reportStorageLayoutRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Storage Layout").map(row => ({
    label: rowValue(row, "Field") || "n/a",
    value: `${rowValue(row, "Kind") || "n/a"} @ ${rowValue(row, "Cell") || "n/a"}:${rowValue(row, "Offset") || "n/a"}`,
    detail: [
      `${rowValue(row, "Bits") || "n/a"} bits`,
      `${rowValue(row, "Refs") || "n/a"} refs`,
      tableCountLabel(rowValue(row, "Observations"), "observation"),
      tableValueLabel("opcodes", rowValue(row, "Opcodes")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
      tableValueLabel("values", rowValue(row, "Value evidence")),
      tableValueLabel("samples", rowValue(row, "Samples")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportEffectSurfaceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Effect Surface").map(row => ({
    label: tableRowLabel(row, ["Opcode", "Source", "Kind"]),
    value: tableCountLabel(rowValue(row, "Count"), "effect"),
    detail: [
      rowValue(row, "Name"),
      tableValueLabel("value", rowValue(row, "Value")),
      tableValueLabel("body", rowValue(row, "Body")),
      tableValueLabel("code", rowValue(row, "Code")),
      tableValueLabel("modes", rowValue(row, "Modes")),
      tableValueLabel("destinations", rowValue(row, "Destinations")),
      tableValueLabel("libraries", rowValue(row, "Libraries")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
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

function reportStateMachineEvidenceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "State Machine Evidence").map(row => ({
    label:
      [rowValue(row, "From"), rowValue(row, "To")].filter(value => value.length > 0).join(" -> ") ||
      "n/a",
    value: rowValue(row, "Opcode") || "<none>",
    detail: [
      tableCountLabel(rowValue(row, "Count"), "transition"),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
      tableValueLabel("state", rowValue(row, "State evidence")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function reportStateMachineNodeRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "State Machine Nodes").map(row => ({
    label: rowValue(row, "Status") || "n/a",
    value: tableCountLabel(rowValue(row, "Transactions"), "transaction"),
    detail: [
      tableValueLabel("pre", rowValue(row, "Pre")),
      tableValueLabel("post", rowValue(row, "Post")),
      tableValueLabel("confidence", rowValue(row, "Confidence")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
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

function reportReplayDiffSurfaceRows(report: StateFlowReport): readonly SummaryRow[] {
  return reportTableRows(report, "Replay Diff Surface").map(row => ({
    label: rowValue(row, "Kind") || "n/a",
    value: rowValue(row, "Label") || "n/a",
    detail: [
      tableValueLabel("source", rowValue(row, "Source tx")),
      tableValueLabel("mutation", rowValue(row, "Mutation")),
      tableValueLabel("baseline", rowValue(row, "Baseline")),
      tableValueLabel("replay", rowValue(row, "Replay")),
      tableValueLabel("delta", rowValue(row, "Delta")),
      tableValueLabel("severity", rowValue(row, "Severity")),
      tableValueLabel("evidence", rowValue(row, "Evidence")),
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
      const unknown = parseUnknownFieldLine(nested[1] ?? "")
      rows.push({
        label: opcode,
        value: unknown.value,
        detail: unknown.detail,
      })
    }
  }

  return rows
}

function parseUnknownFieldLine(raw: string): Pick<SummaryRow, "value" | "detail"> {
  const text = stripMarkdownInline(raw)
  const metaMatch = text.match(/^(.*?)\s*\((confidence:\s*[^;()]+;\s*evidence:\s*[^()]+)\)$/)
  if (!metaMatch) {
    return {value: text}
  }
  const value = metaMatch[1]?.trim() ?? text
  const metadata = metaMatch[2] ?? ""
  const details = metadata
    .split(";")
    .map(part => part.trim())
    .map(value => formatUnknownFieldMetadata(value))
    .filter(part => part.length > 0)
  return {
    value,
    detail: details.length > 0 ? details.join(" · ") : undefined,
  }
}

function formatUnknownFieldMetadata(value: string): string {
  const [label, ...rest] = value.split(":")
  const detail = rest.join(":").trim()
  return detail.length > 0 ? `${label.trim()} ${detail}` : value
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

function runSummaryTargetSourceRows(summary: StateFlowRunSummary): readonly SummaryRow[] {
  return summary.targets.map(target => ({
    label: target.id,
    value: target.address,
    detail: [target.network, target.sourceUrl ?? undefined, target.notes ?? undefined]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }))
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

function schemaOpTableRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schemaOpTableEntries(schema).map(entry => ({
    label: formatOpcode(entry.opcode),
    value: entry.name,
    detail: [
      entry.sourceFunction,
      `${entry.transactionCount} ${plural(entry.transactionCount, "transaction")}`,
      `body ${formatFieldRange(entry.bodyMinBits, entry.bodyMaxBits)} bits/${formatFieldRange(
        entry.bodyMinRefs,
        entry.bodyMaxRefs,
      )} refs`,
      `fields body ${entry.bodyFieldCount}, storage ${entry.storageFieldCount}`,
      `effects outbound ${entry.outboundEffectCount}, actions ${entry.outActionCount}`,
      `transitions ${entry.stateTransitionCount}`,
      `confidence ${entry.confidence}`,
      `evidence ${entry.evidence.map(hash => shortHash(hash)).join(", ")}`,
      entry.unknowns.length > 0 ? `unknowns ${entry.unknowns.join("; ")}` : undefined,
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }))
}

function schemaOpTableEntries(schema: StateFlowSchemaReport): readonly OpTableEntry[] {
  const structured = schema.opTable?.entries ?? []
  if (structured.length > 0) {
    return structured
  }
  return schema.opcodeCandidates.map(candidate => opTableEntryFromCandidate(candidate))
}

function opTableEntryFromCandidate(candidate: OpcodeSchemaCandidate): OpTableEntry {
  const opcode = candidate.opcode
  const name = candidate.methodSurface?.name || `op::${formatOpcode(opcode)}`
  const sourceFunction = candidate.methodSurface?.sourceFunction || "recv_internal"
  const unknowns =
    candidate.methodSurface?.unknowns && candidate.methodSurface.unknowns.length > 0
      ? candidate.methodSurface.unknowns
      : candidate.unknownFields
  return {
    opcode,
    name,
    sourceFunction,
    transactionCount: candidate.count,
    bodyMinBits: candidate.inboundBody.minBits,
    bodyMaxBits: candidate.inboundBody.maxBits,
    bodyMinRefs: candidate.inboundBody.minRefs,
    bodyMaxRefs: candidate.inboundBody.maxRefs,
    bodyFieldCount: candidate.inboundBody.fieldCandidates?.length ?? 0,
    storageFieldCount: candidate.storage?.fields?.length ?? 0,
    outboundEffectCount: candidate.outboundEffects.length,
    outActionCount: candidate.outActions.length,
    stateTransitionCount: candidate.stateTransitions.length,
    confidence: candidate.confidence,
    evidence: candidate.examples,
    unknowns,
  }
}

function schemaMessageSurfaceRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schemaMessageSurfaceMessages(schema).map(message => messageSurfaceRow(message))
}

function schemaMessageSurfaceMessages(
  schema: StateFlowSchemaReport,
): readonly MessageSurfaceMessage[] {
  const structured = schema.messageSurface?.messages ?? []
  if (structured.length > 0) {
    return structured
  }
  return schema.opcodeCandidates.map(candidate => messageSurfaceMessageFromCandidate(candidate))
}

function messageSurfaceMessageFromCandidate(
  candidate: OpcodeSchemaCandidate,
): MessageSurfaceMessage {
  const opcode = candidate.opcode
  const unknowns =
    candidate.methodSurface?.unknowns && candidate.methodSurface.unknowns.length > 0
      ? candidate.methodSurface.unknowns
      : candidate.unknownFields
  return {
    opcode,
    name: candidate.methodSurface?.name || `op::${formatOpcode(opcode)}`,
    sourceFunction: candidate.methodSurface?.sourceFunction || "recv_internal",
    transactionCount: candidate.count,
    bodyMinBits: candidate.inboundBody.minBits,
    bodyMaxBits: candidate.inboundBody.maxBits,
    bodyMinRefs: candidate.inboundBody.minRefs,
    bodyMaxRefs: candidate.inboundBody.maxRefs,
    fields: (candidate.inboundBody.fieldCandidates ?? []).map(field =>
      messageSurfaceFieldFromCandidate(field),
    ),
    unknowns,
    confidence: candidate.confidence,
    evidence: candidate.examples,
  }
}

function messageSurfaceFieldFromCandidate(field: BodyFieldCandidate): MessageSurfaceField {
  return {
    name: field.name,
    kind: field.kind,
    source: "body",
    bitOffset: field.bitOffset,
    minBits: field.minBits,
    maxBits: field.maxBits,
    minRefs: field.minRefs,
    maxRefs: field.maxRefs,
    presentCount: field.presentCount,
    valueSamples: field.valueSamples,
    confidence: field.confidence,
    valueEvidence: field.valueEvidence,
  }
}

function messageSurfaceRow(message: MessageSurfaceMessage): SummaryRow {
  return {
    label: `${formatOpcode(message.opcode)} ${message.name}`,
    value: message.sourceFunction,
    detail: [
      `${message.transactionCount} ${plural(message.transactionCount, "transaction")}`,
      `body ${formatFieldRange(message.bodyMinBits, message.bodyMaxBits)} bits/${formatFieldRange(
        message.bodyMinRefs,
        message.bodyMaxRefs,
      )} refs`,
      `${message.fields.length} ${plural(message.fields.length, "field")}`,
      `confidence ${message.confidence}`,
      `evidence ${message.evidence.map(hash => shortHash(hash)).join(", ")}`,
      message.unknowns.length > 0 ? `unknowns ${message.unknowns.join("; ")}` : undefined,
      messageSurfaceValueEvidenceDetail(message.fields),
      message.fields.length > 0
        ? `fields ${message.fields.map(field => messageSurfaceFieldLabel(field)).join(", ")}`
        : undefined,
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }
}

function messageSurfaceFieldLabel(field: MessageSurfaceField): string {
  return `${field.name}:${field.kind}@${field.source}:${field.bitOffset}`
}

function messageSurfaceValueEvidenceDetail(
  fields: readonly MessageSurfaceField[],
): string | undefined {
  const values = fields
    .filter(field => field.valueEvidence && field.valueEvidence.length > 0)
    .map(field => `${field.name} ${bodyFieldValueEvidenceDetail(field.valueEvidence ?? [])}`)
  return values.length > 0 ? `values ${values.join(" | ")}` : undefined
}

function bodyFieldValueEvidenceDetail(evidence: readonly BodyFieldValueEvidence[]): string {
  return evidence.map(item => `${item.txHash}: ${item.value}`).join("; ")
}

function schemaMethodSurfaceRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => {
    const surface = candidate.methodSurface
    if (!surface) {
      return []
    }
    const opcode = candidate.opcode ?? "<none>"
    return [
      {
        label: `${opcode} ${surface.name}`,
        value: surface.sourceFunction,
        detail: [
          surface.fields.map(field => methodSurfaceFieldLabel(field)).join(", "),
          tableValueLabel("confidence", surface.confidence),
          tableValueLabel("evidence", surface.evidence.map(hash => shortHash(hash)).join(", ")),
          tableValueLabel("unknowns", surface.unknowns.join("; ")),
        ]
          .filter((value): value is string => value !== undefined && value.length > 0)
          .join(" · "),
      },
    ]
  })
}

function methodSurfaceFieldLabel(field: MethodSurfaceField): string {
  return `${field.name}:${field.kind}@${field.source}:${field.bitOffset}`
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

function schemaStorageLayoutRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  const fields = schemaStorageLayoutFields(schema)
  return fields.map(field => ({
    label: field.name,
    value: `${field.kind} @${field.cellPath}:${field.bitOffset}`,
    detail: [
      `${formatFieldRange(field.minBits, field.maxBits)} bits`,
      `${formatFieldRange(field.minRefs, field.maxRefs)} refs`,
      `${field.observationCount} ${plural(field.observationCount, "observation")}`,
      `opcodes ${formatOpcodeList(field.opcodes)}`,
      `confidence ${field.confidence}`,
      `evidence ${field.evidence.map(hash => shortHash(hash)).join(", ")}`,
      field.valueEvidence && field.valueEvidence.length > 0
        ? `values ${storageValueEvidenceDetail(field.valueEvidence)}`
        : undefined,
      field.valueSamples.join(", "),
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }))
}

function schemaStorageLayoutFields(schema: StateFlowSchemaReport): readonly StorageLayoutField[] {
  const structured = schema.storageLayout?.fields ?? []
  if (structured.length > 0) {
    return structured
  }
  return aggregateStorageLayoutFields(schema.opcodeCandidates)
}

function storageValueEvidenceDetail(evidence: readonly StorageValueEvidence[]): string {
  return evidence
    .map(item => {
      const changeLabel = item.changed ? "changed" : "same"
      return `${item.txHash}: ${item.preValue} -> ${item.postValue} (${changeLabel})`
    })
    .join("; ")
}

function schemaEffectSurfaceRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schemaEffectSurfaceEntries(schema).map(effect => effectSurfaceRow(effect))
}

function schemaEffectSurfaceEntries(schema: StateFlowSchemaReport): readonly EffectSurfaceEntry[] {
  const structured = schema.effectSurface?.effects ?? []
  if (structured.length > 0) {
    return structured
  }
  return schema.opcodeCandidates.flatMap(candidate => [
    ...candidate.outboundEffects.map(effect =>
      effectSurfaceEntryFromCandidate(candidate, "outbound", effect),
    ),
    ...candidate.outActions.map(effect =>
      effectSurfaceEntryFromCandidate(candidate, "action", effect),
    ),
  ])
}

function effectSurfaceEntryFromCandidate(
  candidate: OpcodeSchemaCandidate,
  source: string,
  effect: EffectCandidate,
): EffectSurfaceEntry {
  const opcode = candidate.opcode
  return {
    opcode,
    opName: candidate.methodSurface?.name || `op::${formatOpcode(opcode)}`,
    source,
    kind: effect.kind,
    count: effect.count,
    modes: effect.modes ?? [],
    destinations: effect.destinations ?? [],
    valueNanotonsMin: effect.valueNanotonsMin,
    valueNanotonsMax: effect.valueNanotonsMax,
    bodyShape: effect.bodyShape,
    codeShape: effect.codeShape,
    libraryHashes: effect.libraryHashes ?? [],
    confidence: candidate.confidence,
    evidence: effect.txHashes && effect.txHashes.length > 0 ? effect.txHashes : candidate.examples,
  }
}

function effectSurfaceRow(effect: EffectSurfaceEntry): SummaryRow {
  return {
    label: `${formatOpcode(effect.opcode)} ${effect.source} ${effect.kind}`,
    value: `${effect.count} ${plural(effect.count, "effect")}`,
    detail: [
      effect.opName,
      `value ${effectSurfaceValueLabel(effect)}`,
      `body ${shapeLabel(effect.bodyShape)}`,
      `code ${shapeLabel(effect.codeShape)}`,
      listLabel("modes", effect.modes),
      listLabel("destinations", effect.destinations),
      listLabel("libraries", effect.libraryHashes),
      `confidence ${effect.confidence}`,
      `evidence ${effect.evidence.map(hash => shortHash(hash)).join(", ")}`,
    ]
      .filter((value): value is string => value !== undefined && value.length > 0)
      .join(" · "),
  }
}

function effectSurfaceValueLabel(effect: EffectSurfaceEntry): string {
  if (!effect.valueNanotonsMin || !effect.valueNanotonsMax) {
    return "n/a"
  }
  return effect.valueNanotonsMin === effect.valueNanotonsMax
    ? effect.valueNanotonsMin
    : `${effect.valueNanotonsMin}..${effect.valueNanotonsMax}`
}

function aggregateStorageLayoutFields(
  candidates: readonly OpcodeSchemaCandidate[],
): readonly StorageLayoutField[] {
  const byField = new Map<
    string,
    {
      name: string
      cellPath: string
      bitOffset: number
      minBits: number
      maxBits: number
      minRefs: number
      maxRefs: number
      kind: string
      observationCount: number
      opcodes: Set<string | undefined>
      valueSamples: Set<string>
      confidence: string
      evidence: Set<string>
    }
  >()
  for (const candidate of candidates) {
    for (const field of candidate.storage?.fields ?? []) {
      const key = `${field.name}\u0000${field.cellPath}\u0000${field.bitOffset}`
      const existing = byField.get(key)
      const entry = existing ?? {
        name: field.name,
        cellPath: field.cellPath,
        bitOffset: field.bitOffset,
        minBits: field.minBits,
        maxBits: field.maxBits,
        minRefs: field.minRefs,
        maxRefs: field.maxRefs,
        kind: field.kind,
        observationCount: 0,
        opcodes: new Set<string | undefined>(),
        valueSamples: new Set<string>(),
        confidence: field.confidence,
        evidence: new Set<string>(),
      }
      entry.minBits = Math.min(entry.minBits, field.minBits)
      entry.maxBits = Math.max(entry.maxBits, field.maxBits)
      entry.minRefs = Math.min(entry.minRefs, field.minRefs)
      entry.maxRefs = Math.max(entry.maxRefs, field.maxRefs)
      entry.kind = entry.kind === field.kind ? entry.kind : "mixed"
      entry.observationCount += field.presentCount
      entry.opcodes.add(candidate.opcode ?? undefined)
      for (const sample of field.valueSamples) entry.valueSamples.add(sample)
      entry.confidence = weakerConfidence(entry.confidence, field.confidence)
      for (const hash of candidate.examples) entry.evidence.add(hash)
      byField.set(key, entry)
    }
  }
  return [...byField.values()]
    .sort((left, right) =>
      `${left.cellPath}:${left.bitOffset}:${left.name}`.localeCompare(
        `${right.cellPath}:${right.bitOffset}:${right.name}`,
      ),
    )
    .map(field => ({
      ...field,
      opcodes: [...field.opcodes].sort((left, right) =>
        formatOpcode(left).localeCompare(formatOpcode(right)),
      ),
      valueSamples: [...field.valueSamples].sort(),
      evidence: [...field.evidence].sort(),
    }))
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

function schemaReplaySurfaceRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schemaReplaySurfaceProbes(schema).map(probe => replaySurfaceRow(probe))
}

function schemaReplaySurfaceProbes(schema: StateFlowSchemaReport): readonly ReplaySurfaceProbe[] {
  const structured = schema.replaySurface?.probes ?? []
  if (structured.length > 0) {
    return structured
  }
  return schema.opcodeCandidates.flatMap(candidate =>
    (candidate.replayProbes ?? []).map(probe => replaySurfaceProbeFromCandidate(candidate, probe)),
  )
}

function replaySurfaceProbeFromCandidate(
  candidate: OpcodeSchemaCandidate,
  probe: ReplayProbeCandidate,
): ReplaySurfaceProbe {
  const field = (candidate.inboundBody.fieldCandidates ?? []).find(
    field => field.name === probe.fieldName,
  )
  const opcode = candidate.opcode
  return {
    opcode,
    opName: candidate.methodSurface?.name || `op::${formatOpcode(opcode)}`,
    fieldName: probe.fieldName,
    fieldKind: field?.kind ?? "unknown",
    source: "body",
    bitOffset: probe.bitOffset,
    bits: probe.bits,
    value: probe.value,
    mutation: probe.mutation,
    cliArg: probe.cliArg,
    confidence: probe.confidence,
    evidence: probe.evidence,
  }
}

function replaySurfaceRow(probe: ReplaySurfaceProbe): SummaryRow {
  return {
    label: `${formatOpcode(probe.opcode)} ${probe.fieldName}`,
    value: probe.cliArg,
    detail: [
      probe.opName,
      `${probe.source} ${probe.fieldKind} @${probe.bitOffset}:${probe.bits}`,
      mutationLabel(probe.mutation),
      `confidence ${probe.confidence}`,
      `evidence ${probe.evidence.map(hash => shortHash(hash)).join(", ")}`,
    ]
      .filter(value => value.length > 0)
      .join(" · "),
  }
}

function schemaUnknownFieldRows(schema: StateFlowSchemaReport): readonly SummaryRow[] {
  return schema.opcodeCandidates.flatMap(candidate => {
    const opcode = candidate.opcode ?? "<none>"
    return candidateUnknownFieldEvidence(candidate).map(field => ({
      label: opcode,
      value: field.marker,
      detail: [
        tableValueLabel("confidence", field.confidence),
        tableValueLabel("evidence", field.evidence.map(hash => shortHash(hash)).join(", ")),
      ]
        .filter((value): value is string => value !== undefined)
        .join(" · "),
    }))
  })
}

function candidateUnknownFieldEvidence(
  candidate: OpcodeSchemaCandidate,
): readonly UnknownFieldEvidence[] {
  if (candidate.unknownFieldEvidence && candidate.unknownFieldEvidence.length > 0) {
    return candidate.unknownFieldEvidence
  }

  return candidate.unknownFields.map(marker => ({
    marker,
    confidence: candidate.confidence,
    evidence: candidate.examples,
  }))
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
      confidence: stateMachineConfidence(transition.count),
      examples: candidate.examples,
    })),
  )
}

function stateMachineNodes(
  schema: StateFlowSchemaReport,
  edges: readonly StateMachineEdge[],
): readonly StateMachineNode[] {
  const structuredNodes = schema.stateMachine?.nodes ?? []
  if (structuredNodes.length > 0) {
    return structuredNodes
  }

  const byStatus = new Map<string, {preCount: number; postCount: number; examples: Set<string>}>()
  const entryFor = (status: string) => {
    const existing = byStatus.get(status)
    if (existing) {
      return existing
    }
    const created = {preCount: 0, postCount: 0, examples: new Set<string>()}
    byStatus.set(status, created)
    return created
  }
  for (const edge of edges) {
    const from = entryFor(edge.fromStatus)
    from.preCount += edge.count
    for (const hash of edge.examples) from.examples.add(hash)
    const to = entryFor(edge.toStatus)
    to.postCount += edge.count
    for (const hash of edge.examples) to.examples.add(hash)
  }

  return [...byStatus.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([status, node]) => {
      const examples = [...node.examples].sort()
      return {
        status,
        transactionCount: examples.length,
        preCount: node.preCount,
        postCount: node.postCount,
        confidence: stateMachineConfidence(examples.length),
        examples,
      }
    })
}

function stateMachineEdgeDetail(edge: StateMachineEdge): string {
  return [
    `${edge.count} ${plural(edge.count, "observed transition")}`,
    `confidence ${edge.confidence ?? stateMachineConfidence(edge.count)}`,
    edge.examples.length > 0
      ? `examples ${edge.examples.map(hash => shortHash(hash)).join(", ")}`
      : undefined,
    edge.stateEvidence && edge.stateEvidence.length > 0
      ? `state ${stateMachineStateEvidenceDetail(edge.stateEvidence)}`
      : undefined,
  ]
    .filter((value): value is string => value !== undefined)
    .join(" · ")
}

function stateMachineStateEvidenceDetail(evidence: readonly StateMachineStateEvidence[]): string {
  return evidence.map(item => `${item.txHash}: ${item.preState} -> ${item.postState}`).join("; ")
}

function stateMachineNodeDetail(node: StateMachineNode): string {
  return [
    `pre ${node.preCount}`,
    `post ${node.postCount}`,
    `confidence ${node.confidence ?? stateMachineConfidence(node.transactionCount)}`,
    node.examples.length > 0
      ? `examples ${node.examples.map(hash => shortHash(hash)).join(", ")}`
      : undefined,
  ]
    .filter((value): value is string => value !== undefined)
    .join(" · ")
}

function stateMachineConfidence(count: number): string {
  if (count >= 3) {
    return "high"
  }
  if (count === 2) {
    return "medium"
  }
  return "low"
}

function weakerConfidence(left: string, right: string): string {
  return confidenceRank(left) <= confidenceRank(right) ? left : right
}

function confidenceRank(confidence: string): number {
  if (confidence === "high") {
    return 2
  }
  if (confidence === "medium") {
    return 1
  }
  return 0
}

function formatOpcodeList(opcodes: readonly (string | null | undefined)[]): string {
  if (opcodes.length === 0) {
    return "<none>"
  }
  return opcodes.map(opcode => formatOpcode(opcode)).join(", ")
}

function formatOpcode(opcode: string | null | undefined): string {
  return opcode ?? "<none>"
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

function replayDiffSurfaceRows(replay: StateFlowReplayDiff): readonly SummaryRow[] {
  const changes =
    replay.diffSurface && replay.diffSurface.changes.length > 0
      ? replay.diffSurface.changes
      : replayDiffSurfaceFromReplay(replay).changes

  return changes.map(change => ({
    label: change.kind,
    value: change.label,
    detail: [
      tableValueLabel("baseline", change.baseline),
      tableValueLabel("replay", change.replay),
      change.delta !== null && change.delta !== undefined
        ? tableValueLabel("delta", change.delta)
        : undefined,
      tableValueLabel("severity", change.severity),
      tableValueLabel("evidence", change.evidence.join(", ")),
    ]
      .filter((value): value is string => value !== undefined)
      .join(" · "),
  }))
}

function replayDiffSurfaceFromReplay(replay: StateFlowReplayDiff): ReplayDiffSurface {
  const changes: ReplayDiffChange[] = []
  if (!replay.diff.inputChanged) {
    return {changes}
  }

  if (!replay.diff.replayAccepted) {
    changes.push(
      replayDiffChange("accepted", "Replay accepted", "true", "false", undefined, "info", replay),
    )
    return {changes}
  }

  if (replay.diff.stateChanged === true) {
    const [baselineState, replayState] = replayStateSurfaceLabels(
      replay.baseline.state,
      replay.replay.state,
    )
    changes.push(
      replayDiffChange(
        "state",
        "Shard account state",
        baselineState,
        replayState,
        undefined,
        "high",
        replay,
      ),
    )
  }
  if (replay.diff.codeHashChanged === true) {
    changes.push(
      replayDiffChange(
        "codeHash",
        "Code hash",
        replay.baseline.state?.codeHash ?? "<none>",
        replay.replay.state?.codeHash ?? "<none>",
        undefined,
        "high",
        replay,
      ),
    )
  }
  if (replay.diff.dataHashChanged === true) {
    changes.push(
      replayDiffChange(
        "dataHash",
        "Data hash",
        replay.baseline.state?.dataHash ?? "<none>",
        replay.replay.state?.dataHash ?? "<none>",
        undefined,
        "medium",
        replay,
      ),
    )
  }
  if (typeof replay.diff.balanceDeltaDiff === "number" && replay.diff.balanceDeltaDiff !== 0) {
    changes.push(
      replayDiffChange(
        "balanceDelta",
        "Balance delta",
        replayBalanceDeltaLabel(replay.baseline),
        replayBalanceDeltaLabel(replay.replay),
        replay.diff.balanceDeltaDiff.toString(),
        "medium",
        replay,
      ),
    )
  }
  if (replay.diff.exitCodeChanged === true) {
    changes.push(
      replayDiffChange(
        "exitCode",
        "Exit code",
        replayExitCodeLabel(replay.baseline),
        replayExitCodeLabel(replay.replay),
        undefined,
        "medium",
        replay,
      ),
    )
  }
  if (typeof replay.diff.outboundCountDelta === "number" && replay.diff.outboundCountDelta !== 0) {
    changes.push(
      replayDiffChange(
        "outboundCount",
        "Outbound messages",
        replay.baseline.outbound.length.toString(),
        replay.replay.outbound.length.toString(),
        replay.diff.outboundCountDelta.toString(),
        "medium",
        replay,
      ),
    )
  }
  if (typeof replay.diff.actionCountDelta === "number" && replay.diff.actionCountDelta !== 0) {
    changes.push(
      replayDiffChange(
        "actionCount",
        "Out actions",
        replay.baseline.outActions.length.toString(),
        replay.replay.outActions.length.toString(),
        replay.diff.actionCountDelta.toString(),
        "medium",
        replay,
      ),
    )
  }
  if (replay.diff.c5Changed === true) {
    changes.push(
      replayDiffChange(
        "c5",
        "C5/action register",
        replay.baseline.c5?.hash ?? "none",
        replay.replay.c5?.hash ?? "none",
        undefined,
        "medium",
        replay,
      ),
    )
  }

  return {changes}
}

function replayDiffChange(
  kind: string,
  label: string,
  baseline: string,
  replayValue: string,
  delta: string | undefined,
  severity: string,
  replay: StateFlowReplayDiff,
): ReplayDiffChange {
  return {
    kind,
    label,
    baseline,
    replay: replayValue,
    ...(delta === undefined ? {} : {delta}),
    severity,
    evidence: [replay.sourceQueryHash],
  }
}

function replayStateSurfaceLabels(
  baseline: ShardAccountSnapshot | null | undefined,
  replay: ShardAccountSnapshot | null | undefined,
): readonly [string, string] {
  if (baseline && replay && baseline.status === replay.status) {
    return [replayStateFingerprint(baseline), replayStateFingerprint(replay)]
  }
  return [baseline?.status ?? "n/a", replay?.status ?? "n/a"]
}

function replayStateFingerprint(state: ShardAccountSnapshot): string {
  return `${state.status} balance ${state.balanceNanotons} lt ${state.lastTransLt} last ${state.lastTransHash} code ${state.codeHash ?? "<none>"} data ${state.dataHash ?? "<none>"}`
}

function replayBalanceDeltaLabel(observation: ReplayObservation): string {
  return observation.money
    ? (observation.money.balanceAfter - observation.money.balanceBefore).toString()
    : "n/a"
}

function replayExitCodeLabel(observation: ReplayObservation): string {
  return formatNullable(observation.compute?.exitCode)
}

function replayRiskSignalCount(replay: StateFlowReplayDiff): number {
  if (replay.riskSignals && replay.riskSignals.length > 0) {
    return replay.riskSignals.length
  }
  return replayRiskRows(replay).length
}

function replayRiskRows(replay: StateFlowReplayDiff): readonly SummaryRow[] {
  if (replay.riskSignals && replay.riskSignals.length > 0) {
    return replay.riskSignals.map(signal => ({
      label: signal.kind,
      value: signal.description,
      detail: [
        tableValueLabel("severity", signal.severity),
        tableValueLabel("evidence", signal.evidence.map(hash => shortHash(hash)).join(", ")),
      ]
        .filter((value): value is string => value !== undefined)
        .join(" · "),
    }))
  }

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
