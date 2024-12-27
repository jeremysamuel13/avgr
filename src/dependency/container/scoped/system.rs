use super::definition::ScopeDefinition;

#[derive(Default, Eq, PartialEq, Hash, Debug, Clone)]
pub enum SystemScope<UserScope: ScopeDefinition = ()> {
    Global,
    #[default]
    Runtime,
    User(UserScope),
}
