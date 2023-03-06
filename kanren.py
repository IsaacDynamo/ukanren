from dataclasses import dataclass
from typing import Any, Callable, Dict, List, Union
from functools import reduce
from inspect import getfullargspec

@dataclass(eq=True, frozen=True)
class Var:
    i: int

@dataclass(eq=True, frozen=True)
class Cons:
    x: Any
    y: Any

    def list(l: List) -> 'Cons':
        if len(l) == 0:
            return None
        else:
            return Cons(l[0], Cons.list(l[1:]))

    def __repr__(self) -> str:
        s = '('
        c = self
        while True:
            s += f'{repr(c.x)}'
            if c.y is None:
                break
            elif is_cons(c.y):
                c = c.y
                s += ' '
            else:
                s += f'{repr(c.y)}'
                break
        s += ')'
        return s

def is_var(x):  return type(x) is Var
def is_cons(x): return type(x) is Cons

def head(list: Union[Cons, None]): return None if list is None else list.x
def tail(list: Union[Cons, None]): return None if list is None else list.y

def extend(m, key, val):
    m = m.copy()
    m[key] = val
    return m

Term = Union[Var, Cons, str, int]
Subst = Dict[Var, Term]

#walk
def resolve(term: Term, subst_map: Subst) -> Term:
    if is_var(term) and term in subst_map:
        return resolve(subst_map[term], subst_map)
    else:
        return term

def unify(term1: Term, term2: Term, subst_map: Subst) -> Union[Subst, None]:
    if subst_map is None:
        return None

    term1 = resolve(term1, subst_map)
    term2 = resolve(term2, subst_map)

    if is_var(term1) and is_var(term2) and term1 == term2:
        return subst_map
    elif is_var(term1):
        return extend(subst_map, term1, term2)
    elif is_var(term2):
        return extend(subst_map, term2, term1)
    elif is_cons(term1) and is_cons(term2):
        s = unify(head(term1), head(term2), subst_map)
        return unify(tail(term1), tail(term2), s)
    elif term1 == term2:
        return subst_map
    else:
        return None

@dataclass
class State:
    subst_map: Subst
    i: int

Goal = Callable[[State], 'Goal']
ImmatureStream = Callable[[], Goal]
Stream = Union[None, Cons, ImmatureStream]

#unit
def new_stream(state: State) -> Stream: return Cons(state, None)

def call_goal(goal: Goal):
    return goal(State({}, 0))

def eq(term1, term2):
    def lam(state: State):
        s = unify(term1, term2, state.subst_map)
        if s is not None:
            return new_stream(State(s, state.i))
        else:
            return None
    return lam

def call_fresh(f) -> Goal:
    def lam(state: State):
        n = len(getfullargspec(f).args)
        vars = [Var(state.i + i) for i in range(n)]
        return f(*vars)(State(state.subst_map, state.i + n))
    return lam

# mplus
def append(stream1: Stream, stream2: Stream) -> Stream:
    if stream1 is None:
        return stream2
    elif callable(stream1):
        return lambda: append(stream2, stream1())
    else:
        return Cons(head(stream1), append(tail(stream1), stream2))

# bind
def mappend(goal: Goal, stream: Stream) -> Stream:
    if stream is None:
        return None
    elif callable(stream):
        return lambda: mappend(goal, stream())
    else:
        return append(goal(head(stream)), mappend(goal, tail(stream)))

# disj
def either(goal1: Goal, goal2: Goal) -> Goal:
    return lambda state: append(goal1(state), goal2(state))

# conj
def both(goal1: Goal, goal2: Goal) -> Goal:
    return lambda state: mappend(goal2, goal1(state))

# zzz
def _yield(f, *args) -> Goal:
    return lambda state: lambda: f(*args)(state)

def pull(stream):
    if callable(stream):
        return pull(stream())
    else:
        return stream

def take(n, stream):
    if n == 0:
        return []
    else:
        stream = pull(stream)
        if stream is None:
            return []
        else:
            ret = [head(stream)]
            ret.extend(take(n-1, tail(stream)))
            return ret

