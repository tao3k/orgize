;;; -*- Gerbil -*-
;;; Definitions and escaped invocation arguments share native source ownership.

(import (only-in "objects.ss" make-org-event-helper)
        (only-in "event-source-content.ss"
                 source-content-initial source-content-forms source-space-at?))
(export macro-event-helpers)

(def index '(line-index macro-byte-index))
(def next `(line-step ,index))
(def letters (map char->integer (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")))
(def name-bytes (append letters (map char->integer (string->list "0123456789_-"))))

(def macro-event-helpers
  (list
   (make-org-event-helper
    'macro-definition source-content-initial
    (source-content-forms 'MacroDefinitionContent 'MacroTrivia
      (lambda (from until) `((call-source-helper macro-definition-name ,from ,until))) #f))
   (make-org-event-helper
    'macro-definition-name '((macro-name-end 0) (macro-name-valid #t) (macro-name-done #f))
    `((set-uint macro-name-end (offset start))
      (for-line-bytes macro-byte-index start end
        ((if (state macro-name-done) ()
             ((if ,(source-space-at? index 'end)
                  ((set-bool macro-name-done (bool #t)))
                  ((if (line-bytes-all-in? ,index ,next ,name-bytes) ()
                       ((set-bool macro-name-valid (bool #f))))
                   (set-uint macro-name-end (offset ,next))))))))
      (if (and (offset-less? start (state-offset macro-name-end))
               (state macro-name-valid)
               (line-bytes-any-in? start (line-step start) ,letters))
          ((token MacroDefinitionName start (state-offset macro-name-end)))
          ((token MacroDefinitionInvalid start (state-offset macro-name-end))))
      (call-source-helper macro-definition-template (state-offset macro-name-end) end)))
   (make-org-event-helper
    'macro-definition-template source-content-initial
    (source-content-forms 'MacroDefinitionTemplate 'MacroTrivia
      (lambda (from until) `((token MacroTemplateText ,from ,until))) #f))
   (make-org-event-helper
    'macro-arguments '((macro-segment 0) (macro-escaped #f))
    `((set-uint macro-segment (offset start))
      (for-line-bytes macro-byte-index start end
        ((if (state macro-escaped)
             ((set-bool macro-escaped (bool #f)))
             ((if (line-byte-equal? ,index 92)
                  ((set-bool macro-escaped (bool #t)))
                  ((if (line-byte-equal? ,index 44)
                       ((call-source-helper macro-argument-content (state-offset macro-segment) ,index)
                        (token MacroTrivia ,index ,next)
                        (set-uint macro-segment (offset ,next))) ())))))))
      (call-source-helper macro-argument-content (state-offset macro-segment) end)))
   (make-org-event-helper
    'macro-argument-content source-content-initial
    (source-content-forms 'MacroArgumentContent 'MacroTrivia
      (lambda (from until)
        `((if (offset-less? ,from ,until)
              ((call-source-helper macro-argument-value ,from ,until)) ())))))
   (make-org-event-helper
    'macro-argument-value '((macro-escaped #f) (macro-text-start 0))
    `((start-node OrgMacroArgument)
      (set-uint macro-text-start (offset start))
      (for-line-bytes macro-byte-index start end
        ((if (state macro-escaped)
             ((set-bool macro-escaped (bool #f)))
             ((if (line-byte-equal? ,index 92)
                  ((if (and (offset-less? ,next end)
                            (line-bytes-any-in? ,next (line-step ,next) (44 92)))
                       ((token MacroArgumentText (state-offset macro-text-start) ,index)
                        (token MacroTrivia ,index ,next)
                        (set-uint macro-text-start (offset ,next))) ())
                   (set-bool macro-escaped (bool #t))) ())))))
      (token MacroArgumentText (state-offset macro-text-start) end)
      (finish-node)))))
