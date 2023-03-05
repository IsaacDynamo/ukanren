use std::{collections::HashMap, fmt::Debug, rc::Rc};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Var {
    id: u32,
}

#[derive(Debug, Clone)]
pub enum Term {
    Value(i32), // Todo make generic
    Var(u32),
    Cons(Rc<Term>, Rc<Term>),
    Null,
}

type Mapping = HashMap<u32, Term>;

#[derive(Clone)]
pub enum Goal {
    Eq(Term, Term),
    Both(Rc<Goal>, Rc<Goal>),
    Either(Rc<Goal>, Rc<Goal>),
    Fresh(Rc<dyn Fn(&mut State) -> Goal>),
    Yield(Rc<dyn Fn() -> Goal>),
}

impl From<i32> for Term {
    fn from(i: i32) -> Self {
        Self::Value(i)
    }
}

impl From<Var> for Term {
    fn from(var: Var) -> Self {
        Self::Var(var.id)
    }
}

fn resolve<'a>(term: &'a Term, map: &'a Mapping) -> &'a Term {
    use Term::*;
    match term {
        Var(x) => {
            if let Some(q) = map.get(x) {
                resolve(q, map)
            } else {
                term
            }
        }
        _ => term,
    }
}

fn extend(key: &u32, value: &Term, map: &Mapping) -> Mapping {
    let mut m = map.clone();
    m.insert(*key, value.clone());
    m
}

fn unify(a: &Term, b: &Term, map: &Mapping) -> Option<Mapping> {
    use Term::*;

    let a = resolve(a, map);
    let b = resolve(b, map);

    match (a, b) {
        (Var(a), Var(b)) if *a == *b => Some(map.clone()),
        (Value(a), Value(b)) if *a == *b => Some(map.clone()),
        (Null, Null) => Some(map.clone()),
        (Var(a), b) => Some(extend(a, b, map)),
        (a, Var(b)) => Some(extend(b, a, map)),
        (Cons(a_head, a_tail), Cons(b_head, b_tail)) => {
            let map = unify(a_head, b_head, map)?;
            unify(a_tail, b_tail, &map)
        }
        _ => None,
    }
}

pub const NULL: Term = Term::Null;

pub fn eq(a: impl Into<Term>, b: impl Into<Term>) -> Goal {
    Goal::Eq(a.into(), b.into())
}

pub fn cons(a: impl Into<Term>, b: impl Into<Term>) -> Term {
    Term::Cons(Rc::new(a.into()), Rc::new(b.into()))
}

pub fn both(a: Goal, b: Goal) -> Goal {
    Goal::Both(Rc::new(a), Rc::new(b))
}

pub fn either(a: Goal, b: Goal) -> Goal {
    Goal::Either(Rc::new(a), Rc::new(b))
}

fn append(mut a: Stream, b: Stream) -> Stream {
    if a.is_empty() {
        b
    } else if let Some(cont) = a.immature {
        Stream {
            mature: a.mature,
            immature: Some(Box::new(move || append(b, cont()))),
        }
    } else {
        a.mature.extend(b.mature);
        Stream {
            mature: a.mature,
            immature: b.immature,
        }
    }
}

fn mappend(goal: &Goal, stream: Stream) -> Stream {
    if stream.is_empty() {
        stream
    } else if !stream.mature.is_empty() {
        let s = Stream {
            mature: Vec::from_iter(stream.mature[1..].iter().cloned()),
            immature: stream.immature,
        };
        append(goal.call(&stream.mature[0]), mappend(goal, s))
    } else if let Some(cont) = stream.immature {
        let goal = goal.clone();
        Stream {
            mature: Vec::new(),
            immature: Some(Box::new(move || mappend(&goal, cont()))),
        }
    } else {
        unreachable!()
    }

    // match stream.immature {
    //     None if stream.mature.is_empty() => stream,
    //     Some(cont) if stream.mature.is_empty() => {

    //     },
    // }
}

#[derive(Default, Debug, Clone)]
pub struct State {
    map: Mapping,
    id: u32,
}

impl State {
    fn resolve(&self, v: Var) -> Term {
        let term = Term::Var(v.id);
        deep_resolve(&term, &self.map)
    }

    fn var(&mut self) -> Var {
        let id = self.id;
        assert_ne!(id, u32::MAX, "Overflow");
        self.id += 1;
        Var { id }
    }
}

#[derive(Default)]
struct Stream {
    mature: Vec<State>,
    immature: Option<Box<dyn FnOnce() -> Stream>>,
}

impl Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("mature", &self.mature)
            .field("immature", &self.immature.as_ref().map(|_| ()))
            .finish()
    }
}

impl Stream {
    fn is_empty(&self) -> bool {
        self.mature.is_empty() && self.immature.is_none()
    }
}

impl IntoIterator for Stream {
    type Item = State;

    type IntoIter = StreamIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            mature: self.mature.into_iter(),
            immature: self.immature,
        }
    }
}

