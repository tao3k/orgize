;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Yuanxu module declaration for parser-verified Org structural editing.

(.def OrgEdit
  identity: 'org-edit
  core: (.o requires: (.o) scope-contract: OrgEditScopeContract
            exports: OrgEdit.)
  features: (.o))
