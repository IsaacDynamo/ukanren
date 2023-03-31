#[cfg(test)]
mod tests {
    use crate::display::*;
    use crate::*;

    #[test]
    fn test_unify() {
        use Term::*;
        let e = Mapping::default();

        assert_eq!(format!("{:?}", unify(&Value(1), &Value(1), &e)), "Some({})");
        assert_eq!(format!("{:?}", unify(&Value(1), &Value(2), &e)), "None");
        assert_eq!(format!("{:?}", unify(&Var(1), &Var(1), &e)), "Some({})");
        assert_eq!(
            format!("{:?}", unify(&Var(1), &Var(2), &e)),
            "Some({1: Var(2)})"
        );
        assert_eq!(format!("{:?}", unify(&Null, &Null, &e)), "Some({})");
        assert_eq!(
            format!("{:?}", unify(&cons(1, 2), &cons(1, 2), &e)),
            "Some({})"
        );
        assert_eq!(format!("{:?}", unify(&cons(1, 2), &cons(2, 4), &e)), "None");
        assert_eq!(
            format!("{:?}", unify(&cons(1, NULL), &cons(1, NULL), &e)),
            "Some({})"
        );
        assert_eq!(
            format!("{:?}", unify(&cons(1, NULL), &cons(1, cons(2, NULL)), &e)),
            "None"
        );
        assert_eq!(
            format!("{:?}", unify(&cons(1, Var(1)), &cons(1, Var(2)), &e)),
            "Some({1: Var(2)})"
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(AsScheme(run_all(eq(1, 1))).to_string(), "(())");
        assert_eq!(AsScheme(run_all(eq(1, 2))).to_string(), "()");

        assert_eq!(
            AsScheme(run_all(|x| either(eq(x, 1), eq(x, 1)))).to_string(),
            "((1) (1))"
        );
        assert_eq!(
            AsScheme(run_all(|x| either(eq(x, 1), eq(x, 2)))).to_string(),
            "((1) (2))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| either(eq(x, 1), eq(y, 2)))).to_string(),
            "((1 _1) (_0 2))"
        );

        assert_eq!(
            AsScheme(run_all(|x| both(eq(x, 1), eq(x, 1)))).to_string(),
            "((1))"
        );
        assert_eq!(
            AsScheme(run_all(|x| both(eq(x, 1), eq(x, 2)))).to_string(),
            "()"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| both(eq(x, 1), eq(y, 2)))).to_string(),
            "((1 2))"
        );

