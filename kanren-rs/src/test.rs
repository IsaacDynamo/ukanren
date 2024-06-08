#[cfg(test)]
mod tests {
    use crate::display::*;
    use crate::*;

    #[test]
    fn test_unify() {
        use Term::Value;
        let e = Mapping::default();

        fn unify(a: &Term, b: &Term, map: &Mapping) -> Option<Mapping> {
            let mut u = Unify::new(map.clone());
            u.unify(a, b).map(|_| u.map)
        }

        assert_eq!(format!("{:?}", unify(&Value(1), &Value(1), &e)), "Some({})");
        assert_eq!(format!("{:?}", unify(&Value(1), &Value(2), &e)), "None");
        assert_eq!(
            format!("{:?}", unify(&Var(1).into(), &Var(1).into(), &e)),
            "Some({})"
        );
        assert_eq!(
            format!("{:?}", unify(&Var(1).into(), &Var(2).into(), &e)),
            "Some({Var(2): Var(Var(1), Any)})"
        );
        assert_eq!(format!("{:?}", unify(&NULL, &NULL, &e)), "Some({})");
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
            "Some({Var(2): Var(Var(1), Any)})"
        );
    }

    #[test]
    fn test_type_unify() {
        fn unify(a: &Term, b: &Term, map: &Mapping) -> Option<Mapping> {
            let mut u = Unify::new(map.clone());
            u.unify(a, b).map(|_| u.map)
        }

        let e = Mapping::default();
        assert_eq!(unify(&ANY, &Term::Value(1), &e), Some(HashMap::new()));
        assert_eq!(unify(&ANY, &Term::String("hi".to_string()), &e), Some(HashMap::new()));
        assert_eq!(unify(&ANY, &list!(1), &e), Some(HashMap::new()));
        assert_eq!(unify(&ANY, &NULL, &e), Some(HashMap::new()));

        assert_eq!(unify(&ANY, &Term::Var(Var(1), TermType::Any), &e), Some(HashMap::new()));
        assert_eq!(unify(&ANY, &Term::Var(Var(1), TermType::Number), &e), None);
        assert_eq!(unify(&ANY, &Term::Var(Var(1), TermType::String), &e), None);

        assert_eq!(unify(&ANY, &ANY, &e), Some(HashMap::new()));
        assert_eq!(unify(&ANY, &NUM, &e), None);
        assert_eq!(unify(&ANY, &STR, &e), None);

        assert_eq!(unify(&NUM, &Term::Value(1), &e), Some(HashMap::new()));
        assert_eq!(unify(&NUM, &Term::String("hi".to_string()), &e), None);
        assert_eq!(unify(&NUM, &list!(1), &e), None);
        assert_eq!(unify(&NUM, &NULL, &e), None);

        assert_eq!(unify(&NUM, &Term::Var(Var(1), TermType::Any), &e), Some(HashMap::from_iter([(Var(1), Term::Var(Var(1), TermType::Number))].into_iter())));
        assert_eq!(unify(&NUM, &Term::Var(Var(1), TermType::Number), &e), Some(HashMap::new()));
        assert_eq!(unify(&NUM, &Term::Var(Var(1), TermType::String), &e), None);

        assert_eq!(unify(&NUM, &ANY, &e), None);
        assert_eq!(unify(&NUM, &NUM, &e), Some(HashMap::new()));
        assert_eq!(unify(&NUM, &STR, &e), None);

        assert_eq!(unify(&STR, &Term::Value(1), &e), None);
        assert_eq!(unify(&STR, &Term::String("hi".to_string()), &e), Some(HashMap::new()));
        assert_eq!(unify(&STR, &list!(1), &e), None);
        assert_eq!(unify(&STR, &NULL, &e), None);

        assert_eq!(unify(&STR, &Term::Var(Var(1), TermType::Any), &e), Some(HashMap::from_iter([(Var(1), Term::Var(Var(1), TermType::String))].into_iter())));
        assert_eq!(unify(&STR, &Term::Var(Var(1), TermType::Number), &e), None);
        assert_eq!(unify(&STR, &Term::Var(Var(1), TermType::String), &e), Some(HashMap::new()));

        assert_eq!(unify(&STR, &ANY, &e), None);
        assert_eq!(unify(&STR, &NUM, &e), None);
        assert_eq!(unify(&STR, &STR, &e), Some(HashMap::new()));
    }

    #[test]
    fn test_number_display() {
        assert_eq!(AsScheme(run_all(|| num(1))).to_string(), "(())");
        assert_eq!(AsScheme(run_all(|| num(""))).to_string(), "()");
        assert_eq!(AsScheme(run_all(|x| num(x))).to_string(), "((#0))");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(AsScheme(run_all(|x| eq(x, list!(ANY, ANY)))).to_string(), "(((_ _)))");
        assert_eq!(AsScheme(run_all(|x| eq(x, list!(NUM, NUM)))).to_string(), "(((# #)))");
        assert_eq!(AsScheme(run_all(|x| eq(x, list!(STR, STR)))).to_string(), "(((* *)))");
        assert_eq!(AsScheme(run_all(|x| eq(x, list!(ANY, . ANY)))).to_string(), "(((_ . _)))");
        assert_eq!(AsScheme(run_all(|x, y| all([eq(x, list!(y, y)), eq(y, NUM) ]))).to_string(), "(((#1 #1) #1))");

        let result = run(10, |result| {
            fresh(move |a, b, c| { all(vec![
                eq(result, list![a, b, c]),
                num(b)
            ])})
        });
        assert_eq!(AsScheme(result).to_string(), "(((_1 #2 _3)))");
    }

    #[test]
    fn test_number_eq() {
        assert_eq!(AsScheme(run_all(|x| all([eq(x, 1), num(x)]) )).to_string(), "((1))");
        assert_eq!(AsScheme(run_all(|x| all([num(x), eq(x, 1)]) )).to_string(), "((1))");
        assert_eq!(AsScheme(run_all(|x, y| all([
            eq(list!(x, y), list!(1, NUM)),
            eq(list!(x, y), list!(NUM, 2))
        ]))).to_string(), "((1 2))");

        assert_eq!(AsScheme(run_all(|x, y| all([
            num(x),
            num(y),
        ]))).to_string(), "((#0 #1))");

        assert_eq!(AsScheme(run_all(|x, y| all([
            num(x),
            num(y),
            eq(x, y)
        ]))).to_string(), "((#0 #0))");

        assert_eq!(AsScheme(run_all(|x, y| all([num(x), str(y), eq(x, y)]) )).to_string(), "()");
    }

    #[test]
    fn test_number_neq() {
        assert_eq!(AsScheme(run_all(|x| all([neq(x, NUM)]) )).to_string(), "((_0) : (((_0 . #0))))"); // (_0 . #0) -> (_0 . #)
        assert_eq!(AsScheme(run_all(|x| all([neq(x, NUM), neq(x, STR)]) )).to_string(), "((_0) : (((_0 . #0)) ((_0 . *0))))");
        assert_eq!(AsScheme(run_all(|x| all([neq(x, 1), num(x)]) )).to_string(), "((#0) : (((_0 . 1))))");
        assert_eq!(AsScheme(run_all(|x| all([num(x), neq(x, 1),]) )).to_string(), "((#0) : (((_0 . 1))))");
    }

    #[test]
    fn test_operators() {
        assert_eq!(AsScheme(run_all(|| eq(1, 1))).to_string(), "(())");
        assert_eq!(AsScheme(run_all(|| eq(1, 2))).to_string(), "()");

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
            AsScheme(run_all(|| fresh(|x, y| both(eq(x, 1), eq(y, 2))))).to_string(),
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
    fn test_list() {
        let a = list!();
        let b = NULL;
        assert_eq!(a, b);

        let a = list!(1, 2, 3);
        let b = cons(1, cons(2, cons(3, NULL)));
        assert_eq!(a, b);
    }

    #[test]
    fn test_string() {
        let a = "hello";
        let b = String::from("world");
        let c = &String::from("!");
        let l = list!(a, b, c);

        assert_eq!(AsScheme(l).to_string(), "(hello world !)");
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

    fn bounded(set: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(set, NULL)],
                vec![eq(set, cons(head, tail)), jield(move || bounded(tail))],
            ])
        })
    }

    fn contains(set: Var, x: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(set, cons(head, tail)), eq(head, x), bounded(tail)],
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
            AsScheme(run_all(|| fresh(move |x, y| all([
                eq(x, cons(1, cons(2, NULL))),
                eq(y, cons(1, cons(2, cons(3, NULL)))),
                subset(x, y),
            ]))))
            .to_string(),
            "(())"
        );

        assert_eq!(
            AsScheme(run_all(|| fresh(move |x, y| all([
                eq(x, cons(1, cons(2, cons(3, NULL)))),
                eq(y, cons(1, cons(2, NULL))),
                subset(x, y),
            ]))))
            .to_string(),
            "()"
        );

        assert_eq!(
            AsScheme(run_all(|| fresh(move |x, y| all([
                eq(x, cons(1, cons(2, NULL))),
                eq(y, cons(1, cons(2, cons(3, NULL)))),
                superset(x, y),
            ]))))
            .to_string(),
            "()"
        );

        assert_eq!(
            AsScheme(run_all(|| fresh(move |x, y| all([
                eq(x, cons(1, cons(2, cons(3, NULL)))),
                eq(y, cons(1, cons(2, NULL))),
                superset(x, y),
            ]))))
            .to_string(),
            "(())"
        );

        assert_eq!(
            AsScheme(run_all(|| fresh(move |x| all([
                eq(x, cons(1, cons(2, cons(1, NULL)))),
                set_minimal(x),
            ]))))
            .to_string(),
            "()"
        );

        assert_eq!(
            AsScheme(run_all(|| fresh(move |x| all([
                eq(x, cons(1, cons(2, cons(3, NULL)))),
                set_minimal(x),
            ]))))
            .to_string(),
            "(())"
        );

        assert_eq!(
            AsScheme(run_all(|| fresh(move |x| all([
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
                || fresh(move |x, y, set| all([
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

        let first = q.iter().take(2).collect::<Vec<_>>();
        assert_eq!(AsScheme(first).to_string(), "(((1 2)) ((2 1)))");

        // for _ in 0..10_000 {
        //     q.pull();
        // }
        // println!("{}", GoalTree(&q.goal));

        // Doesn't terminate
        let remainder = q.iter().take(2).collect::<Vec<_>>();
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
            "((_0 _1) : (((_1 . _0))))"
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
            "((_0 _0))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(cons(5, x), cons(5, y)))).to_string(),
            "((_0 _1) : (((_1 . _0))))"
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
    fn neq_case1() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([neq(x, y), neq(x, 6)]))).to_string(),
            "((_0 _1) : (((_0 . 6)) ((_1 . _0))))"
        );
    }

    #[test]
    fn constraint_test_todos() {
        assert_eq!(
            AsScheme(run_all(|_| fresh(move |x, y| neq(x, y)))).to_string(),
            "((_0))"
        );
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x| neq(q, x)))).to_string(),
            "((_0))"
        );
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x, y, z| all([
                neq(cons(y, z), x),
                eq(q, cons(x, cons(y, cons(z, NULL))))
            ]))))
            .to_string(),
            "(((_1 _2 _3)) : (((_1 . (_2 . _3)))))"
        );
        assert_eq!(
            AsScheme(run_all(|a, b| fresh(move |x, y| all([
                eq(a, x),
                eq(x, y),
                neq(y, b)
            ]))))
            .to_string(),
            "((_0 _1) : (((_1 . _0))))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(x, y))).to_string(),
            "((_0 _1) : (((_1 . _0))))"
        );
        assert_eq!(
            AsScheme(run_all(|x, y| neq(y, x))).to_string(),
            "((_0 _1) : (((_1 . _0))))"
        );
    }

    #[test]
    fn sudoku() {
        fn number(x: Var) -> Goal {
            any([
                eq(x, 1),
                eq(x, 2),
                eq(x, 3),
                eq(x, 4),
                eq(x, 5),
                eq(x, 6),
                eq(x, 7),
                eq(x, 8),
                eq(x, 9),
            ])
        }

        assert_eq!(
            AsScheme(run_all(|q| number(q))).to_string(),
            "((1) (2) (3) (4) (5) (6) (7) (8) (9))"
        );
    }

    #[test]
    fn example2() {
        fn humans(x: Var) -> Goal {
            eq(x, list!("alice", "bob"))
        }

        fn adult(x: Var, a: Var, c: Var) -> Goal {
            all([contains(a, x), excludes(c, x)])
        }

        fn child(x: Var, a: Var, c: Var) -> Goal {
            all([excludes(a, x), contains(c, x)])
        }

        fn population(hs: Var, a: Var, c: Var) -> Goal {
            fresh(move |h, ht| {
                cond([
                    vec![eq(hs, NULL)],
                    vec![
                        eq(hs, cons(h, ht)),
                        any([adult(h, a, c), child(h, a, c)]),
                        jield(move || population(ht, a, c)),
                    ],
                ])
            })
        }

        let result = AsScheme(run(6, |a, c| {
            fresh(move |x| all([humans(x), population(x, a, c)]))
        }))
        .to_string();

        assert!(result.contains("((bob) (alice))"));
        assert!(result.contains("((alice) (bob))"));
        assert!(result.contains("((bob alice) ())"));
        assert!(result.contains("(() (bob alice))"));
        assert!(result.contains("(() (alice bob))"));
        assert!(result.contains("((alice bob) ())"));
    }
}

