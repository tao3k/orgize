#!/usr/bin/env gxi
(import (only-in :clan/poo/object .get)
        (only-in :std/source this-source-file)
        (only-in :std/build-script defbuild-script)
        (only-in :tao3k/yuanxu/module-system/building/build-script defmodule-build-script)
        (only-in "./package-spec.ss" orgize.package.exports))

(defmodule-build-script
  (defbuild-script (.get orgize.package.exports std-make-spec))
  (.get orgize.package.exports build-actions))
