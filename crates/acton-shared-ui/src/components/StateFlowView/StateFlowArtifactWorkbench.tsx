import * as React from "react"
import {FileUp, Trash2} from "lucide-react"

import {Button} from "../ui/Button"

import {StateFlowArtifactView} from "./StateFlowArtifactView"
import {parseStateFlowArtifact, type StateFlowArtifact} from "./stateFlowArtifacts"
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
  const [raw, setRaw] = React.useState(() => initialRaw ?? readStoredArtifact(storageKey))
  const [artifact, setArtifact] = React.useState<StateFlowArtifact | undefined>(() =>
    parseInitialArtifact(initialRaw ?? readStoredArtifact(storageKey)),
  )
  const [error, setError] = React.useState<string | undefined>()

  const loadArtifact = React.useCallback(
    (nextRaw: string) => {
      try {
        const nextArtifact = parseStateFlowArtifact(nextRaw)
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
      const file = event.currentTarget.files?.[0]
      if (!file) {
        return
      }

      void file.text().then(text => {
        setRaw(text)
        loadArtifact(text)
      })
      event.currentTarget.value = ""
    },
    [loadArtifact],
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
          accept="application/json,.json"
          onChange={handleFileChange}
        />
        {error ? <div className={styles.error}>{error}</div> : undefined}
        <div className={styles.toolbar}>
          <Button type="submit">Load</Button>
          <Button type="button" variant="outline" onClick={() => fileInputRef.current?.click()}>
            <FileUp size={16} />
            File
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
    return parseStateFlowArtifact(raw)
  } catch {
    return undefined
  }
}