#[cfg(test)]
mod constraints {
    use crate::display::*;
    use crate::*;

    #[test]
    fn case1() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([neq(x, 1), neq(y, 2)]))).to_string(),
            "((_0 _1) : (((_0 . 1)) ((_1 . 2))))"
        );
    }

    #[test]
    fn case2() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([neq(cons(x, y), cons(1, 2))]))).to_string(),
            "((_0 _1) : (((_0 . 1) (_1 . 2))))"
        );
    }

    #[test]
    fn case3() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([
                neq(cons(x, y), cons(1, 2)),
                fresh(move |t| any([eq(cons(x, y), cons(1, t)), eq(cons(x, y), cons(t, 2))]))
            ])))
            .to_string(),
            "((1 _1) : (((_1 . 2))) (_0 2) : (((_0 . 1))))"
        );
    }
}

#[cfg(test)]
mod minimal_constraints {
    use crate::display::*;
    use crate::*;

    #[test]
    fn case1a() {
        assert_eq!(
            AsScheme(run_all(|q, x| all([
                neq(q, cons(5, cons(x, x))),
                eq(q, cons(5, cons(1, 1)))
            ])))
            .to_string(),
            "(((5 1 . 1) _1) : (((_1 . 1))))"
        );
    }

    #[test]
    fn case1b() {
        assert_eq!(
            AsScheme(run_all(|q, x| all([
                neq(q, cons(5, cons(x, x))),
                eq(x, 1),
                eq(q, cons(5, cons(1, 1)))
            ])))
            .to_string(),
            "()"
        );
    }

