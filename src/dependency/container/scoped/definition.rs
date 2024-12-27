use super::{priority::ScopePriority, system::SystemScope};
use std::hash::Hash;

pub trait ScopeDefinition: Eq + PartialEq + Hash {
    fn priority(&self) -> ScopePriority;
}

impl ScopeDefinition for () {
    fn priority(&self) -> ScopePriority {
        ScopePriority::User(u8::MAX)
    }
}

impl<UserScope: ScopeDefinition> ScopeDefinition for SystemScope<UserScope> {
    fn priority(&self) -> ScopePriority {
        match self {
            SystemScope::Global => ScopePriority::Global,
            SystemScope::Runtime => ScopePriority::Runtime,
            SystemScope::User(u) => u.priority(),
        }
    }
}

impl<T: ScopeDefinition> From<T> for SystemScope<T> {
    fn from(value: T) -> Self {
        SystemScope::User(value)
    }
}
