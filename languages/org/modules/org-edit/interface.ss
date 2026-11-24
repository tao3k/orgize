;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Yuanxu module declaration for parser-verified Org structural editing.
(import (only-in :clan/poo/object .def .o)
        (only-in "./types.ss" OrgEditScopeContract)
        (only-in "./objects.ss" OrgEdit.))
(export OrgEdit)

(.def OrgEdit
  identity: 'org-edit
  core: (.o scope-contract: OrgEditScopeContract
            exports: OrgEdit.)
  features: (.o))