    #[test]
    fn case1c() {
        assert_eq!(
            AsScheme(run_all(|q, x| all([
                neq(q, cons(5, cons(x, x))),
                neq(x, 1),
                eq(q, cons(5, cons(1, 1)))
            ])))
            .to_string(),
            "(((5 1 . 1) _1) : (((_1 . 1))))"
        );
    }

    #[test]
    fn case2() {
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x| cond([
                vec![neq(q, cons(5, cons(x, x))), neq(x, 1)],
                vec![neq(q, 5), neq(x, 1)]
            ]))))
            .to_string(),
            "((_0) (_0) : (((_0 . 5))))"
        );
    }

    #[test]
    fn case3a() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([
                neq(cons(x, y), cons(1, 2)),
                neq(x, 1),
                neq(y, 2)
            ])))
            .to_string(),
            "((_0 _1) : (((_0 . 1)) ((_1 . 2))))"
        );
    }

    #[test]
    fn case3b() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([
                neq(cons(x, y), cons(1, 2)),
                neq(x, 1)
            ])))
            .to_string(),
            "((_0 _1) : (((_0 . 1))))"
        );
    }

    #[test]
    fn case3c() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([neq(cons(x, y), cons(1, 2)),]))).to_string(),
            "((_0 _1) : (((_0 . 1) (_1 . 2))))"
        );
    }

    #[test]
    fn case4a() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([
                neq(cons(x, y), cons(1, 2)),
                neq(cons(x, y), cons(1, 3)),
            ])))
            .to_string(),
            "((_0 _1) : (((_0 . 1) (_1 . 2)) ((_0 . 1) (_1 . 3))))"
        );
    }

    #[test]
    fn case4b() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([
                neq(cons(x, y), cons(1, 2)),
                neq(cons(x, y), cons(1, 3)),
                neq(x, 1)
            ])))
            .to_string(),
            "((_0 _1) : (((_0 . 1))))"
        );
    }

    #[test]
    fn case4c() {
        assert_eq!(
            AsScheme(run_all(|x, y| all([
                neq(cons(x, y), cons(1, 2)),
                neq(cons(x, y), cons(1, 3)),
                neq(y, 2),
                neq(y, 3)
            ])))
            .to_string(),
            "((_0 _1) : (((_1 . 2)) ((_1 . 3))))"
        );
    }

    #[test]
    fn case5a() {
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x| all([neq(
                q,
                cons(5, cons(x, x))
            )]))))
            .to_string(),
            "((_0))"
        );
    }

    #[test]
    fn case5b() {
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x| all([
                neq(q, cons(5, cons(x, x))),
                eq(x, 1)
            ]))))
            .to_string(),
            "((_0) : (((_0 . (5 1 . 1)))))"
        );
    }

    #[test]
    fn case6() {
        assert_eq!(
            AsScheme(run_all(|q| fresh(move |x, y| neq(q, cons(x, y))))).to_string(),
            "((_0))"
        );
    }
}

