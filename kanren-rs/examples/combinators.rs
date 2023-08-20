// Run with: cargo run --example combinators --release

use kanren_rs::{display::*, *};

goal!(
    fn get_var(x: Var, v: Var, v_: Var) -> Goal {
        fresh(move |dummy| {
            cond([
                vec![eq(v, NULL), eq(x, "a"), eq(v_, cons(x, v))],
                vec![eq(v, cons("a", dummy)), eq(x, "b"), eq(v_, cons(x, v))],
                vec![eq(v, cons("b", dummy)), eq(x, "c"), eq(v_, cons(x, v))],
                vec![eq(v, cons("c", dummy)), eq(x, "d"), eq(v_, cons(x, v))],
                vec![eq(v, cons("d", dummy)), eq(x, "e"), eq(v_, cons(x, v))],
            ])
        })
    }
);

goal!(
    fn bind1(e: Var, comb: Var, a: Var, rem: Var, v: Var, v_: Var) -> Goal {
        cond([
            vec![eq(e, list!(comb, a, . rem)), eq(v, v_)],
            vec![eq(e, list!(comb)), eq(rem, NULL), get_var(a, v, v_)],
        ])
    }
);

goal!(
    fn bind2(e: Var, comb: Var, a: Var, b: Var, rem: Var, v: Var, v_: Var) -> Goal {
        fresh(move |vn| {
            cond([
                vec![eq(e, list!(comb, a, b, . rem)), eq(v, v_)],
                vec![eq(e, list!(comb, a)), eq(rem, NULL), get_var(b, v, v_)],
                vec![
                    eq(e, list!(comb)),
                    eq(rem, NULL),
                    get_var(a, v, vn),
                    get_var(b, vn, v_),
                ],
            ])
        })
    }
);

goal!(
    fn bind3(e: Var, comb: Var, a: Var, b: Var, c: Var, rem: Var, v: Var, v_: Var) -> Goal {
        fresh(move |v0, v1| {
            cond([
                vec![eq(e, list!(comb, a, b, c, . rem)), eq(v, v_)],
                vec![eq(e, list!(comb, a, b)), eq(rem, NULL), get_var(c, v, v_)],
                vec![
                    eq(e, list!(comb, a)),
                    eq(rem, NULL),
                    get_var(b, v, v0),
                    get_var(c, v0, v_),
                ],
                vec![
                    eq(e, list!(comb)),
                    eq(rem, NULL),
                    get_var(a, v, v0),
                    get_var(b, v0, v1),
                    get_var(c, v1, v_),
                ],
            ])
        })
    }
);

goal!(
    fn apply(e: Var, e_: Var, v: Var, v_: Var) -> Goal {
        fresh(move |a, b, c, rem| {
            cond([
                vec![bind1(e, "I", a, rem, v, v_), eq(e_, cons(a, rem))],
                vec![bind2(e, "K", a, b, rem, v, v_), eq(e_, cons(a, rem))],
                vec![
                    bind3(e, "S", a, b, c, rem, v, v_),
                    eq(e_, list!(a, c, list!(b, c), . rem)),
                ],
            ])
        })
    }
);

goal!(
    fn var(e: Var, e_: Var, t: Var, t_: Var, v: Var) -> Goal {
        fresh(move |x| {
            cond([vec![
                eq(e, cons(x, e_)),
                eq(t_, cons(x, t)),
                list::contains(v, x),
            ]])
        })
    }
);

goal!(
    fn eval_inner(e: Var, v: Var, v_: Var, t: Var, t_: Var) -> Goal {
        cond([
            // Empty expression
            vec![eq(e, NULL), eq(v, v_), eq(t, t_)],
            // Apply combinator
            vec![fresh(move |te, tv| {
                all([
                    apply(e, te, v, tv),
                    jield(move || eval_inner(te, tv, v_, t, t_)),
                ])
            })],
            // If var, no further reduction is possible, move head to term
            vec![fresh(move |te, tt| {
                all([
                    var(e, te, t, tt, v),
                    jield(move || eval_inner(te, v, v_, tt, t_)),
                ])
            })],
            // Unwrap head if term is empty
            vec![fresh(move |head, tail, te| {
                all([
                    eq(t, NULL),
                    eq(e, cons(head, tail)),
                    list::at_least_two(head),
                    list::append(head, tail, te),
                    jield(move || eval_inner(te, v, v_, t, t_)),
                ])
            })],
            // Reduce non-head sub expressions
            vec![fresh(move |sub, res, tail, tt| {
                all([
                    list::not_empty(t),
                    eq(e, cons(sub, tail)),
                    list::at_least_two(sub),
                    cond([
                        vec![eq(tt, cons(res, t)), list::at_least_two(res)],
                        vec![fresh(move |single| {
                            all([eq(tt, cons(single, t)), eq(res, list!(single))])
                        })],
                    ]),
                    jield(move || eval_inner(sub, v, v, NULL, res)),
                    jield(move || eval_inner(tail, v, v_, tt, t_)),
                ])
            })],
        ])
    }
);

goal!(
    fn eval(expr: Var, var: Var, term: Var) -> Goal {
        eval_inner(expr, NULL, var, NULL, term)
    }
);

goal!(
    fn is_comb(comb: Var) -> Goal {
        cond([[eq(comb, "S")], [eq(comb, "K")], [eq(comb, "I")]])
    }
);

