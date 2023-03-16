# µKanren experiments
This repo contains my experimentation with [µKanren](http://webyrd.net/scheme-2013/papers/HemannMuKanren2013.pdf). Thanks [Nate](https://github.com/ncatelli), for the introduction.


## kanren.py
Based on a [papers we love video](https://youtu.be/Dm7_DiNxFNk), where Nicola Mometto walks through a µKanren implementation in Python.

Made some small modifications to the code, and added type annotations. Also fixed a bug that was in the original implementation. The `if subst_map:` in `eq()` will not be taken when `subst_map` is an empty dictionary, and this is incorrect. Changed it to `if subst_map is not None:`, so only a `None` will not take the branch.

At the end of the file some [test cases from an other implementation](https://github.com/pythological/kanren/blob/main/tests/test_facts.py) are reproduced.

## boom.scm
While playing with recursive goals in the python implementation I found two goals that are equivalent, but one would terminate and the other one would blow the stack. The termination was depended on evaluation order.

I found this surprising because the paper presents a couple of techniques that should result in a breath-first search that can deal with infinite streams. At first I thought that there was a bug in my Python implementation. But `boom.scm` show that the original scheme implementation from the paper also has this issue. So this seems to be a limitation of the algorithm.

I'm not the first to notice this limitation. The blog post [Search trees and core.logic](https://www.scattered-thoughts.net/writing/search-trees-and-core-dot-logic/) discuses a similar issue in the closure core.logic library, and an alternative way to approach the problem is proposed.

## kanren-rs
A µKanren implementation in Rust.
