;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Org Edit POO module scope object.

(import (only-in :clan/poo/object .def .get)
        "types.ss"
        (only-in "ops.ss" org-apply-edit org-apply-edits))
(export OrgEdit.)

(.def (OrgEdit. @ (.get OrgEditScopeContract proto))
  apply-edit: org-apply-edit
  apply-edits: org-apply-edits)
