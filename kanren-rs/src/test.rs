#[cfg(test)]
mod tests {
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
        assert_eq!(format!("{:?}", run(eq(1, 1))), "[[]]");
        assert_eq!(format!("{:?}", run(eq(1, 2))), "[]");

        assert_eq!(
            format!("{:?}", run(|x| either(eq(x, 1), eq(x, 1)))),
            "[[Value(1)], [Value(1)]]"
        );
        assert_eq!(
            format!("{:?}", run(|x| either(eq(x, 1), eq(x, 2)))),
            "[[Value(1)], [Value(2)]]"
        );
        assert_eq!(
            format!("{:?}", run(|x, y| either(eq(x, 1), eq(y, 2)))),
            "[[Value(1), Var(1)], [Var(0), Value(2)]]"
        );

        assert_eq!(
            format!("{:?}", run(|x| both(eq(x, 1), eq(x, 1)))),
            "[[Value(1)]]"
        );
        assert_eq!(format!("{:?}", run(|x| both(eq(x, 1), eq(x, 2)))), "[]");
        assert_eq!(
            format!("{:?}", run(|x, y| both(eq(x, 1), eq(y, 2)))),
            "[[Value(1), Value(2)]]"
        );

        assert_eq!(
            format!("{:?}", run(fresh(|x, y| both(eq(x, 1), eq(y, 2))))),
            "[[]]"
        );
        assert_eq!(
            format!("{:?}", run(|x| fresh(move |y| both(eq(x, 1), eq(y, 2))))),
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
            format!("{:?}", run(|x, y| and(x, y, x))),
            "[[Value(0), Value(0)], [Value(0), Value(1)], [Value(1), Value(1)]]"
        );
    }

    #[test]
    fn test_yield() {
        fn fives(x: Var) -> Goal {
            return either(eq(x, 5), jield(move || fives(x)));
        }

        fn sixes(x: Var) -> Goal {
            return either(eq(x, 6), jield(move || sixes(x)));
        }

        println!("{:?}", runx(5, |x| fives(x)));
        println!("{:?}", runx(5, |x| either(fives(x), sixes(x))));
        println!(
            "{:?}",
            runx(8, |x, y| both(
                either(fives(x), sixes(x)),
                either(eq(y, 7), sixes(y))
            ))
        );
    }

    #[test]
    fn test_concat() {
        fn concat(l: Var, r: Var, out: Var) -> Goal {
            println!("{l:?} {r:?} {out:?}");

            return jield(move || {
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
            });
        }

        fn l() -> Term {
            cons(1, cons(2, cons(3, cons(4, NULL))))
        }

        // println!(
        //     "{:?}",
        //     runx(10, move |x, y| fresh(move |r| both(
        //         eq(r, l()),
        //         concat(x, y, r)
        //     )))
        // );
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