        assert_eq!(
            AsScheme(run_all(fresh(|x, y| both(eq(x, 1), eq(y, 2))))).to_string(),
            "(())"
        );
        assert_eq!(
            AsScheme(run_all(|x| fresh(move |y| both(eq(x, 1), eq(y, 2))))).to_string(),
            "((1))"
        );
    }

    #[test]
    fn test_and_gate() {
        fn and(a: Var, b: Var, o: Var) -> Goal {
            cond([
                [eq(a, 0), eq(b, 0), eq(o, 0)],
                [eq(a, 0), eq(b, 1), eq(o, 0)],
                [eq(a, 1), eq(b, 0), eq(o, 0)],
                [eq(a, 1), eq(b, 1), eq(o, 1)],
            ])
        }

        assert_eq!(
            AsScheme(run_all(|x, y| and(x, y, x))).to_string(),
            "((0 0) (0 1) (1 1))"
        );
    }

    #[test]
    fn test_yield() {
        fn fives(x: Var) -> Goal {
            either(eq(x, 5), jield(move || fives(x)))
        }

        fn sixes(x: Var) -> Goal {
            either(eq(x, 6), jield(move || sixes(x)))
        }

        assert_eq!(
            AsScheme(run(5, |x| fives(x))).to_string(),
            "((5) (5) (5) (5) (5))"
        );

        assert_eq!(
            AsScheme(run(5, |x| either(fives(x), sixes(x)))).to_string(),
            "((5) (6) (5) (6) (5))"
        );

        assert_eq!(
            AsScheme(run(12, |x, y| both(
                either(eq(x, 6), fives(x)),
                either(eq(y, 5), sixes(y))
            )))
            .to_string(),
            "((6 5) (6 6) (5 5) (5 6) (6 6) (5 6) (5 5) (5 6) (6 6) (5 6) (5 6) (5 5))"
        );
    }

    #[test]
    #[ignore = "Boom issue, doesn't terminate"]
    fn test_boom() {
        fn fives(x: Var) -> Goal {
            either(eq(x, 5), jield(move || fives(x)))
        }

        assert_eq!(
            AsScheme(run_all(|x| both(eq(0, 1), fives(x)))).to_string(),
            "()"
        );

        // Boom!
        assert_eq!(
            AsScheme(run_all(|x| both(fives(x), eq(0, 1)))).to_string(),
            "()"
        );
    }

    #[test]
    fn test_concat() {
        fn concat(l: Var, r: Var, out: Var) -> Goal {
            fresh(move |a, d, res| {
                cond([
                    vec![eq(NULL, l), eq(r, out)],
                    vec![
                        eq(cons(a, d), l),
                        eq(cons(a, res), out),
                        jield(move || concat(d, r, res)),
                    ],
                ])
            })
        }

        fn l() -> Term {
            cons(1, cons(2, cons(3, cons(4, NULL))))
        }

        assert_eq!(
            AsScheme(run_all(move |x, y| fresh(move |r| both(
                eq(r, l()),
                concat(x, y, r)
            ))))
            .to_string(),
            "((() (1 2 3 4)) ((1) (2 3 4)) ((1 2) (3 4)) ((1 2 3) (4)) ((1 2 3 4) ()))"
        );
    }

    fn contains(set: Var, x: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(set, cons(head, tail)), eq(head, x)],
                vec![
                    eq(set, cons(head, tail)),
                    neq(head, x),
                    jield(move || contains(tail, x)),
                ],
            ])
        })
    }

    fn excludes(set: Var, x: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(set, NULL)],
                vec![
                    eq(set, cons(head, tail)),
                    neq(head, x),
                    jield(move || excludes(tail, x)),
                ],
            ])
        })
    }

    fn set_eq(a: Var, b: Var) -> Goal {
        both(subset(a, b), subset(b, a))
    }

    // Set a is a subset of b
    fn subset(a: Var, b: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(a, NULL)],
                vec![
                    eq(a, cons(head, tail)),
                    contains(b, head),
                    jield(move || subset(tail, b)),
                ],
            ])
        })
    }

    fn superset(a: Var, b: Var) -> Goal {
        subset(b, a)
    }

    fn set_insert(set: Var, x: Var, result: Var) -> Goal {
        fresh(move |head, tail, c| {
            cond([
                vec![eq(set, NULL), eq(result, cons(x, NULL))],
                vec![eq(set, cons(head, tail)), eq(head, x), eq(result, set)],
                vec![
                    eq(set, cons(head, tail)),
                    neq(head, x),
                    eq(result, cons(head, c)),
                    jield(move || set_insert(tail, x, c)),
                ],
            ])
        })
    }

    fn set_remove(set: Var, x: Var, result: Var) -> Goal {
        fresh(move |head, tail, c| {
            cond([
                vec![eq(set, NULL), eq(result, NULL)],
                vec![
                    eq(set, cons(head, tail)),
                    eq(head, x),
                    jield(move || set_remove(tail, x, result)),
                ],
                vec![
                    eq(set, cons(head, tail)),
                    neq(head, x),
                    jield(move || set_remove(tail, x, c)),
                    set_insert(c, head, result),
                ],
            ])
        })
    }

    fn set_join(a: Var, b: Var, result: Var) -> Goal {
        fresh(move |head, tail, c| {
            cond([
                vec![eq(b, NULL), eq(result, a)],
                vec![
                    eq(b, cons(head, tail)),
                    set_insert(a, head, c),
                    jield(move || set_join(c, tail, result)),
                ],
            ])
        })
    }

    fn set_minimal(set: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(set, NULL)],
                vec![
                    eq(set, cons(head, tail)),
                    excludes(tail, head),
                    jield(move || set_minimal(tail)),
                ],
            ])
        })
    }

    #[test]
    fn test_set() {
        assert_eq!(
            AsScheme(run_all(|_| fresh(move |s, x| all([
                eq(s, cons(1, cons(2, cons(3, NULL)))),
                eq(x, 2),
                excludes(s, x),
            ]))))
            .to_string(),
            "()"
        );

        assert_eq!(
            AsScheme(run_all(|_| fresh(move |s, x| all([
                eq(s, cons(1, cons(2, cons(3, NULL)))),
                eq(x, 4),
                excludes(s, x),
            ]))))
            .to_string(),
            "((_0))"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |s, x| all([
                eq(s, cons(1, cons(2, cons(3, NULL)))),
                eq(x, 3),
                set_insert(s, x, q),
            ]))))
            .to_string(),
            "(((1 2 3)))"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |s, x| all([
                eq(s, cons(1, cons(2, cons(3, NULL)))),
                eq(x, 4),
                set_insert(s, x, q),
            ]))))
            .to_string(),
            "(((1 2 3 4)))"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |a, b| all([
                eq(a, cons(1, cons(2, NULL))),
                eq(b, cons(2, cons(3, NULL))),
                set_join(a, b, q),
            ]))))
            .to_string(),
            "(((1 2 3)))"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |a, b| all([
                eq(a, cons(1, cons(2, NULL))),
                eq(b, NULL),
                set_join(a, b, q),
            ]))))
            .to_string(),
            "(((1 2)))"
        );

        assert_eq!(
            AsScheme(run_all(fresh(move |x| all([
                eq(x, cons(1, cons(2, cons(3, NULL)))),
                set_minimal(x),
            ]))))
            .to_string(),
            "(())"
        );

        assert_eq!(
            AsScheme(run_all(fresh(move |x| all([
                eq(x, cons(1, cons(2, cons(1, NULL)))),
                set_minimal(x),
            ]))))
            .to_string(),
            "()"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x, y| all([
                eq(x, cons(1, cons(2, cons(1, NULL)))),
                eq(y, 1),
                set_remove(x, y, q),
            ]))))
            .to_string(),
            "(((2)))"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x, y| all([
                eq(x, cons(1, cons(2, cons(1, NULL)))),
                eq(y, 2),
                set_remove(x, y, q),
            ]))))
            .to_string(),
            "(((1)))"
        );

        assert_eq!(
            AsScheme(run_all(|x, y| fresh(move |set| all([
                eq(set, cons(1, cons(2, NULL))),
                contains(set, x),
                contains(set, y),
            ]))))
            .to_string(),
            "((1 1) (1 2) (2 1) (2 2))"
        );

        // there are inf sets that contain 1 and 2, so only show that there exists at least one
        assert_eq!(
            AsScheme(run(
                1,
                fresh(move |x, y, set| all([
                    eq(x, 1),
                    eq(y, 2),
                    contains(set, x),
                    contains(set, y),
                ]))
            ))
            .to_string(),
            "(())"
        );

        assert_eq!(
            AsScheme(run(2, |q| fresh(move |set| all([
                eq(set, cons(1, cons(2, NULL))),
                set_eq(q, set),
                set_minimal(q)
            ]))))
            .to_string(),
            "(((1 2)) ((2 1)))"
        );

        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x| all([
                eq(x, cons(1, cons(2, cons(3, NULL)))),
                excludes(x, q)
            ]))))
            .to_string(),
            "((_0) : (((_0 . 1)) ((_0 . 2)) ((_0 . 3))))"
        );
    }

    #[test]
    #[ignore = "Doesn't terminate, same issue as boom maybe?"]
    fn test_set_todo() {
        let mut q = query(|q| {
            fresh(move |x| all([eq(x, cons(1, cons(2, NULL))), set_eq(q, x), set_minimal(q)]))
        });

        let first = q
            .iter()
            .take(2)
            .map(|s| StateN::<1> { state: s })
            .collect::<Vec<_>>();
        assert_eq!(AsScheme(first).to_string(), "(((1 2)) ((2 1)))");

        // for _ in 0..10_000 {
        //     q.pull();
        // }
        // println!("{}", GoalTree(&q.goal));

        // Doesn't terminate
        let remainder = q
            .iter()
            .take(2)
            .map(|s| StateN::<1> { state: s })
            .collect::<Vec<_>>();
        assert_eq!(AsScheme(remainder).to_string(), "()");
    }

    #[test]
    fn neq_test() {
        assert_eq!(AsScheme(run_all(|_| neq(5, 6))).to_string(), "((_0))");
        assert_eq!(AsScheme(run_all(|_| neq(5, 5))).to_string(), "()");
        assert_eq!(
            AsScheme(run_all(|q| neq(q, 6))).to_string(),
            "((_0) : (((_0 . 6))))"
        );
        assert_eq!(
            AsScheme(run_all(|q| all([neq(q, 6), eq(q, 6)]))).to_string(),
            "()"
        );
        assert_eq!(
            AsScheme(run_all(|q| all([eq(q, 6), neq(q, 6)]))).to_string(),
            "()"
        );
        assert_eq!(
            AsScheme(run_all(|q| neq(q, 5))).to_string(),
            "((_0) : (((_0 . 5))))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(x, y))).to_string(),
            "((_0 _1) : (((_0 . _1))))"
        );
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x| all([
                neq(5, q),
                eq(x, q),
                neq(6, x)
            ]))))
            .to_string(),
            "((_0) : (((_0 . 5)) ((_0 . 6))))"
        );

        assert_eq!(
            AsScheme(run_all(|x, y| eq(cons(5, x), cons(6, y)))).to_string(),
            "()"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(cons(5, x), cons(6, y)))).to_string(),
            "((_0 _1))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| eq(cons(5, x), cons(5, y)))).to_string(),
            "((_1 _1))" // or "((_0 _0))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(cons(5, x), cons(5, y)))).to_string(),
            "((_0 _1) : (((_0 . _1))))"
        );

        assert_eq!(
            AsScheme(run_all(|x| neq(cons(x, x), cons(5, 6)))).to_string(),
            "((_0))"
        );
        assert_eq!(
            AsScheme(run_all(|x| neq(cons(x, x), cons(5, 5)))).to_string(),
            "((_0) : (((_0 . 5))))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(cons(x, y), cons(5, 6)))).to_string(),
            "((_0 _1) : (((_0 . 5) (_1 . 6))))"
        );
        assert_eq!(
            AsScheme(run_all(|q| all([neq(q, 2), eq(q, 2)]))).to_string(),
            "()"
        );

        assert_eq!(
            AsScheme(run_all(|x, y| all([neq(x, y), neq(x, 6)]))).to_string(),
            "((_0 _1) : (((_0 . _1)) ((_0 . 6))))" // or  "((_0 _1) : (((_0 . 6))) ((_0 . _1)))"
        );
        assert_eq!(
            AsScheme(run_all(|p, x, y| all([
                neq(cons(5, 6), p),
                eq(cons(x, y), p),
                eq(5, x),
                eq(7, y)
            ])))
            .to_string(),
            "(((5 . 7) 5 7))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(x, cons(5, y)))).to_string(),
            "((_0 _1) : (((_0 . (5 . _1)))))"
        );
    }

    #[test]
    fn constraint_test_todo() {
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x, y| neq(q, cons(x, y))))).to_string(),
            "((_0))"
        );
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x, y, z| all([
                neq(cons(y, z), x),
                eq(q, cons(x, cons(y, cons(z, NULL))))
            ]))))
            .to_string(),
            "(((_1 _2 _3)) : (_1 _2 . _3) )"
        );
    }
}

// println!("{:?}", eq(cons(1,2), cons(3, NULL)));

//println!("{:?}", and(x, y, z));
// println!("{:?}", call_goal(fresh(|q| both(eq(z, 0), and(q, q, z)))));
// println!("{:?}", call_goal(and(x, y, x)).iter().map(|s| (s.resolve(x), s.resolve(y))).collect::<Vec<_>>());
// println!("{:?}", run(|x| and(x, x, x)));

// println!("{:?}", call_goal(eq(1, 1)));
// println!("{:?}", call_goal(eq(1, 2)));
// println!("{:?}", call_goal(eq(Var(1), Var(1))));
// println!("{:?}", call_goal(eq(Var(1), Var(2))));
