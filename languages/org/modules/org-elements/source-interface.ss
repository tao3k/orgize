;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

;;; Source-backed projection is separate from the parser-independent graph
;;; query runtime to keep existing callers free of parser initialization.
(import "source-headlines.ss")
(export (import: "source-headlines.ss"))
