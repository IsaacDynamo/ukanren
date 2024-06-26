pub mod display;
pub mod list;
pub mod set;
mod test;

use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Deref,
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermType {
    Any,
    Number,
    String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Type(TermType),
    Value(i32), // Todo make generic
    String(String),
    Var(Var, TermType),
    Cons(Rc<Term>, Rc<Term>),
    Null,
}

impl Term {
    pub fn to_vec(&self) -> Option<Vec<Term>> {
        fn inner(term: &Term, mut list: Vec<Term>) -> Option<Vec<Term>> {
            match term {
                Term::Null => Some(list),
                Term::Cons(a, b) => {
                    let a = (*(*a)).clone();
                    list.push(a);
                    inner(b, list)
                }
                _ => None,
            }
        }
        inner(self, Vec::new())
    }
}

impl From<&Term> for Term {
    fn from(t: &Term) -> Self {
        t.clone()
    }
}

impl From<i32> for Term {
    fn from(i: i32) -> Self {
        Self::Value(i)
    }
}

impl From<Var> for Term {
    fn from(var: Var) -> Self {
        Self::Var(var, TermType::Any)
    }
}

impl From<&str> for Term {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Term {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&String> for Term {
    fn from(s: &String) -> Self {
        Self::String(s.clone())
    }
}

pub fn cons(a: impl Into<Term>, b: impl Into<Term>) -> Term {
    Term::Cons(Rc::new(a.into()), Rc::new(b.into()))
}

pub const NULL: Term = Term::Null;
pub const NUM: Term = Term::Type(TermType::Number);
pub const STR: Term = Term::Type(TermType::String);
pub const ANY: Term = Term::Type(TermType::Any);

#[macro_export]
macro_rules! list {
    () => { Term::Null };
    ($a:expr, . $b:expr) => { cons($a, $b) };
    ($head:expr $(, $tail:expr)*  $(, . $rem:expr)? $(,)?) => {
        cons($head, list!( $( $tail ),*  $(, . $rem)? ))
    };
}

#[macro_export]
macro_rules! goal {
    ( $pub:vis fn $name:ident ($($terms:ident : Var ),+ ) -> Goal $goal:block)  => (
        paste::paste!{
            $pub fn $name ( $($terms : impl Into<Term>),+ ) -> Goal {
                $(let [<term_ $terms>]: Term = $terms.into();)+
                fresh(move | $( [<var_ $terms>] ),+ | all([
                    $(eq(&[<term_ $terms>], [<var_ $terms>]),)+
                    (| $($terms),+ | $goal)(
                        $([<var_ $terms>]),+
                    )
                ]))
            }
        }
    )
}

type Mapping = HashMap<Var, Term>;

fn resolve<'a>(term: &'a Term, map: &'a Mapping) -> &'a Term {
    use Term::*;
    match term {
        Var(x, _) => {
            if let Some(q) = map.get(x) {
                match q {
                    Term::Var(y, _) if x == y => q,
                    _ => resolve(q, map)
                }
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

fn unify_type(a: TermType, b: TermType) -> Option<TermType> {
    match (a, b) {
        (a, b) if a == b => Some(a),
        (TermType::Any, a) | (a, TermType::Any) => Some(a),
        _ => None
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
        use Term as T;

        let a_term = resolve(a, &self.map).clone();
        let b_term = resolve(b, &self.map).clone();
        match (a_term, b_term) {
            (T::Var(a, a_type), T::Var(b, b_type)) if a == b => {
                assert_eq!(a_type, b_type);
                Some(())
            },
            (T::Value(a), T::Value(b)) if a == b => Some(()),
            (T::String(a), T::String(b)) if a == b => Some(()),
            (Term::Type(a), Term::Type(b)) if a == b => Some(()),
            (T::Null, T::Null) => Some(()),

            (NUM, T::Value(_)) | (T::Value(_), NUM) => Some(()),
            (STR, T::String(_)) | (T::String(_), STR) => Some(()),
            (ANY, term) | (term, ANY) if !matches!(term, Term::Var(_, _) | Term::Type(_)) => Some(()),

            (T::Var(a, a_type), T::Var(b, b_type)) if a != b => {
                let var_max = max(a, b);
                let var_min = min(a, b);
                if let Some(typ) = unify_type(a_type, b_type) {
                    self.extend(var_max, Term::Var(var_min, typ));
                    if typ != TermType::Any {
                        self.extend(var_min, Term::Var(var_min, typ));
                    }
                    Some(())
                } else {
                    None
                }
            }

            (T::Var(var, TermType::Any), Term::Type(typ)) | (Term::Type(typ), T::Var(var, TermType::Any)) => {
                if typ != TermType::Any {
                    self.extend(var, Term::Var(var, typ));
                }
                Some(())
            }

            (T::Var(_, var_type), Term::Type(term_type))
                | (Term::Type(term_type), T::Var(_, var_type))
                if var_type == term_type =>
            {
                Some(())
            }

            (T::Var(var, TermType::Any), term)
                | (term, T::Var(var, TermType::Any))
                | (T::Var(var, TermType::Number), term@ Term::Value(_))
                | (term @ Term::Value(_), T::Var(var, TermType::Number))
                | (T::Var(var, TermType::String), term @ Term::String(_))
                | (term @ Term::String(_), T::Var(var, TermType::String))
            => {
                self.extend(var, term);
                Some(())
            }
            (T::Cons(a_head, a_tail), T::Cons(b_head, b_tail)) => {
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
    pub depth: u32,
    pub id: Rc<AtomicU32>,
}

impl State {
    pub fn resolve(&self, v: Var) -> Term {
        let term = Term::Var(v, TermType::Any);
        deep_resolve(&term, &self.map)
    }

    fn var(&mut self) -> Var {
        let id = self
            .id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| x.checked_add(1))
            .expect("ID overflow");
        Var(id)
    }
}

pub enum Goal {
    Eq(Term, Term),
    Neq(Term, Term),
    Both(Rc<Goal>, Rc<Goal>),
    Either(Rc<Goal>, Rc<Goal>),
    Fresh(RefCell<FreshInner>),
    Yield(RefCell<YieldInner>),
}

pub enum FreshInner {
    Pending(Rc<dyn Fn(&mut State) -> Goal>),
    Resolved(Rc<Goal>),
}

pub enum YieldInner {
    Pending(Rc<dyn Fn() -> Goal>),
    Resolved(Rc<Goal>),
}

pub fn eq(a: impl Into<Term>, b: impl Into<Term>) -> Goal {
    Goal::Eq(a.into(), b.into())
}

pub fn neq(a: impl Into<Term>, b: impl Into<Term>) -> Goal {
    Goal::Neq(a.into(), b.into())
}

pub fn num(a: impl Into<Term>) -> Goal {
    Goal::Eq(a.into(), NUM)
}

pub fn str(a: impl Into<Term>) -> Goal {
    Goal::Eq(a.into(), STR)
}

pub fn both(a: Goal, b: Goal) -> Goal {
    Goal::Both(Rc::new(a), Rc::new(b))
}

pub fn either(a: Goal, b: Goal) -> Goal {
    Goal::Either(Rc::new(a), Rc::new(b))
}

pub fn fresh<const N: usize>(f: impl Binding<N> + 'static) -> Goal {
    Goal::Fresh(RefCell::new(FreshInner::Pending(Rc::new(move |state| {
        f.bind(state)
    }))))
}

pub fn jield(f: impl Fn() -> Goal + 'static) -> Goal {
    Goal::Yield(RefCell::new(YieldInner::Pending(Rc::new(f))))
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
    pub mature: Vec<State>,
    pub immature: Vec<Box<dyn FnOnce() -> Stream>>,
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

fn mappend(goal: &Rc<Goal>, stream: Stream) -> Stream {
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
            r.and_then(|_| u.unify(&Term::Var(element.0, TermType::Any), &element.1))
        });

        if x.is_some() {
            if u.new.is_empty() {
                // Unification without addition, so constraint is violated
                return false;
            } else {
                new.push(u.new);
            }
        } else {
            // Unification of constraint failed, so the constraint holds.
            // The constraint is no longer need
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
                                constraints,
                                id: state.id.clone(),
                                depth: state.depth,
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
                            constraints,
                            id: state.id.clone(),
                            depth: state.depth,
                        })
                    }
                    None => Stream::new(state.clone()),
                }
            }
            Either(a, b) => append(a.call(state), b.call(state)),
            Both(a, b) => mappend(b, a.call(state)),
            Fresh(inner) => {
                let mut inner = inner.borrow_mut();
                if let FreshInner::Pending(func) = inner.deref() {
                    let mut state = state.clone();
                    let goal = func(&mut state);
                    *inner = FreshInner::Resolved(Rc::new(goal));
                }
                match inner.deref() {
                    FreshInner::Pending(_) => panic!("Should be resolved"),
                    FreshInner::Resolved(goal) => goal.call(state),
                }
            }
            Yield(inner) => {
                let mut inner = inner.borrow_mut();

                if let YieldInner::Pending(func) = inner.deref() {
                    let goal = func();
                    *inner = YieldInner::Resolved(Rc::new(goal));
                }

                match inner.deref() {
                    YieldInner::Pending(_) => panic!("Should be resolved"),
                    YieldInner::Resolved(goal) => {
                        let goal = goal.clone();
                        let mut state = state.clone();
                        state.depth += 1;

                        Stream {
                            mature: Vec::new(),
                            immature: vec![Box::new(move || goal.call(&state))],
                        }
                    }
                }
            }
        }
    }
}

pub trait Binding<const N: usize> {
    fn bind(&self, state: &mut State) -> Goal;
}

impl<T: Fn() -> Goal> Binding<0> for T {
    fn bind(&self, _: &mut State) -> Goal {
        self()
    }
}

impl<T: Fn(Var) -> Goal> Binding<1> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let x = state.var();
        self(x)
    }
}

