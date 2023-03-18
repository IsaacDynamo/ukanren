mod display;
mod test;

use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

// TODO:
// - neq()
// - impl Goal + 'recursive' types
// - Prefer non-yield goals in eval of Both
// - Add bool and str
// - Use term arguments in custom goals
// - Remove StreamIter

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

pub fn cons(a: impl Into<Term>, b: impl Into<Term>) -> Term {
    Term::Cons(Rc::new(a.into()), Rc::new(b.into()))
}

pub const NULL: Term = Term::Null;

type Mapping = HashMap<u32, Term>;

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

fn deep_resolve(term: &Term, map: &Mapping) -> Term {
    let term = resolve(term, map);

    match term {
        Term::Cons(a, b) => cons(deep_resolve(a, map), deep_resolve(b, map)),
        _ => term.clone(),
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

#[derive(Clone)]
pub enum Goal {
    Eq(Term, Term),
    Both(Rc<Goal>, Rc<Goal>),
    Either(Rc<Goal>, Rc<Goal>),
    Fresh(Rc<dyn Fn(&mut State) -> Goal>, RefCell<Option<Box<Goal>>>),
    Yield(Rc<dyn Fn() -> Goal>, Rc<RefCell<Option<Goal>>>),
}

pub fn eq(a: impl Into<Term>, b: impl Into<Term>) -> Goal {
    Goal::Eq(a.into(), b.into())
}

pub fn both(a: Goal, b: Goal) -> Goal {
    Goal::Both(Rc::new(a), Rc::new(b))
}

pub fn either(a: Goal, b: Goal) -> Goal {
    Goal::Either(Rc::new(a), Rc::new(b))
}

pub fn fresh<const N: usize>(f: impl Binding<N> + Copy + 'static) -> Goal {
    Goal::Fresh(Rc::new(move |state| f.bind(state)), RefCell::new(None))
}

pub fn jield(f: impl Fn() -> Goal + 'static) -> Goal {
    Goal::Yield(Rc::new(f), Rc::new(RefCell::new(None)))
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

#[derive(Default)]
pub struct Stream {
    mature: Vec<State>,
    immature: Option<Box<dyn FnOnce() -> Stream>>,
}

impl Stream {
    pub fn is_empty(&self) -> bool {
        self.mature.is_empty() && self.immature.is_none()
    }

    pub fn pull(&mut self) {
        let immature = self.immature.take();
        if let Some(cont) = immature {
            let s = cont();
            self.mature.extend(s.mature);
            self.immature = s.immature;
        }
    }
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
            Both(a, b) => mappend(b, a.call(state)),
            Fresh(f, node) => {
                let mut s = state.clone();
                let goal = f(&mut s);
                node.replace(Some(Box::new(goal)));
                node.borrow().as_ref().map(|f| f.call(&s)).unwrap()
            }
            Yield(cont, node) => {
                let state = state.clone();
                let cont = cont.clone();
                let node = node.clone();

                Stream {
                    mature: Vec::new(),
                    immature: Some(Box::new(move || {
                        let goal = cont();
                        node.replace(Some(goal));
                        node.borrow().as_ref().map(|g| g.call(&state)).unwrap()
                    })),
                }
            }
        }
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

pub struct StreamIter {
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

pub trait Binding<const N: usize> {
    fn bind(self, state: &mut State) -> Goal;
}

impl Binding<0> for Goal {
    fn bind(self, _: &mut State) -> Goal {
        self
    }
}

impl<T: Fn(Var) -> Goal> Binding<1> for T {
    fn bind(self, state: &mut State) -> Goal {
        let x = state.var();
        self(x)
    }
}

impl<T: Fn(Var, Var) -> Goal> Binding<2> for T {
    fn bind(self, state: &mut State) -> Goal {
        let x = state.var();
        let y = state.var();
        self(x, y)
    }
}

impl<T: Fn(Var, Var, Var) -> Goal> Binding<3> for T {
    fn bind(self, state: &mut State) -> Goal {
        let x = state.var();
        let y = state.var();
        let z = state.var();
        self(x, y, z)
    }
}

pub struct Query<const N: usize> {
    pub goal: Goal,
    pub stream: Stream,
}

pub fn query<const N: usize>(f: impl Binding<N>) -> Query<N> {
    let mut state = State::default();
    let goal = f.bind(&mut state);
    let stream = goal.call(&state);
    Query { goal, stream }
}

pub fn run_all<const N: usize>(f: impl Binding<N>) -> Vec<Vec<Term>> {
    let q = query(f);
    q.stream
        .into_iter()
        .map(|s| {
            (0..N)
                .map(|v| {
                    s.resolve(Var {
                        id: v.try_into().unwrap(),
                    })
                })
                .collect::<Vec<Term>>()
        })
        .collect()
}

pub fn run<const N: usize>(n: usize, f: impl Binding<N>) -> Vec<Vec<Term>> {
    let q = query(f);
    q.stream
        .into_iter()
        .map(|s| {
            (0..N)
                .map(|v| {
                    s.resolve(Var {
                        id: v.try_into().unwrap(),
                    })
                })
                .collect::<Vec<Term>>()
        })
        .take(n)
        .collect()
}
