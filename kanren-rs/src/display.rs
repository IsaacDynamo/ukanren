use crate::{Goal, Term};

pub struct AsScheme(pub Vec<Vec<Term>>);

impl std::fmt::Display for AsScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn term(t: &Term, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match t {
                Term::Var(x) => f.write_fmt(format_args!("_{x}")),
                Term::Value(x) => f.write_fmt(format_args!("{x}")),
                Term::Cons(head, tail) => {
                    f.write_str("(")?;
                    term(head.as_ref(), f)?;
                    inner(tail.as_ref(), f)?;
                    f.write_str(")")
                }
                _ => unreachable!(),
            }
        }

        fn inner(t: &Term, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match t {
                Term::Null => Ok(()),
                Term::Cons(head, tail) => {
                    f.write_str(" ")?;
                    term(head.as_ref(), f)?;
                    inner(tail.as_ref(), f)
                }
                _ => {
                    f.write_str(" . ")?;
                    term(t, f)
                }
            }
        }

        f.write_str("(")?;
        for (i, vars) in self.0.iter().enumerate() {
            if i != 0 {
                f.write_str(" ")?;
            }
            f.write_str("(")?;
            for (j, var) in vars.iter().enumerate() {
                if j != 0 {
                    f.write_str(" ")?;
                }
                term(var, f)?;
            }
            f.write_str(")")?;
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
