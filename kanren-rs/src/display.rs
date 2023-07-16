use crate::{purify, reify};
use crate::{Goal, StateN, Term, Var};

use std::fmt::Display;

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
                Goal::Fresh(_, node) => {
                    f.write_str(&spacer)?;
                    f.write_str("Fresh\n")?;
                    if let Some(g) = node.borrow().as_ref() {
                        inner(g, f, depth + 1)
                    } else {
                        f.write_str(&spacer)?;
                        f.write_str(" -\n")
                    }
                }
                Goal::Yield(_, node) => {
                    f.write_str(&spacer)?;
                    f.write_str("Yield\n")?;
                    if let Some(g) = node.borrow().as_ref() {
                        inner(g, f, depth + 1)
                    } else {
                        f.write_str(&spacer)?;
                        f.write_str(" -\n")
                    }
                }
            }
        }

        inner(self.0, f, 0)
    }
}