#[cfg(test)]
mod mininal_contraints_add {
    use crate::{mininal_contraints_add, set, Term, Var};

    #[test]
    fn equal() {
        let mut minimal = Vec::new();
        mininal_contraints_add(&mut minimal, set!((Var(0), Term::Value(0))));
        mininal_contraints_add(&mut minimal, set!((Var(0), Term::Value(0))));

        let result = vec![set!((Var(0), Term::Value(0)))];

        assert_eq!(minimal, result);
    }

    #[test]
    fn disjoint() {
        let mut minimal = Vec::new();
        mininal_contraints_add(&mut minimal, set!((Var(0), Term::Value(0))));
        mininal_contraints_add(&mut minimal, set!((Var(1), Term::Value(1))));

        let result = vec![
            set!((Var(0), Term::Value(0))),
            set!((Var(1), Term::Value(1))),
        ];

        assert_eq!(minimal, result);
    }

    #[test]
    fn joint() {
        let mut minimal = Vec::new();
        mininal_contraints_add(
            &mut minimal,
            set!((Var(0), Term::Value(0)), (Var(1), Term::Value(1))),
        );
        mininal_contraints_add(
            &mut minimal,
            set!((Var(0), Term::Value(0)), (Var(2), Term::Value(2))),
        );

        let result = vec![
            set!((Var(0), Term::Value(0)), (Var(1), Term::Value(1))),
            set!((Var(0), Term::Value(0)), (Var(2), Term::Value(2))),
        ];

        assert_eq!(minimal, result);
    }

