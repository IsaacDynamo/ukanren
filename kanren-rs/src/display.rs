use crate::{purify, reify, FreshInner, YieldInner};
use crate::{Goal, StateN, Term, Var};

use std::fmt::Display;
use std::ops::Deref;

pub struct AsScheme<T: DisplayScheme>(pub T);

impl<T: DisplayScheme> Display for AsScheme<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Scheme<'a, T: DisplayScheme>(&'a T);

impl<'a, T: DisplayScheme> Display for Scheme<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub trait DisplayScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<const N: usize> DisplayScheme for StateN<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        AsScheme(reify::<N>(&self.state)).fmt(f)?;
        let mut constraints = purify::<N>(&self.state);
        if !constraints.is_empty() {
            // Sort to make string representation comparable
            for constraint in constraints.iter_mut() {
                constraint.sort();
            }
            constraints.sort();
            f.write_str(" : ")?;
            Scheme(&constraints).fmt(f)?;
        }
        Ok(())
    }
}

impl DisplayScheme for (Var, Term) {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("(_{} . {})", self.0 .0, Scheme(&self.1)))
    }
}

impl DisplayScheme for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self).fmt(f)
    }
}

impl DisplayScheme for &Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn inner(term: &Term, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match term {
                Term::Null => Ok(()),
                Term::Cons(head, tail) => {
                    f.write_str(" ")?;
                    head.as_ref().fmt(f)?;
                    inner(tail.as_ref(), f)
                }
                _ => {
                    f.write_str(" . ")?;
                    term.fmt(f)
                }
            }
        }

        match self {
            Term::Var(x) => f.write_fmt(format_args!("_{}", x.0)),
            Term::Value(x) => f.write_fmt(format_args!("{x}")),
            Term::String(x) => {
                if x.contains(' ') {
                    f.write_fmt(format_args!("\"{x}\""))
                } else {
                    f.write_fmt(format_args!("{x}"))
                }
            }
            Term::Null => f.write_str("()"),
            Term::Cons(head, tail) => {
                f.write_str("(")?;
                head.as_ref().fmt(f)?;
                inner(tail.as_ref(), f)?;
                f.write_str(")")
            }
        }
    }
}

impl<T: DisplayScheme> DisplayScheme for Vec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T: DisplayScheme, const N: usize> DisplayScheme for [T; N] {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T: DisplayScheme> DisplayScheme for &[T] {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        for (i, x) in self.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            x.fmt(f)?;
        }
        f.write_str(")")
    }
}

pub struct GoalTree<'a>(pub &'a Goal);

impl<'a> std::fmt::Display for GoalTree<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn inner(goal: &Goal, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
            let spacer = " ".repeat(depth);
            match goal {
                Goal::Eq(a, b) => f.write_fmt(format_args!("{}{:?} == {:?}\n", spacer, a, b)),
                Goal::Neq(a, b) => f.write_fmt(format_args!("{}{:?} != {:?}\n", spacer, a, b)),
                Goal::Both(a, b) => {
                    f.write_str(&spacer)?;
                    f.write_str("Both\n")?;
                    inner(a, f, depth + 1)?;
                    inner(b, f, depth + 1)
                }
                Goal::Either(a, b) => {
                    f.write_str(&spacer)?;
                    f.write_str("Either\n")?;
                    inner(a, f, depth + 1)?;
                    inner(b, f, depth + 1)
                }
                Goal::Fresh(fresh_inner) => {
                    f.write_str(&spacer)?;
                    f.write_str("Fresh\n")?;
                    let fresh_inner = fresh_inner.borrow();
                    match fresh_inner.deref() {
                        FreshInner::Pending(_) => {
                            f.write_str(&spacer)?;
                            f.write_str(" -\n")
                        }
                        FreshInner::Resolved(goal) => inner(goal, f, depth + 1),
                    }
                }
                Goal::Yield(yield_inner) => {
                    f.write_str(&spacer)?;
                    f.write_str("Yield\n")?;
                    let yield_inner = yield_inner.borrow();
                    match yield_inner.deref() {
                        YieldInner::Pending(_) => {
                            f.write_str(&spacer)?;
                            f.write_str(" -\n")
                        }
                        YieldInner::Resolved(goal) => inner(goal, f, depth + 1),
                    }
                }
            }
        }

        inner(self.0, f, 0)
    }
}


