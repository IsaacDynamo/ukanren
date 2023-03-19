mod display;
mod test;

use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

// TODO:
// - neq()
// - impl Goal + 'recursive' types
// - Prefer non-yield goals in eval of Both
// - Add bool and str
// - Use term arguments in custom goals

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
    immature: Vec<Box<dyn FnOnce() -> Stream>>,
}

impl Stream {
    pub fn append(&mut self, s: Stream) {
        fn combine<T>(a: &mut Vec<T>, mut b: Vec<T>) {
            if a.is_empty() {
                std::mem::swap(a, &mut b)
            } else if !b.is_empty() {
                a.append(&mut b);
            }
        }

        combine(&mut self.mature, s.mature);
        combine(&mut self.immature, s.immature);
    }
}

fn append(a: Stream, b: Stream) -> Stream {
    let mut a = a;
    a.append(b);
    a
}

fn mappend(goal: &Goal, stream: Stream) -> Stream {
    let mut result = stream
        .mature
        .into_iter()
        .map(|state| goal.call(&state))
        .fold(Stream::default(), append);

    for cont in stream.immature {
        let goal = goal.clone();
        result
            .immature
            .push(Box::new(move || mappend(&goal, cont())))
    }

    result
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
                        immature: Vec::new(),
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
                    immature: vec![Box::new(move || {
                        let goal = cont();
                        node.replace(Some(goal));
                        node.borrow().as_ref().map(|g| g.call(&state)).unwrap()
                    })],
                }
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
    pub mature_iter: std::vec::IntoIter<State>,
    pub immature_iter: std::vec::IntoIter<Box<dyn FnOnce() -> Stream>>,
}

impl<const N: usize> Iterator for Query<N> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let state = self.mature_iter.next();
            if state.is_some() {
                return state;
            } else if !self.stream.mature.is_empty() {
                let mature = std::mem::take(&mut self.stream.mature);
                self.mature_iter = mature.into_iter();
            } else {
                if let Some(cont) = self.immature_iter.next() {
                    self.stream.append(cont());
                } else if !self.stream.immature.is_empty() {
                    let immature = std::mem::take(&mut self.stream.immature);
                    self.immature_iter = immature.into_iter();
                } else {
                    return None;
                }
            }
        }
    }
}

impl<const N: usize> Query<N> {
    fn resolve(&mut self) -> impl Iterator<Item = [Term; N]> + '_ {
        self.map(|s| {
            std::array::from_fn(|v| {
                s.resolve(Var {
                    id: v.try_into().unwrap(),
                })
            })
        })
    }
}

pub fn query<const N: usize>(f: impl Binding<N>) -> Query<N> {
    let mut state = State::default();
    let goal = f.bind(&mut state);
    let stream = goal.call(&state);
    Query {
        goal,
        stream,
        mature_iter: Vec::new().into_iter(),
        immature_iter: Vec::new().into_iter(),
    }
}

pub fn run_all<const N: usize>(f: impl Binding<N>) -> Vec<[Term; N]> {
    let mut q = query(f);
    q.resolve().collect()
}

pub fn run<const N: usize>(n: usize, f: impl Binding<N>) -> Vec<[Term; N]> {
    let mut q = query(f);
    q.resolve().take(n).collect()
}