    #[test]
    fn subset() {
        let mut minimal = Vec::new();
        mininal_contraints_add(
            &mut minimal,
            set!((Var(0), Term::Value(0)), (Var(1), Term::Value(1))),
        );
        mininal_contraints_add(&mut minimal, set!((Var(0), Term::Value(0))));

        let result = vec![set!((Var(0), Term::Value(0)))];

        assert_eq!(minimal, result);
    }

    #[test]
    fn superset() {
        let mut minimal = Vec::new();
        mininal_contraints_add(&mut minimal, set!((Var(0), Term::Value(0))));
        mininal_contraints_add(
            &mut minimal,
            set!((Var(0), Term::Value(0)), (Var(1), Term::Value(1))),
        );

        let result = vec![set!((Var(0), Term::Value(0)))];

        assert_eq!(minimal, result);
    }
}

#[test]
fn example1() {
    use crate::display::AsScheme;
    use crate::*;

    fn facts(x: Var, y: Var, z: Var) -> Goal {
        cond([
            vec![eq(x, "male"), eq(y, "monarch"), eq(z, "king")],
            vec![eq(x, "female"), eq(y, "monarch"), eq(z, "queen")],
        ])
    }

    assert_eq!(
        AsScheme(run_all(|q| fresh(move |king, male, female, x| all([
            eq(king, "king"),
            eq(male, "male"),
            eq(female, "female"),
            facts(male, x, king),
            facts(female, x, q)
        ]))))
        .to_string(),
        "((queen))"
    );
}

#[test]
fn term_args() {
    use crate::display::AsScheme;
    use crate::*;

    fn facts(x: impl Into<Term>, y: impl Into<Term>, z: impl Into<Term>) -> Goal {
        let x = &x.into();
        let y = &y.into();
        let z = &z.into();

        all([cond([
            vec![eq(x, "male"), eq(y, "monarch"), eq(z, "king")],
            vec![eq(x, "female"), eq(y, "monarch"), eq(z, "queen")],
        ])])
    }

    assert_eq!(
        AsScheme(run_all(|q| fresh(move |x| all([
            facts("male", x, "king"),
            facts("female", x, q),
        ]))))
        .to_string(),
        "((queen))"
    );
}

#[test]
fn three_brothers() {
    use crate::display::AsScheme;
    use crate::*;

    fn brothers(name: Var, tells: Var) -> Goal {
        cond([
            [eq(name, "John"), eq(tells, "lies")],
            [eq(name, "James"), eq(tells, "lies")],
            [eq(name, "William"), eq(tells, "truth")],
        ])
    }

    fn is(a: Var, b: Var, answer: Var) -> Goal {
        cond([[eq(a, b), eq(answer, "yes")], [neq(a, b), eq(answer, "no")]])
    }

    fn says(tells: Var, result: Var, answer: Var) -> Goal {
        cond([
            [eq(tells, "truth"), eq(result, "yes"), eq(answer, "yes")],
            [eq(tells, "truth"), eq(result, "no"), eq(answer, "no")],
            [eq(tells, "lies"), eq(result, "yes"), eq(answer, "no")],
            [eq(tells, "lies"), eq(result, "no"), eq(answer, "yes")],
        ])
    }

    // Hardcoded question "Is your name ...?"
    // Query will find what name to ask, and what unique answer Johns will give.
    let result = run_all(|name, unique| {
        fresh(move |common| {
            all([
                neq(unique, common),
                fresh(move |your_name, tells, result| {
                    all([
                        eq(your_name, "John"),
                        brothers(your_name, tells),
                        is(your_name, name, result),
                        says(tells, result, unique),
                    ])
                }),
                fresh(move |your_name, tells, result| {
                    all([
                        eq(your_name, "James"),
                        brothers(your_name, tells),
                        is(your_name, name, result),
                        says(tells, result, common),
                    ])
                }),
                fresh(move |your_name, tells, result| {
                    all([
                        eq(your_name, "William"),
                        brothers(your_name, tells),
                        is(your_name, name, result),
                        says(tells, result, common),
                    ])
                }),
            ])
        })
    });

    assert_eq!(AsScheme(result).to_string(), "((James yes))");
}

