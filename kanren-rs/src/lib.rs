mod display;
mod test;

use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

// TODO:
// - impl Goal + 'recursive' types
// - Prefer non-yield goals in eval of Both
// - Add bool and str
// - Use term arguments in custom goals
// - Rc state
// - Stacked map in Unify

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Var(u32);

impl Var {
    fn from_usize(v: usize) -> Self {
        Self(v.try_into().expect("Conversion failed"))
    }
}

#[derive(Debug, Clone)]
pub enum Term {
    Value(i32), // Todo make generic
    Var(Var),
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
        Self::Var(var)
    }
}

pub fn cons(a: impl Into<Term>, b: impl Into<Term>) -> Term {
    Term::Cons(Rc::new(a.into()), Rc::new(b.into()))
}

pub const NULL: Term = Term::Null;

type Mapping = HashMap<Var, Term>;

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

#[derive(Debug)]
struct Unify {
    map: Mapping,
    new: Vec<(Var, Term)>,
}

impl Unify {
    fn new(map: Mapping) -> Self {
        Self {
            map,
            new: Vec::new(),
        }
    }

    fn extend(&mut self, var: Var, term: Term) {
        self.new.push((var, term.clone()));
        self.map.insert(var, term);
    }

    fn unify(&mut self, a: &Term, b: &Term) -> Option<()> {
        use Term::*;

        let a = resolve(a, &self.map).clone();
        let b = resolve(b, &self.map).clone();
        match (a, b) {
            (Var(a), Var(b)) if a == b => Some(()),
            (Value(a), Value(b)) if a == b => Some(()),
            (Null, Null) => Some(()),
            (Var(a), Var(b)) if a != b => {
                let var = max(a, b);
                let term = Term::Var(min(a, b));
                self.extend(var, term);
                Some(())
            }
            (Var(var), term) | (term, Var(var)) => {
                self.extend(var, term);
                Some(())
            }
            (Cons(a_head, a_tail), Cons(b_head, b_tail)) => {
                self.unify(&a_head, &b_head)?;
                self.unify(&a_tail, &b_tail)
            }
            _ => None,
        }
    }
}

type Constraint = Vec<(Var, Term)>;
type Constraints = Vec<Constraint>;

#[derive(Default, Debug, Clone)]
pub struct State {
    map: Mapping,
    constraints: Constraints,
    id: u32,
}

impl State {
    fn resolve(&self, v: Var) -> Term {
        let term = Term::Var(v);
        deep_resolve(&term, &self.map)
    }

    fn var(&mut self) -> Var {
        let id = self.id;
        assert_ne!(id, u32::MAX, "Overflow");
        self.id += 1;
        Var(id)
    }
}

#[derive(Clone)]
pub enum Goal {
    Eq(Term, Term),
    Neq(Term, Term),
    Both(Rc<Goal>, Rc<Goal>),
    Either(Rc<Goal>, Rc<Goal>),
    Fresh(Rc<dyn Fn(&mut State) -> Goal>, RefCell<Option<Box<Goal>>>),
    Yield(Rc<dyn Fn() -> Goal>, Rc<RefCell<Option<Goal>>>),
}

pub fn eq(a: impl Into<Term>, b: impl Into<Term>) -> Goal {
    Goal::Eq(a.into(), b.into())
}

