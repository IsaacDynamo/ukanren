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
        assert_eq!(format!("{:?}", run_all(eq(1, 1))), "[[]]");
        assert_eq!(format!("{:?}", run_all(eq(1, 2))), "[]");

        assert_eq!(
            format!("{:?}", run_all(|x| either(eq(x, 1), eq(x, 1)))),
            "[[Value(1)], [Value(1)]]"
        );
        assert_eq!(
            format!("{:?}", run_all(|x| either(eq(x, 1), eq(x, 2)))),
            "[[Value(1)], [Value(2)]]"
        );
        assert_eq!(
            format!("{:?}", run_all(|x, y| either(eq(x, 1), eq(y, 2)))),
            "[[Value(1), Var(1)], [Var(0), Value(2)]]"
        );

        assert_eq!(
            format!("{:?}", run_all(|x| both(eq(x, 1), eq(x, 1)))),
            "[[Value(1)]]"
        );
        assert_eq!(format!("{:?}", run_all(|x| both(eq(x, 1), eq(x, 2)))), "[]");
        assert_eq!(
            format!("{:?}", run_all(|x, y| both(eq(x, 1), eq(y, 2)))),
            "[[Value(1), Value(2)]]"
        );

        assert_eq!(
            format!("{:?}", run_all(fresh(|x, y| both(eq(x, 1), eq(y, 2))))),
            "[[]]"
        );
        assert_eq!(
            format!(
                "{:?}",
                run_all(|x| fresh(move |y| both(eq(x, 1), eq(y, 2))))
            ),
            "[[Value(1)]]"
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
            format!("{:?}", run_all(|x, y| and(x, y, x))),
            "[[Value(0), Value(0)], [Value(0), Value(1)], [Value(1), Value(1)]]"
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

        println!("{:?}", run(5, |x| fives(x)));
        println!("{:?}", run(5, |x| either(fives(x), sixes(x))));
        println!(
            "{:?}",
            run(8, |x, y| both(
                either(fives(x), sixes(x)),
                either(eq(y, 7), sixes(y))
            ))
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

        println!(
            "{:?}",
            run(10, move |x, y| fresh(move |r| both(
                eq(r, l()),
                concat(x, y, r)
            )))
        );

        let mut q = query(move |x, y| fresh(move |r| both(eq(r, l()), concat(x, y, r))));

        for _ in 0..14 {
            q.stream.pull()
        }

        println!("{}", GoalTree(&q.goal));
        println!("{:?}", q.stream.mature);
    }

    #[test]
    fn test_set() {
        fn contains(set: Var, x: Var) -> Goal {
            fresh(move |head, tail| {
                cond([
                    vec![eq(set, cons(head, tail)), eq(head, x)],
                    vec![
                        eq(set, cons(head, tail)),
                        /*neq(head, val),*/ jield(move || contains(tail, x)),
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
                        /*neq(head, val),*/ jield(move || excludes(tail, x)),
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
                        /*neq(head, x), */ eq(result, cons(head, c)),
                        jield(move || set_insert(tail, x, c)),
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

        println!(
            "contains: {}",
            AsScheme(run(10, |set| fresh(move |x, y, z| all([
                eq(x, 1),
                eq(y, 2),
                eq(z, 3),
                contains(set, x),
                contains(set, y),
                contains(set, z),
            ]))))
        );

        println!(
            "contains: {}",
            AsScheme(run(100, |x, y, z| fresh(move |set, t| all([
                eq(t, 1),
                eq(set, cons(1, cons(2, cons(3, NULL)))),
                contains(set, x),
                contains(set, y),
                contains(set, z),
            ]))))
        );

        println!(
            "excludes: {}",
            AsScheme(run(100, |q| fresh(move |s, x| all([
                eq(s, cons(1, cons(2, cons(3, NULL)))),
                eq(x, 2),
                excludes(s, x),
            ]))))
        );

        let mut q = query(|q| fresh(move |x| all([eq(x, cons(1, cons(2, cons(3, NULL)))), set_eq(q, x)])));
        println!("set_eq: {}", AsScheme(q.resolve().take(100).collect()));
        println!("{}", GoalTree(&q.goal));

        println!(
            "set_insert: {}",
            AsScheme(run(100, |q| fresh(move |s, x| all([
                eq(s, cons(1, cons(2, cons(3, NULL)))),
                eq(x, 3),
                set_insert(s, x, q),
            ]))))
        );

        println!(
            "set_join: {}",
            AsScheme(run(10, |q| fresh(move |a, b| all([
                eq(a, cons(1, cons(2, NULL))),
                eq(b, cons(2, cons(3, NULL))),
                set_join(a, b, q),
            ]))))
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
