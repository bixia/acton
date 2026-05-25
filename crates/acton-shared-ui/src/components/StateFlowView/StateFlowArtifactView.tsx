import * as React from "react"

import {type StateFlowArtifact, summarizeStateFlowArtifact} from "./stateFlowArtifacts"
import styles from "./StateFlowArtifactView.module.css"

export interface StateFlowArtifactViewProps {
  readonly artifact: StateFlowArtifact
  readonly className?: string
}

export const StateFlowArtifactView: React.FC<StateFlowArtifactViewProps> = ({
  artifact,
  className,
}) => {
  const summary = React.useMemo(() => summarizeStateFlowArtifact(artifact), [artifact])

  return (
    <section className={`${styles.view} ${className ?? ""}`}>
      <header className={styles.header}>
        <h2 className={styles.title}>{summary.title}</h2>
        {summary.subtitle ? <p className={styles.subtitle}>{summary.subtitle}</p> : undefined}
      </header>

      <div className={styles.metricGrid}>
        {summary.metrics.map(metric => (
          <div key={metric.label} className={styles.metric}>
            <span className={styles.metricLabel}>{metric.label}</span>
            <span className={styles.metricValue} title={metric.value}>
              {metric.value}
            </span>
          </div>
        ))}
      </div>

      <div className={styles.sections}>
        {summary.sections.map(section => (
          <section key={section.title} className={styles.section}>
            <h3 className={styles.sectionTitle}>{section.title}</h3>
            {section.rows.length === 0 ? (
              <div className={styles.empty}>No rows.</div>
            ) : (
              <div className={styles.rowTable}>
                {section.rows.map(row => (
                  <div key={`${row.label}:${row.value}:${row.detail ?? ""}`} className={styles.row}>
                    <span className={styles.rowLabel}>{row.label}</span>
                    <span className={styles.rowValue}>
                      {row.value}
                      {row.detail ? (
                        <span className={styles.rowDetail}>{row.detail}</span>
                      ) : undefined}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </section>
        ))}
      </div>
    </section>
  )
}