use std::io::Write;

pub fn output_dot(output: &mut impl Write, goal: &Goal) -> std::io::Result<()> {

    fn id(goal: &Goal) -> usize {
        goal as *const _ as usize
    }

    fn link(output: &mut impl Write, parent: &Goal, goal: &Goal) -> std::io::Result<()> {
        if id(parent) != id(goal) {
            output.write_fmt(format_args!("n{} -> n{}\n", id(parent), id(goal)))?;
        }
        Ok(())
    }

    fn inner(output: &mut impl Write, parent: &Goal, goal: &Goal) -> std::io::Result<()> {
        match goal {
            Goal::Eq(a, b) => {
                output.write_fmt(format_args!("n{} [label=\"==\"]\n", id(goal)))?;
                link(output, parent, goal)?;


                let a = format!("{}", AsScheme(a)).escape_debug().collect::<String>();
                let b = format!("{}", AsScheme(b)).escape_debug().collect::<String>();
                output.write_fmt(format_args!("n{}_a [label=\"{}\" shape=box]\n", id(goal), a))?;
                output.write_fmt(format_args!("n{} -> n{}_a\n", id(goal), id(goal)))?;
                output.write_fmt(format_args!("n{}_b [label=\"{}\" shape=box]\n", id(goal), b))?;
                output.write_fmt(format_args!("n{} -> n{}_b\n", id(goal), id(goal)))?;

                Ok(())
            },
            Goal::Neq(_, _) => {
                output.write_fmt(format_args!("n{} [label=\"!=\"]\n", id(goal)))?;
                link(output, parent, goal)?;

                Ok(())
            },
            Goal::Both(a, b) => {
                if let Goal::Both(_, _) = parent {
                    inner(output, parent, a)?;
                    inner(output, parent, b)?;
                } else {
                    output.write_fmt(format_args!("n{} [label=\"&&\"]\n", id(goal)))?;
                    link(output, parent, goal)?;
                    inner(output, goal, a)?;
                    inner(output, goal, b)?;
                }
                Ok(())
            },
            Goal::Either(a, b) => {
                if let Goal::Either(_, _) = parent {
                    inner(output, parent, a)?;
                    inner(output, parent, b)?;
                } else {
                    output.write_fmt(format_args!("n{} [label=\"||\"]\n", id(goal)))?;
                    link(output, parent, goal)?;
                    inner(output, goal, a)?;
                    inner(output, goal, b)?;
                }
                Ok(())
            },
            Goal::Fresh(i) => {
                let i = i.borrow();
                if let FreshInner::Resolved(x) = i.deref() {
                    inner(output, parent, x)?;
                } else {
                    output.write_fmt(format_args!("n{} [label=\"Fresh\"]\n", id(goal)))?;
                    link(output, parent, goal)?;
                }
                Ok(())
            },
            Goal::Yield(i) => {
                let i = i.borrow();
                if let YieldInner::Resolved(x) = i.deref() {
                    output.write_fmt(format_args!("n{} [label=\"Yield: {}\"]\n", id(goal), std::rc::Rc::strong_count(x)))?;
                    link(output, parent, goal)?;
                    inner(output, goal, x)?;
                } else {
                    output.write_fmt(format_args!("n{} [label=\"Yield\"]\n", id(goal)))?;
                    link(output, parent, goal)?;
                }
                Ok(())
            },
        }
    }

    output.write("digraph {\n".as_bytes())?;
    inner(output, goal, goal)?;
    output.write("}\n".as_bytes())?;

    Ok(())
}