goal!(
    fn combs(expr: Var) -> Goal {
        cond([
            [eq(expr, NULL)],
            [fresh(move |comb, tail| {
                all([
                    eq(expr, cons(comb, tail)),
                    is_comb(comb),
                    jield(move || combs(tail)),
                ])
            })],
            [fresh(move |l, tail| {
                all([
                    eq(expr, cons(l, tail)),
                    list::at_least_two(l),
                    jield(move || combs(l)),
                    jield(move || combs(tail)),
                ])
            })],
        ])
    }
);

fn parse(chars: &mut impl Iterator<Item = char>) -> Term {
    let c = chars.next();
    match c {
        None => Term::Null,
        Some('(') => {
            let sub = parse(chars);
            cons(sub, parse(chars))
        }
        Some(')') => Term::Null,
        Some(c) => cons(c.to_string(), parse(chars)),
    }
}

fn fmt_lambda(vars: &Term, terms: &Term) -> String {
    let var = vars.to_vec().unwrap();
    let term = terms.to_vec().unwrap();
    let var: String = var.iter().rev().map(|x| to_string(x)).collect();
    let term: String = term.iter().rev().map(|x| to_string(x)).collect();
    format!("λ{}.{}", var, term)
}

fn fmt_list(term: &Term) -> String {
    match term {
        Term::Null => "".to_string(),
        Term::Cons(a, b) => {
            let mut result = to_string(&a);
            result.push_str(&fmt_list(&b));
            result
        }
        _ => "?".to_string(),
    }
}

fn to_string(term: &Term) -> String {
    match term {
        Term::Null => "()".to_string(),
        Term::String(x) => x.clone(),
        Term::Cons(a, b) => {
            let mut result = "(".to_string();
            let l = term.to_vec().unwrap();
            result.extend(l.iter().rev().map(|x| to_string(x)));
            result.push_str(")");
            result
        }
        _ => "?".to_string(),
    }
}

fn fmt_result(result: &Vec<StateN<2>>) -> String {
    if result.is_empty() {
        "-".to_string()
    } else {
        let [vars, terms] = result[0].reify();
        let mut l = fmt_lambda(&vars, &terms);
        if result.len() > 1 {
            l.push_str(", ...")
        }
        l
    }
}

fn test_apply(input: &str, expected: &str) {
    let expr = parse(&mut input.chars());

    let result = run(1, |v, e| apply(&expr, e, NULL, v));
    let result = AsScheme(result).to_string();
    let (mark, equal) = if result == expected {
        (" ", "==")
    } else {
        ("*", "!=")
    };
    println!("{} {:24} {:24} {} {}", mark, input, expected, equal, result);
}

fn test_eval(input: &str, expected: &str) {
    let expr = parse(&mut input.chars());

    let result = run(2, |var, term| all([eval(&expr, var, term), combs(&expr)]));
    let result = fmt_result(&result);
    let (mark, equal) = if result == expected {
        (" ", "==")
    } else {
        ("*", "!=")
    };
    println!("{} {:24} {:24} {} {}", mark, input, expected, equal, result);
}

fn main() {
    println!(
        "{} {:24} {:24} {} {}",
        " ", "Input", "Expected", "  ", "Result"
    );
    test_apply("I", "(((a) (a)))");
    test_apply("Ix", "((() (x)))");
    test_apply("Ixy", "((() (x y)))");
    test_apply("K", "(((b a) (a)))");
    test_apply("Kx", "(((a) (x)))");
    test_apply("Kxy", "((() (x)))");
    test_apply("Kxyz", "((() (x z)))");
    test_apply("S", "(((c b a) (a c (b c))))");
    test_apply("Sx", "(((b a) (x b (a b))))");
    test_apply("Sxy", "(((a) (x a (y a))))");
    test_apply("Sxyz", "((() (x z (y z))))");
    test_apply("Sxyzw", "((() (x z (y z) w)))");

    test_apply("SKK", "(((a) (K a (K a))))");
    test_apply("KK", "(((a) (K)))");
    println!();

    test_eval("I", "λa.a");
    test_eval("K", "λab.a");
    test_eval("S", "λabc.ac(bc)");

    test_eval("II", "λa.a");
    test_eval("KI", "λab.b");
    test_eval("IKI", "λab.b");
    test_eval("SK", "λab.b");
    test_eval("SKK", "λa.a");
    test_eval("SII", "λa.aa");
    test_eval("KK", "λabc.b");

    test_eval("(II)", "λa.a");
    test_eval("K(II)(II)", "λa.a");
    test_eval("S(KS)K", "λabc.a(bc)");
    test_eval("S(K(SI))K", "λab.ba");

    test_eval("S(K((S(KS))K))((S(KS))K)", "λabcd.a(bcd)"); // Blackbird
    test_eval("S((S(K((S(KS))K)))S)(KK)", "λabc.acb"); // Cardinal
    test_eval("S((SK)K)((SK)K)", "λa.aa"); // Mocking
    println!();

    let mut result = query(|expr, vars, terms| {
        all([
            list::at_least_two(vars),
            list::at_least_two(terms),
            fresh(move |tail| cond([[eq(expr, cons("S", tail))], [eq(expr, cons("K", tail))]])),
            combs(expr),
            eval(expr, vars, terms),
        ])
    });

    while let Some(state) = result.next() {
        let vars = reify::<3>(&state);
        println!(
            "{:8} {}",
            fmt_list(&vars[0]),
            fmt_lambda(&vars[1], &vars[2])
        );
    }
}
