use crate::Term;

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