impl<T: Fn(Var, Var) -> Goal> Binding<2> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let x = state.var();
        let y = state.var();
        self(x, y)
    }
}

impl<T: Fn(Var, Var, Var) -> Goal> Binding<3> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let x = state.var();
        let y = state.var();
        let z = state.var();
        self(x, y, z)
    }
}

impl<T: Fn(Var, Var, Var, Var) -> Goal> Binding<4> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let v1 = state.var();
        let v2 = state.var();
        let v3 = state.var();
        let v4 = state.var();
        self(v1, v2, v3, v4)
    }
}

impl<T: Fn(Var, Var, Var, Var, Var) -> Goal> Binding<5> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let v1 = state.var();
        let v2 = state.var();
        let v3 = state.var();
        let v4 = state.var();
        let v5 = state.var();
        self(v1, v2, v3, v4, v5)
    }
}

impl<T: Fn(Var, Var, Var, Var, Var, Var) -> Goal> Binding<6> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let v1 = state.var();
        let v2 = state.var();
        let v3 = state.var();
        let v4 = state.var();
        let v5 = state.var();
        let v6 = state.var();
        self(v1, v2, v3, v4, v5, v6)
    }
}

impl<T: Fn(Var, Var, Var, Var, Var, Var, Var) -> Goal> Binding<7> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let v1 = state.var();
        let v2 = state.var();
        let v3 = state.var();
        let v4 = state.var();
        let v5 = state.var();
        let v6 = state.var();
        let v7 = state.var();
        self(v1, v2, v3, v4, v5, v6, v7)
    }
}