#[test]
fn three_brothers_v2() {
    use crate::display::AsScheme;
    use crate::*;

    goal!(
        fn bounded(set: Var) -> Goal {
            fresh(move |head, tail| {
                cond([
                    vec![eq(set, NULL)],
                    vec![eq(set, cons(head, tail)), jield(move || bounded(tail))],
                ])
            })
        }
    );

    goal!(
        fn contains(set: Var, x: Var) -> Goal {
            fresh(move |head, tail| {
                cond([
                    vec![eq(set, cons(head, tail)), eq(head, x), bounded(tail)],
                    vec![
                        eq(set, cons(head, tail)),
                        neq(head, x),
                        jield(move || contains(tail, x)),
                    ],
                ])
            })
        }
    );

    goal!(
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
    );

    goal!(
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
    );

    fn not(a: Var, b: Var) -> Goal {
        cond([[eq(a, "yes"), eq(b, "no")], [eq(a, "no"), eq(b, "yes")]])
    }

    goal!(
        fn says(tells: Var, result: Var, answer: Var) -> Goal {
            cond([
                [eq(tells, "truth"), eq(result, answer)],
                [eq(tells, "lies"), not(result, answer)],
            ])
        }
    );

    goal!(
        fn query(question: Var, facts: Var, result: Var) -> Goal {
            fresh(move |index, identifier, prop, value| {
                all([
                    eq(question, list!(identifier, prop, value)),
                    any([
                        contains(facts, list!(index, "is", identifier)),
                        contains(facts, list!(index, "named", identifier)),
                    ]),
                    cond([
                        [
                            contains(facts, list!(index, prop, value)),
                            eq(result, "yes"),
                        ],
                        [excludes(facts, list!(index, prop, value)), eq(result, "no")],
                    ]),
                ])
            })
        }
    );

    goal!(
        fn ask_question(index: Var, question: Var, answer: Var) -> Goal {
            fresh(move |result, facts, my_facts, tells| {
                all([
                    get_facts(facts),
                    set_insert(facts, list!(index, "is", "You"), my_facts),
                    query(question, my_facts, result),
                    contains(facts, list!(index, "tells", tells)),
                    says(tells, result, answer),
                ])
            })
        }
    );

    goal!(
        fn get_facts(f: Var) -> Goal {
            eq(
                f,
                list!(
                    list!("obj#john", "named", "John"),
                    list!("obj#john", "tells", "lies"),
                    list!("obj#james", "named", "James"),
                    list!("obj#james", "tells", "lies"),
                    list!("obj#william", "named", "William"),
                    list!("obj#william", "tells", "truth"),
                ),
            )
        }
    );

    // Find question and answer that would uniquely identify John
    let result = run_all(|question, answer_john| {
        fresh(move |answer_james, answer_william| {
            all([
                neq(answer_john, answer_james),
                neq(answer_john, answer_william),
                ask_question("obj#john", question, answer_john),
                ask_question("obj#james", question, answer_james),
                ask_question("obj#william", question, answer_william),
            ])
        })
    });

    assert_eq!(
        AsScheme(result).to_string(),
        "(((You named James) yes) ((James is You) yes))"
    );
}

#[test]
fn paradox() {
    use crate::display::AsScheme;
    use crate::*;

    fn eqv(a: Var, b: Var, result: Var) -> Goal {
        cond([
            [eq(a, b), eq(result, "true")],
            [neq(a, b), eq(result, "false")],
        ])
    }

    // This sentence is true
    // sentence = (sentence == true)
    let result =
        run_all(|sentence| fresh(move |x| all([eq(x, "true"), eqv(sentence, x, sentence)])));
    assert_eq!(AsScheme(result).to_string(), "((true) (false))");

    fn not(x: Var, y: Var) -> Goal {
        cond([
            [eq(x, "true"), eq(y, "false")],
            [eq(x, "false"), eq(y, "true")],
        ])
    }

    // This sentence is not true
    // sentence = !(sentence == true)
    let result = run_all(|sentence| {
        fresh(move |x, y| all([eq(x, "true"), eqv(sentence, x, y), not(y, sentence)]))
    });
    assert_eq!(AsScheme(result).to_string(), "()");

    // https://en.wikipedia.org/wiki/Three-valued_logic
    // LP (Logic of Paradox)

    // logical equivalence or biconditional, <->
    fn lp_leq(a: Var, b: Var, result: Var) -> Goal {
        cond([
            vec![eq(a, "false"), eq(b, "false"), eq(result, "true")],
            vec![eq(a, "true"), eq(b, "true"), eq(result, "true")],
            vec![eq(a, "false"), eq(b, "true"), eq(result, "false")],
            vec![eq(a, "true"), eq(b, "false"), eq(result, "false")],
            vec![eq(a, "true"), eq(b, "undecided"), eq(result, "undecided")],
            vec![eq(a, "false"), eq(b, "undecided"), eq(result, "undecided")],
            vec![eq(a, "undecided"), eq(b, "true"), eq(result, "undecided")],
            vec![eq(a, "undecided"), eq(b, "false"), eq(result, "undecided")],
            vec![
                eq(a, "undecided"),
                eq(b, "undecided"),
                eq(result, "undecided"),
            ], // In other logics this can be true
        ])
    }

    fn lp_not(x: Var, y: Var) -> Goal {
        cond([
            [eq(x, "true"), eq(y, "false")],
            [eq(x, "undecided"), eq(y, "undecided")],
            [eq(x, "false"), eq(y, "true")],
        ])
    }

    // This sentence is not true
    // sentence = !(sentence <-> true)
    let result = run_all(|sentence| {
        fresh(move |x, y| all([eq(x, "true"), lp_leq(sentence, y, sentence), lp_not(y, x)]))
    });
    assert_eq!(AsScheme(result).to_string(), "((undecided))");
}

