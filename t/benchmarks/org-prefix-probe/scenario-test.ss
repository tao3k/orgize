(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/source-line-support line-starts-with?))
(export prefix-probe-test)

(def prefix-probe-test
  (test-suite "Org prefix probe benchmark"
    (test-case "The prefix matches only at the start of a complete prefix"
      (check (line-starts-with? "#+BEGIN_QUOTE\n" "#+") => #t)
      (check (line-starts-with? " #+BEGIN_QUOTE\n" "#+") => #f)
      (check (line-starts-with? "#" "#+") => #f))))
