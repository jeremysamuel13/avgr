pub mod dashmap;
pub mod scoped;

use std::{any::Any, ops::Deref};

pub trait DependencyContainer {
    type DependencyRef<T>: Deref<Target = T>;
    type DependencyOwned<T>: Deref<Target = T>;

    fn resolve<T: Any + Send + Sync>(&self) -> Option<Self::DependencyRef<T>>;
    fn register<T: Any + Send + Sync>(&self, dependency: T) -> Option<Self::DependencyOwned<T>>;
    fn deregister<T: Any + Send + Sync>(&self) -> Option<Self::DependencyOwned<T>>;

    fn register_default<T: Any + Send + Sync + Default>(&self) -> Option<Self::DependencyOwned<T>> {
        self.register(T::default())
    }
}

impl<V, C> DependencyContainer for V
where
    V: Deref<Target = C>,
    C: DependencyContainer,
{
    type DependencyRef<T> = C::DependencyRef<T>;

    type DependencyOwned<T> = C::DependencyOwned<T>;

    fn resolve<T: Any + Send + Sync>(&self) -> Option<Self::DependencyRef<T>> {
        self.deref().resolve::<T>()
    }

    fn register<T: Any + Send + Sync>(&self, dependency: T) -> Option<Self::DependencyOwned<T>> {
        self.deref().register::<T>(dependency)
    }

    fn deregister<T: Any + Send + Sync>(&self) -> Option<Self::DependencyOwned<T>> {
        self.deref().deregister::<T>()
    }
}