#[test]
fn goal_macro() {
    use crate::display::AsScheme;
    use crate::*;

    goal!(
        fn eqv(a: Var, b: Var, result: Var) -> Goal {
            cond([
                [eq(a, b), eq(result, "true")],
                [neq(a, b), eq(result, "false")],
            ])
        }
    );

    let result = run_all(|a, b, c| {
        fresh(move |_| all([eqv("Hello", 42, a), eqv(a, "false", b), eqv(b, "true", c)]))
    });
    assert_eq!(AsScheme(result).to_string(), "((false true true))");
}

#[test]
fn interpeter() {
    use crate::display::AsScheme;
    use crate::*;

    fn append(a: Var, b: Var, c: Var) -> Goal {
        cond([
            vec![eq(a, NULL), eq(c, b)],
            vec![fresh(move |head, tail, x| {
                all([
                    eq(a, cons(head, tail)),
                    eq(c, cons(head, x)),
                    jield(move || append(tail, b, x)),
                ])
            })],
        ])
    }

    // quote, atom, eq, car, cdr, cons and cond

    goal!(
        fn eval(expr: Var, result: Var) -> Goal {
            cond([
                vec![eq(expr, 1), eq(result, 1)],
                vec![eq(expr, 2), eq(result, 2)],
                vec![eq(expr, 3), eq(result, 3)],
                vec![eq(expr, list!("quote", result))],
                vec![eq(expr, "nil"), eq(result, NULL)],
                vec![eq(expr, NULL), eq(result, NULL)],
                vec![fresh(move |list, unused, list_eval| {
                    all([
                        eq(expr, list!("first", list)),
                        eq(list_eval, cons(result, unused)),
                        jield(move || eval(list, list_eval)),
                    ])
                })],
                vec![fresh(move |list, unused, list_eval| {
                    all([
                        eq(expr, list!("rest", list)),
                        eq(list_eval, cons(unused, result)),
                        jield(move || eval(list, list_eval)),
                    ])
                })],
                vec![fresh(move |a, b, a_eval, b_eval| {
                    all([
                        eq(expr, list!("cons", a, b)),
                        eq(result, cons(a_eval, b_eval)),
                        jield(move || eval(a, a_eval)),
                        jield(move || eval(b, b_eval)),
                    ])
                })],
                vec![fresh(move |e, ee| {
                    all([
                        eq(expr, list!("eval", e)),
                        jield(move || eval(e, ee)),
                        jield(move || eval(ee, result)),
                    ])
                })],
            ])
        }
    );

    let result = run(10, |result| {
        eval(
            list!("cons", 1, list!("cons", 2, list!("cons", 3, "nil"))),
            result,
        )
    });
    assert_eq!(AsScheme(result).to_string(), "(((1 2 3)))");

    let result = run(10, |result| eval(list!("quote", list!()), result));
    assert_eq!(AsScheme(result).to_string(), "((()))");

    let result = run(10, |result| eval("nil", result));
    assert_eq!(AsScheme(result).to_string(), "((()))");

    let result = run(10, |result| {
        eval(list!("quote", list!("cons", 1, 2)), result)
    });
    assert_eq!(AsScheme(result).to_string(), "(((cons 1 2)))");

    let result = run(10, |result| {
        eval(list!("eval", list!("quote", list!("cons", 1, 2))), result)
    });
    assert_eq!(AsScheme(result).to_string(), "(((1 . 2)))");

    let result = run(10, |expr| eval(expr, list!(1, 2, 3)));
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());

    let result = run(3, |r| eval(r, r));
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());
}

