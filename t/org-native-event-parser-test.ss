;;; -*- Gerbil -*-
;;; Aggregate the independently owned Org event-parser test modules.
(import (only-in "org-native-inline-parser-test.ss"
                 org-native-inline-parser-test)
        (only-in "org-native-structural-parser-test.ss"
                 org-native-structural-parser-test)
        (only-in "org-native-table-container-parser-test.ss"
                 org-native-table-container-parser-test))
;; gxtest discovers each exported *-test suite. Evaluating suite values inside
;; another test-suite only constructs an empty suite; it does not run them.
(export org-native-inline-parser-test
        org-native-structural-parser-test
        org-native-table-container-parser-test)
