;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :clan/poo/object .o .ref)
        (only-in "../languages/org/modules/org-contract/interface.ss"
                 OrgDirectoryProfile. org-directory-profile?
                 org-directory-validate org-directory-validation-current?
                 org-directory-contracts make-org-contract-definition)
        (only-in "org-directory-test-support.ss"
                 directory-mutation-fixture directory-test-write directory-test-read))
(export org-directory-test)

(def documents "tests/fixtures/org-directory/documents")
(def common "tests/fixtures/org-directory/common")
(def profile
  (.o (:: @ OrgDirectoryProfile.)
      directory: documents
      contracts: (.o directory: common)))
(def org-directory-test
  (test-suite "Native Org Directory Contract composition"
    (test-case "one native declaration adopts the Directory and common DI"
      (check (org-directory-profile? profile) => #t)
      (let (validation (org-directory-validate profile))
        (check (.ref validation 'complete?) => #t)
        (check (.ref validation 'passed?) => #t)
        (check (.ref validation 'action-authority?) => #f)
        (check (length (.ref validation 'files)) => 1)
        (check (length (.ref validation 'evaluations)) => 2)
        (check (org-directory-validation-current? validation profile) => #t)))
    (test-case "foreign neighboring Evidence cannot satisfy the family"
      (let (p (.o (:: @ profile) directory: "tests/fixtures/org-directory/missing"))
        (check (.ref (org-directory-validate p) 'passed?) => #f)))
    (test-case "common applicability uses declared role without CONTRACT_REF"
      (let* ((p (.o (:: @ profile) directory: "tests/fixtures/org-directory/automatic"))
             (validation (org-directory-validate p)))
        (check (.ref validation 'passed?) => #t)
        (check (length (.ref validation 'evaluations)) => 2)))
    (test-case "embedded local Contract applies to its own scope automatically"
      (let (p (.o (:: @ profile) directory: "tests/fixtures/org-directory/local"))
        (check (.ref (org-directory-validate p) 'passed?) => #f)))
    (test-case "ambiguous stable IDs cannot select the first occurrence"
      (check-exception
       (org-directory-validate (.o (:: @ profile) directory: "tests/fixtures/org-directory/duplicate"))
       exception?))
    (test-case "a missing common DI never silently becomes optional"
      (check-exception
       (org-directory-validate
        (.o (:: @ profile) contracts: (.o directory: "tests/fixtures/org-directory/absent" definitions: '())))
       exception?))
    (test-case "a declared common DI without Contracts is rejected"
      (check-exception
       (org-directory-validate
        (.o (:: @ profile) contracts: (.o directory: documents definitions: '())))
       exception?))
    (test-case "unknown reference is an unresolved obligation"
      (check-exception
       (org-directory-validate (.o (:: @ profile) directory: "tests/fixtures/org-directory/unknown"))
       exception?))
    (test-case "source blocks cannot execute arbitrary Scheme"
      (check-exception
       (org-directory-validate
        (.o (:: @ profile) contracts: (.o directory: "tests/fixtures/org-directory/forged" definitions: '())))
       exception?))
    (test-case "common and local cannot overwrite each other's identity"
      (let (contract (make-org-contract-definition "collision" 'document '()))
        (check-exception
         (org-directory-contracts (.o (:: @ profile) local: (list contract)) (list contract))
         exception?)))
    (test-case "mandatory baseline cannot be shadowed through the local layer"
      (check-exception
       (org-directory-contracts
        (.o (:: @ profile)
            local: (list (make-org-contract-definition "orgize.directory.document-root.v1" 'document '())))
        '()) exception?))
    (test-case "validation cannot be transferred to another Profile"
      (let ((validation (org-directory-validate profile))
            (other (.o (:: @ profile) directory: "tests/fixtures/org-directory/missing")))
        (check (org-directory-validation-current? validation other) => #f)))
    (test-case "source revision reopens validation and revalidation closes it"
      (let* ((p (directory-mutation-fixture "source-update"))
             (validation (org-directory-validate p))
             (path (path-expand "main.org" (.ref p 'directory))))
        (directory-test-write path (directory-test-read "tests/fixtures/org-directory/missing/doc.org"))
        (check (org-directory-validation-current? validation p) => #f)
        (check (.ref (org-directory-validate p) 'passed?) => #f)
        (directory-test-write path (directory-test-read "tests/fixtures/org-directory/documents/main.org"))
        (let (refreshed (org-directory-validate p))
          (check (.ref refreshed 'passed?) => #t)
          (check (org-directory-validation-current? refreshed p) => #t))))
    (test-case "common Contract revision invalidates retained evidence"
      (let* ((p (directory-mutation-fixture "common-update"))
             (validation (org-directory-validate p))
             (path (path-expand "evidence.org" (.ref (.ref p 'contracts) 'directory))))
        (directory-test-write path (directory-test-read "tests/fixtures/org-directory/strict-evidence.org"))
        (check (org-directory-validation-current? validation p) => #f)
        (check (.ref (org-directory-validate p) 'passed?) => #f)))
    (test-case "a newly discovered Org file reopens inventory coverage"
      (let* ((p (directory-mutation-fixture "inventory-update"))
             (validation (org-directory-validate p)))
        (directory-test-write (path-expand "new.org" (.ref p 'directory))
          (directory-test-read "tests/fixtures/org-directory/unknown/doc.org"))
        (check (org-directory-validation-current? validation p) => #f)
        (check-exception (org-directory-validate p) exception?)))
    (test-case "a local source Contract update reopens its previous result"
      (let* ((p (directory-mutation-fixture "local-update"))
             (path (path-expand "main.org" (.ref p 'directory)))
             (validation (org-directory-validate p)))
        (directory-test-write path (directory-test-read "tests/fixtures/org-directory/local/doc.org"))
        (check (org-directory-validation-current? validation p) => #f)
        (check (.ref (org-directory-validate p) 'passed?) => #f)))))