pub fn neq(a: impl Into<Term>, b: impl Into<Term>) -> Goal {
    Goal::Neq(a.into(), b.into())
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
    pub fn new(state: State) -> Self {
        Stream {
            mature: vec![state],
            immature: Vec::new(),
        }
    }

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

fn verify(map: &Mapping, constraints: &Constraints, new: &mut Constraints) -> bool {
    for elements in constraints {
        let mut u = Unify::new(map.clone());
        let x = elements.iter().fold(Some(()), |r, element| {
            r.and_then(|_| u.unify(&Term::Var(element.0), &element.1))
        });

        if x.is_some() {
            if u.new.is_empty() {
                // Unification without addition, so constraint is violated
                return false;
            } else {
                new.push(u.new);
            }
        }
    }

    true
}

impl Goal {
    fn call(&self, state: &State) -> Stream {
        use Goal::*;

        match self {
            Eq(a, b) => {
                let mut u = Unify::new(state.map.clone());
                match u.unify(a, b) {
                    Some(_) if u.new.is_empty() => Stream::new(state.clone()),
                    Some(_) => {
                        let mut constraints = Vec::new();
                        if verify(&u.map, &state.constraints, &mut constraints) {
                            Stream::new(State {
                                map: u.map,
                                id: state.id,
                                constraints,
                            })
                        } else {
                            Stream::default()
                        }
                    }
                    None => Stream::default(),
                }
            }
            Neq(a, b) => {
                let mut u = Unify::new(state.map.clone());
                match u.unify(a, b) {
                    Some(_) if u.new.is_empty() => Stream::default(),
                    Some(_) => {
                        let mut constraints = state.constraints.clone();
                        constraints.push(u.new);
                        Stream::new(State {
                            map: state.map.clone(),
                            id: state.id,
                            constraints,
                        })
                    }
                    None => Stream::new(state.clone()),
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

impl<const N: usize> Query<N> {
    fn pull(&mut self) -> Option<Option<State>> {
        let state = self.mature_iter.next();
        if state.is_some() {
            return Some(state);
        } else if !self.stream.mature.is_empty() {
            let mature = std::mem::take(&mut self.stream.mature);
            self.mature_iter = mature.into_iter();
        } else if let Some(cont) = self.immature_iter.next() {
            self.stream.append(cont());
        } else if !self.stream.immature.is_empty() {
            let immature = std::mem::take(&mut self.stream.immature);
            self.immature_iter = immature.into_iter();
        } else {
            return None;
        }
        Some(None)
    }
}

impl<const N: usize> Iterator for Query<N> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let n = self.pull()?;
            if n.is_some() {
                return n;
            }
        }
    }
}

pub struct QueryIter<'a, const N: usize>(&'a mut Query<N>);

impl<'a, const N: usize> Iterator for QueryIter<'a, N> {
    type Item = StateN<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|state| StateN { state })
    }
}

pub fn reify<const N: usize>(state: &State) -> [Term; N] {
    std::array::from_fn(|v| state.resolve(Var::from_usize(v)))
}

pub fn purify<const N: usize>(state: &State) -> Constraints {
    fn add_vars_to_set(term: &Term, set: &mut HashSet<Var>) {
        match term {
            Term::Cons(a, b) => {
                add_vars_to_set(a, set);
                add_vars_to_set(b, set);
            }
            Term::Var(v) => _ = set.insert(*v),
            _ => (),
        }
    }

    let mut reachable_vars = HashSet::new();
    for v in 0..N {
        let t = state.resolve(Var::from_usize(v));
        add_vars_to_set(&t, &mut reachable_vars);
    }

    fn only_unreachable(term: &Term, set: &HashSet<Var>) -> bool {
        match term {
            Term::Cons(a, b) => only_unreachable(a, set) && only_unreachable(b, set),
            Term::Var(v) => !set.contains(v),
            _ => false,
        }
    }

    state
        .constraints
        .iter()
        .filter_map(|constraint| {
            let minimal = constraint
                .iter()
                .filter(|(var, term)| {
                    reachable_vars.contains(var) && !only_unreachable(term, &reachable_vars)
                })
                .cloned()
                .collect::<Vec<(Var, Term)>>();

            if !minimal.is_empty() {
                Some(minimal)
            } else {
                None
            }
        })
        .collect::<Constraints>()
}

impl<const N: usize> Query<N> {
    fn iter(&mut self) -> QueryIter<N> {
        QueryIter(self)
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

#[derive(Debug)]
pub struct StateN<const N: usize> {
    state: State,
}

pub fn run_all<const N: usize>(f: impl Binding<N>) -> Vec<StateN<N>> {
    let mut q = query(f);
    q.iter().collect()
}

pub fn run<const N: usize>(n: usize, f: impl Binding<N>) -> Vec<StateN<N>> {
    let mut q = query(f);
    q.iter().take(n).collect()
}
