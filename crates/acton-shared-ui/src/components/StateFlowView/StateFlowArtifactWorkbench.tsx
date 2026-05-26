import * as React from "react"
import {FileUp, FolderOpen, Trash2} from "lucide-react"

import {Button} from "../ui/Button"

import {StateFlowArtifactView} from "./StateFlowArtifactView"
import {
  parseStateFlowArtifactBundleFromSources,
  parseStateFlowArtifactFromSource,
  STATE_FLOW_ARTIFACT_DIRECTORY_INPUT_PROPS,
  STATE_FLOW_ARTIFACT_FILE_ACCEPT,
  type StateFlowArtifact,
  type StateFlowArtifactSource,
} from "./stateFlowArtifacts"
import styles from "./StateFlowArtifactWorkbench.module.css"

export interface StateFlowArtifactWorkbenchProps {
  readonly className?: string
  readonly initialRaw?: string
  readonly storageKey?: string
}

export const StateFlowArtifactWorkbench: React.FC<StateFlowArtifactWorkbenchProps> = ({
  className,
  initialRaw,
  storageKey,
}) => {
  const fileInputRef = React.useRef<HTMLInputElement>(null)
  const directoryInputRef = React.useRef<HTMLInputElement>(null)
  const loadedInitialRawRef = React.useRef(initialRaw)
  const [raw, setRaw] = React.useState(() => initialRaw ?? readStoredArtifact(storageKey))
  const [artifact, setArtifact] = React.useState<StateFlowArtifact | undefined>(() =>
    parseInitialArtifact(initialRaw ?? readStoredArtifact(storageKey)),
  )
  const [error, setError] = React.useState<string | undefined>()

  const loadArtifact = React.useCallback(
    (nextRaw: string, sourceName?: string) => {
      try {
        const nextArtifact = parseStateFlowArtifactFromSource(nextRaw, sourceName)
        setArtifact(nextArtifact)
        setError(undefined)
        if (storageKey) {
          globalThis.localStorage?.setItem(storageKey, nextRaw)
        }
      } catch (parseError) {
        setArtifact(undefined)
        setError(parseError instanceof Error ? parseError.message : String(parseError))
      }
    },
    [storageKey],
  )

  React.useEffect(() => {
    if (initialRaw === undefined || loadedInitialRawRef.current === initialRaw) {
      return
    }
    loadedInitialRawRef.current = initialRaw
    setRaw(initialRaw)
    loadArtifact(initialRaw)
  }, [initialRaw, loadArtifact])

  const handleSubmit = React.useCallback(
    (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      if (raw.trim().length === 0) {
        setArtifact(undefined)
        setError("StateFlow JSON is empty")
        return
      }
      loadArtifact(raw)
    },
    [loadArtifact, raw],
  )

  const handleFileChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const files = [...(event.currentTarget.files ?? [])]
      if (files.length === 0) {
        return
      }

      void Promise.all(files.map(file => readArtifactSource(file))).then(sources => {
        if (sources.length === 1) {
          const [source] = sources
          setRaw(source.raw)
          loadArtifact(source.raw, source.name)
          return
        }

        const bundleArtifact = parseStateFlowArtifactBundleFromSources(sources)
        const bundleRaw = JSON.stringify(
          {
            kind: "stateFlowArtifactBundle",
            sources,
          },
          undefined,
          2,
        )
        setRaw(bundleRaw)
        setArtifact(bundleArtifact)
        setError(undefined)
        if (storageKey) {
          globalThis.localStorage?.setItem(storageKey, bundleRaw)
        }
      })
      event.currentTarget.value = ""
    },
    [loadArtifact, storageKey],
  )

  const handleClear = React.useCallback(() => {
    setRaw("")
    setArtifact(undefined)
    setError(undefined)
    if (storageKey) {
      globalThis.localStorage?.removeItem(storageKey)
    }
  }, [storageKey])

  return (
    <div className={`${styles.workbench} ${className ?? ""}`}>
      <form className={styles.controlPanel} onSubmit={handleSubmit}>
        <label className={styles.label} htmlFor="state-flow-json">
          StateFlow JSON
        </label>
        <textarea
          id="state-flow-json"
          className={styles.textarea}
          value={raw}
          spellCheck={false}
          onChange={event => setRaw(event.currentTarget.value)}
        />
        <input
          ref={fileInputRef}
          className={styles.fileInput}
          type="file"
          multiple
          accept={STATE_FLOW_ARTIFACT_FILE_ACCEPT}
          onChange={handleFileChange}
        />
        <input
          ref={directoryInputRef}
          className={styles.fileInput}
          type="file"
          {...STATE_FLOW_ARTIFACT_DIRECTORY_INPUT_PROPS}
          onChange={handleFileChange}
        />
        {error ? <div className={styles.error}>{error}</div> : undefined}
        <div className={styles.toolbar}>
          <Button type="submit">Load</Button>
          <Button type="button" variant="outline" onClick={() => fileInputRef.current?.click()}>
            <FileUp size={16} />
            File
          </Button>
          <Button
            type="button"
            variant="outline"
            onClick={() => directoryInputRef.current?.click()}
          >
            <FolderOpen size={16} />
            Directory
          </Button>
          <Button type="button" variant="ghost" onClick={handleClear}>
            <Trash2 size={16} />
            Clear
          </Button>
        </div>
      </form>

      <section className={styles.resultPanel}>
        {artifact ? (
          <StateFlowArtifactView artifact={artifact} />
        ) : (
          <div className={styles.placeholder}>No StateFlow artifact loaded.</div>
        )}
      </section>
    </div>
  )
}

function readStoredArtifact(storageKey: string | undefined): string {
  if (!storageKey) {
    return ""
  }
  return globalThis.localStorage?.getItem(storageKey) ?? ""
}

function parseInitialArtifact(raw: string): StateFlowArtifact | undefined {
  if (raw.trim().length === 0) {
    return undefined
  }
  try {
    return parseStateFlowArtifactFromSource(raw)
  } catch {
    return undefined
  }
}

async function readArtifactSource(file: File): Promise<StateFlowArtifactSource> {
  return {
    name: file.webkitRelativePath || file.name,
    raw: await file.text(),
  }
}