# disj+
def any(goals):
    return reduce(either, goals)

# conj+
def all(goals):
    return reduce(both, goals)

# conde
def cond(goals_s):
    return any(map(all, goals_s))

# walk*
def deep_resolve(term, subst_map):
    term = resolve(term, subst_map)
    if is_cons(term):
        return Cons(deep_resolve(head(term), subst_map), deep_resolve(tail(term), subst_map))
    else:
        return term

def reify (state: State):
    return deep_resolve(Var(0), state.subst_map)

def run(n, f):
    assert len(getfullargspec(f).args) == 1, "Multi-variable queries don't work"
    return list(map(reify, take(n, call_goal(call_fresh(f)))))


print("Equality, either and both")
print("q == 23 or  q == 24 -> q in", run(10, lambda q: either(eq(q, 23), eq(q, 24))))
print("q == 23 and q == 24 -> q in", run(10, lambda q: both(eq(q, 23), eq(q, 24))))
print("q == 23 and q == 23 -> q in", run(10, lambda q: both(eq(q, 23), eq(q, 23))))
print()


def fives(x):
    return either(eq(x, 5), _yield(fives, x))

def sixes(x):
    return either(eq(x, 6), _yield(sixes, x))

print("Breadth-first search, without stack overflow")
print("fives(x): x == 5 or fives(x)")
print("sixes(x): x == 6 or sixes(x)")
print("fives(q) or sixes(q) -> q in", run(10, lambda q: either(fives(q), sixes(q))))
print()


# appendo
def concat(l, r, out):
    return _yield(call_fresh, (lambda a, d, res: cond([
        [eq(None, l), eq(r, out)],
        [eq(Cons(a, d), l), eq(Cons(a, res), out), _yield(concat, d, r, res)]
        ])))

print("concat(x,y) == [1, 2, 3, 4] -> (x, y) in")
print(run(10, lambda q: call_fresh(lambda x, y: both(
    eq(q, Cons.list([x, y])),
    concat(x, y, Cons.list([1, 2, 3, 4]))))))
print()


def parent(x, y):
    return any([
        eq(Cons.list([x, y]), Cons.list(["Homer", "Bart"])),
        eq(Cons.list([x, y]), Cons.list(["Marge", "Bart"])),
        eq(Cons.list([x, y]), Cons.list(["Homer", "Lisa"])),
        eq(Cons.list([x, y]), Cons.list(["Marge", "Lisa"])),
        eq(Cons.list([x, y]), Cons.list(["Abe",   "Homer"])),
        eq(Cons.list([x, y]), Cons.list(["Jackie", "Marge"])),
    ])

print("Parents of Bart   ->", run(10, lambda q: parent(q, "Bart")))
print("Children of Homer ->", run(10, lambda q: parent("Homer", q)))
print("Grandparent grandchild pairs ->", run(10, lambda q: call_fresh(lambda p, x, y: all([
    eq(q, Cons.list([x, y])),
    parent(x, p),
    parent(p, y) ]))))
print()


def parent_alt(x, y):
    return cond([
        [eq(x, "Homer"), eq(y, "Bart")],
        [eq(x, "Marge"), eq(y, "Bart")],
        [eq(x, "Homer"), eq(y, "Lisa")],
        [eq(x, "Marge"), eq(y, "Lisa")],
        [eq(x, "Abe"),   eq(y, "Homer")],
        [eq(x, "Jackie"),eq(y, "Marge")],
    ])

print("Alternative parent() implementation around cond()")
print("Children of Homer ->", run(10, lambda q: parent_alt("Homer", q)))


def yield_forever() -> Goal:
    return _yield(yield_forever)

def terminates() -> Goal:
    return both(eq(True, False), yield_forever())

def boom() -> Goal:
    return both(yield_forever(), eq(True, False))

print("BUG")
print("There still is issue with the order of evaluation")
print("Following goals are logically equivalent but one terminates and the other blows the stack")
print("terminates ->", run(10, lambda x: terminates()))
print("boom ->", run(10, lambda x: boom()))
