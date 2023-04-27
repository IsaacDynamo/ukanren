use std::collections::HashSet;
use std::hash::Hash;

#[macro_export]
macro_rules! set {
    () => (
        std::collections::HashSet::new()
    );
    ($($x:expr),+ $(,)?) => (
        std::collections::HashSet::from([$($x),+])
    );
}

#[derive(Debug, PartialEq, Eq)]
pub enum Relation {
    Subset,
    Equal,
    Superset,
    Joint,
    Disjoint,
}

// Get set relation in a single pass
// returns what `a` is relative to `b`
pub fn relation<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> Relation {
    use Relation::*;

    let overlap = if a.len() <= b.len() {
        (true, a.iter().filter(|v| b.contains(v)).count())
    } else {
        (false, b.iter().filter(|v| a.contains(v)).count())
    };

    match overlap {
        (_, x) if x == a.len() && x == b.len() => Equal,
        (true, x) if x == a.len() => Subset,
        (false, x) if x == b.len() => Superset,
        (_, x) if x > 0 => Joint,
        (_, 0) => Disjoint,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::set::{relation, Relation::*};

    #[test]
    fn equal() {
        assert_eq!(relation(&set!(1), &set!(1)), Equal);
    }

    #[test]
    fn disjoint() {
        assert_eq!(relation(&set!(1), &set!(2)), Disjoint);
    }

    #[test]
    fn joint() {
        assert_eq!(relation(&set!(1, 2), &set!(1, 3)), Joint);
    }

    #[test]
    fn subset() {
        assert_eq!(relation(&set!(1), &set!(1, 2)), Subset);
    }

    #[test]
    fn superset() {
        assert_eq!(relation(&set!(1, 2), &set!(1)), Superset);
    }

    #[test]
    fn empty_some() {
        assert_eq!(relation(&set!(), &set!(1)), Subset);
    }

    #[test]
    fn some_empty() {
        assert_eq!(relation(&set!(1), &set!()), Superset);
    }

    #[test]
    fn empty_empty() {
        assert_eq!(relation::<u8>(&set!(), &set!()), Equal);
    }
}
