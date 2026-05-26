import * as React from "react"
import {StateFlowArtifactWorkbench} from "@acton/shared-ui"

import type {TonClient} from "../../explorer/api/client"
import styles from "../DashboardPage.module.css"

interface StateFlowPageProps {
  readonly client: TonClient
}

export const StateFlowPage: React.FC<StateFlowPageProps> = ({client}) => {
  const [initialRaw, setInitialRaw] = React.useState<string | undefined>()

  React.useEffect(() => {
    let cancelled = false

    void client
      .getStateFlowArtifacts()
      .then(bundle => {
        if (cancelled) {
          return
        }
        setInitialRaw(bundle.sources.length > 0 ? JSON.stringify(bundle, undefined, 2) : undefined)
      })
      .catch(error => {
        if (!cancelled) {
          console.error("Failed to fetch state-flow artifacts", error)
          setInitialRaw(undefined)
        }
      })

    return () => {
      cancelled = true
    }
  }, [client])

  return (
    <>
      <section className={styles.hero}>
        <div>
          <h1 className={styles.title}>State Flow</h1>
          <p className={styles.subtitle}>
            Reverse artifacts from retrace, collect, infer, and replay.
          </p>
        </div>
      </section>

      <section className={styles.stateFlowLayout}>
        <StateFlowArtifactWorkbench
          initialRaw={initialRaw}
          storageKey="acton-localnet-state-flow-artifact"
        />
      </section>
    </>
  )
}
