;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Diagnostic only: genuine import boundaries, never a qualification receipt.
(import :gerbil/expander
        (rename-in (only-in :gerbil/tools/gxtest main) (main native-test-main)))
(export main)

(def (main . args)
  (let ((importer (current-expander-module-import))
        (sequence 0))
    (parameterize
        ((current-expander-module-import
          (lambda (path reload?)
            (set! sequence (+ sequence 1))
            (let ((id sequence) (start (real-time)))
              (displayln "IMPORT-BEGIN " id " " path)
              (force-output)
              (let (context (importer path reload?))
                (displayln "IMPORT-END " id " seconds=" (- (real-time) start) " " path)
                (force-output)
                context)))))
      (apply native-test-main args))))
