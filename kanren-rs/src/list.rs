use crate::goal;
use crate::*;

goal!(
    pub fn append(a: Var, b: Var, ab: Var) -> Goal {
        cond([
            vec![eq(a, NULL), eq(ab, b)],
            vec![fresh(move |head, tail, x| {
                all([
                    eq(a, cons(head, tail)),
                    eq(ab, cons(head, x)),
                    jield(move || append(tail, b, x)),
                ])
            })],
        ])
    }
);

goal!(
    pub fn contains(list: Var, x: Var) -> Goal {
        fresh(move |head, tail| {
            cond([
                vec![eq(list, cons(head, tail)), eq(head, x)],
                vec![eq(list, cons(head, tail)), jield(move || contains(tail, x))],
            ])
        })
    }
);

goal!(
    pub fn not_empty(list: Var) -> Goal {
        fresh(move |head, tail| eq(list, cons(head, tail)))
    }
);

goal!(
    pub fn at_least_two(list: Var) -> Goal {
        fresh(move |a, b, c| eq(list, list!(a, b, . c)))
    }
);
