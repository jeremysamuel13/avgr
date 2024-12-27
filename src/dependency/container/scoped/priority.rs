#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ScopePriority {
    Global,
    Runtime,
    User(u8),
}

impl PartialOrd for ScopePriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScopePriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ScopePriority::Global, ScopePriority::Global) => std::cmp::Ordering::Equal,
            (ScopePriority::Global, ScopePriority::Runtime) => std::cmp::Ordering::Greater,
            (ScopePriority::Global, ScopePriority::User(_)) => std::cmp::Ordering::Greater,
            (ScopePriority::Runtime, ScopePriority::Global) => std::cmp::Ordering::Less,
            (ScopePriority::Runtime, ScopePriority::Runtime) => std::cmp::Ordering::Equal,
            (ScopePriority::Runtime, ScopePriority::User(_)) => std::cmp::Ordering::Greater,
            (ScopePriority::User(_), ScopePriority::Global) => std::cmp::Ordering::Less,
            (ScopePriority::User(_), ScopePriority::Runtime) => std::cmp::Ordering::Less,
            (ScopePriority::User(a), ScopePriority::User(b)) => a.cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::dependency::container::scoped::{definition::ScopeDefinition, system::SystemScope};

    use super::*;

    #[derive(Eq, PartialEq, Hash, Debug, Clone)]
    struct CustomScope {
        priority: u8,
    }

    impl ScopeDefinition for CustomScope {
        fn priority(&self) -> ScopePriority {
            ScopePriority::User(self.priority)
        }
    }

    #[test]
    fn test_priority() {
        assert_eq!(SystemScope::<()>::Global.priority(), ScopePriority::Global);
        assert_eq!(
            SystemScope::<()>::Runtime.priority(),
            ScopePriority::Runtime
        );
        assert_eq!(
            SystemScope::User(()).priority(),
            ScopePriority::User(u8::MAX)
        );
        assert_eq!(
            SystemScope::User(CustomScope { priority: 5 }).priority(),
            ScopePriority::User(5)
        );
    }

    #[test]
    fn test_ord() {
        assert!(ScopePriority::Global > ScopePriority::Runtime);
        assert!(ScopePriority::Runtime > ScopePriority::User(5));
        assert!(ScopePriority::User(5) < ScopePriority::User(6));
        assert!(ScopePriority::User(5) == ScopePriority::User(5));
    }
}
