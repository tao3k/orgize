;;; -*- Gerbil -*-
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; The official make/compiler executor owns object scheduling and link barriers.
(import (only-in :std/make make))
(export main)

(def (main source binary cc-options)
  (make `((exe: ,source bin: ,binary "-cc-options" ,cc-options))
        srcdir: (current-directory)
        parallelize: (##core-count)
        verbose: 9))
