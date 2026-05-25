import * as React from "react"
import {StateFlowArtifactWorkbench} from "@acton/shared-ui"

import styles from "../DashboardPage.module.css"

export const StateFlowPage: React.FC = () => {
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
        <StateFlowArtifactWorkbench storageKey="acton-localnet-state-flow-artifact" />
      </section>
    </>
  )
}