struct StreamIter {
    mature: std::vec::IntoIter<State>,
    immature: Option<Box<dyn FnOnce() -> Stream>>,
}

impl Iterator for StreamIter {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.mature.next();
        if r.is_some() {
            r
        } else {
            let immature = self.immature.take();
            if let Some(cont) = immature {
                *self = cont().into_iter();
                self.next()
            } else {
                None
            }
        }
    }
}

impl Goal {
    fn call(&self, state: &State) -> Stream {
        use Goal::*;

        match self {
            Eq(a, b) => {
                if let Some(mapping) = unify(a, b, &state.map) {
                    Stream {
                        mature: vec![State {
                            map: mapping,
                            id: state.id,
                        }],
                        immature: None,
                    }
                } else {
                    Stream::default()
                }
            }
            Either(a, b) => append(a.call(state), b.call(state)),
            Both(a, b) => mappend(a, b.call(state)),
            Fresh(f) => {
                let mut s = state.clone();
                let goal = f(&mut s);
                goal.call(&s)
            }
            Yield(cont) => {
                let state = state.clone();
                let cont = cont.clone();
                Stream {
                    mature: Vec::new(),
                    immature: Some(Box::new(move || cont().call(&state))),
                }
            }
        }
    }
}

pub fn all(v: impl IntoIterator<Item = Goal>) -> Goal {
    fn inner(mut iter: impl Iterator<Item = Goal>) -> Option<Goal> {
        let a = iter.next()?;
        match inner(iter) {
            Some(b) => Some(both(a, b)),
            None => Some(a),
        }
    }

    inner(v.into_iter()).unwrap()
}

pub fn any(v: impl IntoIterator<Item = Goal>) -> Goal {
    fn inner(mut iter: impl Iterator<Item = Goal>) -> Option<Goal> {
        let a = iter.next()?;
        match inner(iter) {
            Some(b) => Some(either(a, b)),
            None => Some(a),
        }
    }

    inner(v.into_iter()).unwrap()
}

pub fn cond<T, R>(table: T) -> Goal
where
    T: IntoIterator<Item = R>,
    R: IntoIterator<Item = Goal>,
{
    any(table.into_iter().map(|goals| all(goals)))
}

fn deep_resolve(term: &Term, map: &Mapping) -> Term {
    let term = resolve(term, map);

    match term {
        Term::Cons(a, b) => cons(deep_resolve(a, map), deep_resolve(b, map)),
        _ => term.clone(),
    }
}

pub trait Binding<T> {
    fn bind(self, state: &mut State) -> (Vec<Var>, Goal);
}

impl Binding<()> for Goal {
    fn bind(self, _: &mut State) -> (Vec<Var>, Goal) {
        (vec![], self)
    }
}

impl<T: Fn(Var) -> Goal> Binding<(Var,)> for T {
    fn bind(self, state: &mut State) -> (Vec<Var>, Goal) {
        let x = state.var();
        (vec![x], self(x))
    }
}

impl<T: Fn(Var, Var) -> Goal> Binding<(Var, Var)> for T {
    fn bind(self, state: &mut State) -> (Vec<Var>, Goal) {
        let x = state.var();
        let y = state.var();
        (vec![x, y], self(x, y))
    }
}

impl<T: Fn(Var, Var, Var) -> Goal> Binding<(Var, Var, Var)> for T {
    fn bind(self, state: &mut State) -> (Vec<Var>, Goal) {
        let x = state.var();
        let y = state.var();
        let z = state.var();
        (vec![x, y, z], self(x, y, z))
    }
}

pub struct Query {
    vars: Vec<Var>,
    stream: Stream,
}

pub fn query<T>(f: impl Binding<T>) -> Query {
    let mut state = State::default();
    let (vars, goal) = f.bind(&mut state);
    let stream = goal.call(&state);
    Query { vars, stream }
}

pub fn run<T>(f: impl Binding<T>) -> Vec<Vec<Term>> {
    let q = query(f);
    q.stream
        .into_iter()
        .map(|s| q.vars.iter().map(|v| s.resolve(*v)).collect::<Vec<Term>>())
        .collect()
}

pub fn runx<T>(n: usize, f: impl Binding<T>) -> Vec<Vec<Term>> {
    let q = query(f);
    q.stream
        .into_iter()
        .map(|s| q.vars.iter().map(|v| s.resolve(*v)).collect::<Vec<Term>>())
        .take(n)
        .collect()
}

pub fn fresh<T>(f: impl Binding<T> + Copy + 'static) -> Goal {
    Goal::Fresh(Rc::new(move |state| f.bind(state).1))
}

pub fn jield(f: impl Fn() -> Goal + 'static) -> Goal {
    Goal::Yield(Rc::new(f))
}

#[cfg(test)]
mod tests {
    use super::*;

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

        println!(
            "{:?}",
            runx(10, move |x, y| fresh(move |r| both(
                eq(r, l()),
                concat(x, y, r)
            )))
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