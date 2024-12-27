pub mod definition;
pub mod priority;
pub mod system;

#[cfg(test)]
mod tests;

use dashmap::{mapref::one::Ref, DashMap};
use itertools::Itertools;

use definition::ScopeDefinition;
use system::SystemScope;

use super::DependencyContainer;

pub struct ScopedDependencyContainer<C: DependencyContainer, UserScope: ScopeDefinition> {
    scopes: DashMap<SystemScope<UserScope>, C>,
}

impl<C: DependencyContainer + Default, UserScope: ScopeDefinition + Clone> Default
    for ScopedDependencyContainer<C, UserScope>
{
    fn default() -> Self {
        let c = Self::new_empty();
        c.create_default_scope(SystemScope::Global);
        c.create_default_scope(SystemScope::Runtime);

        c
    }
}

impl<C: DependencyContainer + Default, UserScope: ScopeDefinition + Clone>
    ScopedDependencyContainer<C, UserScope>
{
    pub fn create_default_scope(
        &self,
        scope: impl Into<SystemScope<UserScope>>,
    ) -> Ref<'_, SystemScope<UserScope>, C> {
        self.create_scope_with_factory(scope, C::default)
    }

    pub fn register_with_default_scope<T: std::any::Any + Send + Sync>(
        &self,
        scope: impl Into<SystemScope<UserScope>>,
        dependency: T,
    ) -> Option<<ScopedDependencyContainer<C, UserScope> as DependencyContainer>::DependencyOwned<T>>
    {
        let scope = scope.into();
        if !self.scopes.contains_key(&scope) {
            self.create_default_scope(scope.clone());
        }
        self.get_scope(&scope)
            .expect("Scope not found")
            .register(dependency)
    }
}

impl<C: DependencyContainer, UserScope: ScopeDefinition> ScopedDependencyContainer<C, UserScope> {
    pub(crate) fn new_empty() -> Self {
        Self {
            scopes: DashMap::new(),
        }
    }

    pub fn create_scope_with_factory<F>(
        &self,
        scope: impl Into<SystemScope<UserScope>>,
        container_fn: F,
    ) -> Ref<'_, SystemScope<UserScope>, C>
    where
        F: Fn() -> C,
    {
        self.create_scope(scope, container_fn())
    }

    pub fn create_scope(
        &self,
        scope: impl Into<SystemScope<UserScope>>,
        container: C,
    ) -> Ref<'_, SystemScope<UserScope>, C> {
        let scope = scope.into();
        let entry = self.scopes.entry(scope).or_insert(container).downgrade();
        self.get_scope(entry.key()).expect("Scope should exist")
    }

    pub fn delete_scope(&self, scope: impl Into<SystemScope<UserScope>>) -> bool {
        self.scopes.remove(&scope.into()).is_some()
    }

    pub fn get_scope(
        &self,
        scope: &SystemScope<UserScope>,
    ) -> Option<Ref<'_, SystemScope<UserScope>, C>> {
        self.scopes.get(scope)
    }

    pub fn has_scope(&self, scope: &SystemScope<UserScope>) -> bool {
        self.scopes.contains_key(scope)
    }

    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    pub fn clear_scopes(&self) {
        self.scopes.clear();
    }

    pub fn register_with_scope<T: std::any::Any + Send + Sync>(
        &self,
        dependency: T,
        scope: impl Into<SystemScope<UserScope>>,
    ) -> Option<<ScopedDependencyContainer<C, UserScope> as DependencyContainer>::DependencyOwned<T>>
    {
        let scope = scope.into();
        self.get_scope(&scope)?.register(dependency)
    }

    pub fn register_with_scope_factory<T: std::any::Any + Send + Sync, F: Fn() -> C>(
        &self,
        dependency: T,
        scope: impl Into<SystemScope<UserScope>>,
        container_fn: F,
    ) -> Option<<ScopedDependencyContainer<C, UserScope> as DependencyContainer>::DependencyOwned<T>>
    {
        self.create_scope_with_factory(scope, container_fn)
            .register(dependency)
    }

    pub fn resolve_from_scope<T: std::any::Any + Send + Sync>(
        &self,
        scope: &SystemScope<UserScope>,
    ) -> Option<<ScopedDependencyContainer<C, UserScope> as DependencyContainer>::DependencyRef<T>>
    {
        self.get_scope(scope)?.resolve::<T>()
    }

    pub fn deregister_from_scope<T: std::any::Any + Send + Sync>(
        &self,
        scope: &SystemScope<UserScope>,
    ) -> Option<<ScopedDependencyContainer<C, UserScope> as DependencyContainer>::DependencyOwned<T>>
    {
        self.get_scope(scope)?.deregister::<T>()
    }
}

impl<C: DependencyContainer, UserScope: ScopeDefinition> DependencyContainer
    for ScopedDependencyContainer<C, UserScope>
{
    type DependencyRef<T> = C::DependencyRef<T>;

    type DependencyOwned<T> = C::DependencyOwned<T>;

    fn resolve<T: std::any::Any + Send + Sync>(&self) -> Option<Self::DependencyRef<T>> {
        self.scopes
            .iter()
            .sorted_by_key(|e| e.key().priority())
            .find_map(|entry| entry.value().resolve::<T>())
    }

    fn register<T: std::any::Any + Send + Sync>(
        &self,
        dependency: T,
    ) -> Option<Self::DependencyOwned<T>> {
        self.scopes
            .iter()
            .max_by_key(|e| e.key().priority())
            .and_then(|entry| entry.value().register(dependency))
    }

    fn deregister<T: std::any::Any + Send + Sync>(&self) -> Option<Self::DependencyOwned<T>> {
        self.scopes
            .iter()
            .max_by_key(|e| e.key().priority())
            .and_then(|entry| entry.value().deregister::<T>())
    }
}
