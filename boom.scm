; Demo evaluation order dependent termination issue.
;
;
; Install scheme on WSL Ubuntu 20.04 with:
;   sudo apt-get install chezscheme
;
; Get microKanren sources with:
;   git clone https://github.com/jasonhemann/microKanren
;
; Run with:
;   chezscheme boom.scm

(load "microKanren.scm")
(load "miniKanren-wrappers.scm")


(define (fine)
    (conj (== 0 1) (Zzz (fine))))

(define (boom)
    (conj (Zzz (boom)) (== 0 1)))

(display (run 1 (q) (fine))) (newline)
(display (run 1 (q) (boom))) (newline)
