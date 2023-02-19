# µKanren experiments
This repo contains my experimentation with [µKanren](http://webyrd.net/scheme-2013/papers/HemannMuKanren2013.pdf). Thanks [Nate](https://github.com/ncatelli), for the introduction.


## kanren.py
Based on a [papers we love video](https://youtu.be/Dm7_DiNxFNk), where Nicola Mometto walks through a µKanren implementation in Python.

Made some small modifications to the code, and added type annotations. Also fixed a bug that was in the original implementation. The `if subst_map:` in `eq()` will not be taken when `subst_map` is an empty dictionary, and this is incorrect. Changed it to `if subst_map is not None:`, so only a `None` will not take the branch.

At the end of the file some [test cases from an other implementation](https://github.com/pythological/kanren/blob/main/tests/test_facts.py) are reproduced.
