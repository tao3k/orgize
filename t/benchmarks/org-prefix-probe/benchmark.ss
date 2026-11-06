;; ASP's normal micro-kernel batch sizes keep both baseline legs measurable.
;; Diagnostic only: this envelope is not a document-parse SLO.
((benchmarkKind . micro-kernel)
 (name . org-prefix-probe)
 (warmupOperations . 20000)
 (batchOperations . 100000)
 (sampleCount . 20)
 (minimumBatchDuration . 1ms)
 (targetNsPerOp . 10000)
 (regressionBudgetNsPerOp . 10000)
 (maxNsPerOp . 20000)
 (targetRationale . "Isolated prefix allocation probe, not a parser performance gate"))