#[test]
fn json_de_ser() {
    use crate::display::AsScheme;
    use crate::*;

    goal!(
        fn member(str: Var, rem: Var, expr: Var) -> Goal {
            fresh(move |key, value, e| {
                all([
                    eq(str, cons(key, cons(":", e))),
                    eq(expr, cons(key, value)),
                    element(e, rem, value),
                ])
            })
        }
    );

    goal!(
        fn members(str: Var, rem: Var, expr: Var) -> Goal {
            cond([
                vec![fresh(move |e| {
                    all([member(str, rem, e), eq(expr, list!(e))])
                })],
                vec![fresh(move |e, es, comma, ms| {
                    all([
                        eq(comma, cons(",", ms)),
                        eq(expr, cons(e, es)),
                        member(str, comma, e),
                        jield(move || members(ms, rem, es)),
                    ])
                })],
            ])
        }
    );

    goal!(
        fn object(str: Var, rem: Var, expr: Var) -> Goal {
            cond([
                vec![fresh(move |tail, close| {
                    all([
                        eq(str, cons("{", tail)),
                        eq(close, cons("}", rem)),
                        members(tail, close, expr),
                    ])
                })],
                vec![eq(str, cons("{", cons("}", rem))), eq(expr, NULL)],
            ])
        }
    );

    goal!(
        fn value(str: Var, rem: Var, expr: Var) -> Goal {
            cond([
                vec![eq(str, cons("false", rem)), eq(expr, "#f")],
                vec![eq(str, cons("true", rem)), eq(expr, "#t")],
                vec![eq(str, cons(expr, rem)), eq(expr, "bye")],
                vec![eq(str, cons(expr, rem)), eq(expr, "world")],
                vec![eq(str, cons(expr, rem)), eq(expr, "night")],
                vec![jield(move || object(str, rem, expr))],
                vec![jield(move || array(str, rem, expr))],
            ])
        }
    );

    goal!(
        fn element(str: Var, rem: Var, expr: Var) -> Goal {
            value(str, rem, expr)
        }
    );

    goal!(
        fn elements(str: Var, rem: Var, expr: Var) -> Goal {
            cond([
                vec![fresh(move |e| {
                    all([eq(expr, list!(e)), element(str, rem, e)])
                })],
                vec![fresh(move |e, es, comma, ms| {
                    all([
                        eq(comma, cons(",", ms)),
                        eq(expr, cons(e, es)),
                        element(str, comma, e),
                        jield(move || elements(ms, rem, es)),
                    ])
                })],
            ])
        }
    );

    goal!(
        fn array(str: Var, rem: Var, expr: Var) -> Goal {
            cond([
                vec![fresh(move |tail, close| {
                    all([
                        eq(str, cons("[", tail)),
                        eq(close, cons("]", rem)),
                        elements(tail, close, expr),
                    ])
                })],
                vec![eq(str, cons("[", cons("]", rem))), eq(expr, NULL)],
            ])
        }
    );

    goal!(
        fn json(str: Var, expr: Var) -> Goal {
            element(str, NULL, expr)
        }
    );

    let result = run(1, |expr| json(list!("{", "}"), expr));
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());

    let result = run(1, |expr| json(list!("{", "hello", ":", "world", "}"), expr));
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());

    let result = run(1, |expr| json(expr, list!(cons("bye", "bye"))));
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());

    let result = run(1, |expr| {
        json(
            list!("{", "hello", ":", "world", ",", "good", ":", "night", "}"),
            expr,
        )
    });
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());

    let result = run(100, |str, expr| json(str, expr));
    //assert_eq!(AsScheme(result).to_string(), "((false true true))");
    println!("{}", AsScheme(result).to_string());
}

#[test]
fn depth() {
    use crate::*;

    let mut q = query(move |x| cond([[list::contains(list!(1, 2, 3), x)]]));

    assert_eq!(q.next().map(|s| s.depth), Some(0));
    assert_eq!(q.next().map(|s| s.depth), Some(1));
    assert_eq!(q.next().map(|s| s.depth), Some(2));
    assert_eq!(q.next().map(|s| s.depth), None);
}

#[test]
fn tree() {
    use crate::*;

    let mut q = query(move |x, y| {
        cond([
            vec![any([eq(y, 1), eq(y, 2)]), list::contains(list!(1, 2), x)],
            vec![list::contains(list!(3, 4), x)],
        ])
    });

    println!("{} {}", q.stream.mature.len(), q.stream.immature.len());

    println!("{}", display::GoalTree(&q.goal));
    println!("{:?}", q.next().map(|s| s.depth));
    println!("{:?}", q.next().map(|s| s.depth));
    println!("{:?}", q.next().map(|s| s.depth));
    println!("{:?}", q.next().map(|s| s.depth));
    println!("{:?}", q.next().map(|s| s.depth));
    println!("{:?}", q.next().map(|s| s.depth));
    println!("{:?}", q.next().map(|s| s.depth));

    println!("{}", display::GoalTree(&q.goal));

    println!("{}", q.id.load(Ordering::Relaxed));
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