impl<T: Fn(Var, Var, Var, Var, Var, Var, Var, Var) -> Goal> Binding<8> for T {
    fn bind(&self, state: &mut State) -> Goal {
        let v1 = state.var();
        let v2 = state.var();
        let v3 = state.var();
        let v4 = state.var();
        let v5 = state.var();
        let v6 = state.var();
        let v7 = state.var();
        let v8 = state.var();
        self(v1, v2, v3, v4, v5, v6, v7, v8)
    }
}

pub struct Query<const N: usize> {
    pub goal: Goal,
    pub id: Rc<AtomicU32>,
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

fn mininal_contraints_add(
    minimal_constraints: &mut Vec<HashSet<(Var, Term)>>,
    new_constraint: HashSet<(Var, Term)>,
) {
    assert!(!new_constraint.is_empty());

    let mut fallthrough = false;
    minimal_constraints.retain(|constraint| {
        if fallthrough {
            return true;
        }

        use set::Relation::*;
        match set::relation(&new_constraint, constraint) {
            Subset => false,
            Equal | Superset => {
                fallthrough = true;
                true
            }
            Joint | Disjoint => true,
        }
    });

    if !fallthrough {
        minimal_constraints.push(new_constraint);
    }
}

pub fn purify<const N: usize>(state: &State) -> Constraints {
    // Find all reachable variables
    let mut reachable_vars = HashSet::new();
    for v in 0..N {
        /// Insert the Vars of a Term into a set
        fn insert(set: &mut HashSet<Var>, term: &Term) {
            match term {
                Term::Cons(a, b) => {
                    insert(set, a);
                    insert(set, b);
                }
                Term::Var(v, _) => _ = set.insert(*v),
                _ => (),
            }
        }

        let term = state.resolve(Var::from_usize(v));
        insert(&mut reachable_vars, &term);
    }

    // Initial constraints, only keep constraint terms with constants or reachable variables
    let constraints = Vec::from_iter(
        state
            .constraints
            .iter()
            .map(|vec| {
                HashSet::from_iter(
                    vec.iter()
                        .map(|(var, term)| (*var, deep_resolve(term, &state.map)))
                        .filter(|(var, term)| {
                            fn only_reachable(term: &Term, set: &HashSet<Var>) -> bool {
                                match term {
                                    Term::Cons(a, b) => {
                                        only_reachable(a, set) && only_reachable(b, set)
                                    }
                                    Term::Var(v, _) => set.contains(v),
                                    _ => true,
                                }
                            }

                            reachable_vars.contains(var) && only_reachable(term, &reachable_vars)
                        }),
                )
            })
            .filter(|set| !set.is_empty()),
    );

    // Start with empty set
    let mut minimal_constraints = Vec::<HashSet<(Var, Term)>>::new();

    // Process constraints
    for new_constraint in constraints {
        mininal_contraints_add(&mut minimal_constraints, new_constraint);
    }

    // Convert inner HashSet to a Vec
    minimal_constraints
        .into_iter()
        .map(|s| Vec::from_iter(s.into_iter()))
        .collect()
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
        id: state.id.clone(),
        stream,
        mature_iter: Vec::new().into_iter(),
        immature_iter: Vec::new().into_iter(),
    }
}

#[derive(Debug)]
pub struct StateN<const N: usize> {
    state: State,
}

impl<const N: usize> StateN<N> {
    pub fn reify(&self) -> [Term; N] {
        reify::<N>(&self.state)
    }
}

pub fn run_all<const N: usize>(f: impl Binding<N>) -> Vec<StateN<N>> {
    let mut q = query(f);
    q.iter().collect()
}

pub fn run<const N: usize>(n: usize, f: impl Binding<N>) -> Vec<StateN<N>> {
    let mut q = query(f);
    q.iter().take(n).collect()
}
