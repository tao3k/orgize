;;; One parameter admission shared by the native event and tape consumers.
(export native-parameter-overrides)
(def (native-parameter-overrides level policy)
  (unless (and (exact-integer? level) (> level 0) (memv policy '(0 1 2)))
    (error "invalid Org parser parameters" level policy))
  (list (cons 'inlinetask-min-level level)
        (cons 'inline-script-policy policy)))
