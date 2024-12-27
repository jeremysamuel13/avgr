use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    dependency::container::{
        dashmap::DashmapDependencyContainer, scoped::ScopedDependencyContainer, DependencyContainer,
    },
    route::{handler::Handler, path::RoutePath, Route},
    storage::{hashmap::HashMapStorage, RouteStorage},
};

pub type RouterContainer<C, UserScope> = Arc<ScopedDependencyContainer<C, UserScope>>;

#[derive(Default)]
pub struct Router<S, P, O, C>
where
    S: RouteStorage<P, O, C>,
    P: RoutePath,
    C: DependencyContainer,
{
    storage: Arc<RwLock<S>>,
    pub container: C,
    _p: std::marker::PhantomData<P>,
    _o: std::marker::PhantomData<O>,
}

impl<S, P, O, C> Router<S, P, O, C>
where
    S: RouteStorage<P, O, C>,
    P: RoutePath,
    C: DependencyContainer,
{
    pub fn new(storage: S, container: impl Into<C>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            container: container.into(),
            _p: std::marker::PhantomData,
            _o: std::marker::PhantomData,
        }
    }

    pub fn add_route(&mut self, path: impl Into<P>, handler: impl Handler<O, C>) {
        self.storage.write().add_route(path, handler);
    }

    fn match_route(&self, path: impl Into<P>) -> Option<Route<P, O, C>> {
        self.storage.read().match_route(path)
    }

    pub fn dispatch(&self, path: P) -> Option<O>
    where
        O: 'static,
        C: 'static + Clone,
    {
        let r = self.match_route(path)?;
        Some(r.handle(self.container.clone()))
    }
}

pub type StandardRouter<O, P = String, C = DashmapDependencyContainer, UserScope = ()> = Router<
    HashMapStorage<P, O, Arc<ScopedDependencyContainer<C, UserScope>>>,
    P,
    O,
    Arc<ScopedDependencyContainer<C, UserScope>>,
>;

// impl<O, P: RoutePath> Default for StandardRouter<O, P> {
//     fn default() -> Self {
//         Self::new(
//             HashMapStorage::default(),
//             Arc::new(ScopedDependencyContainer::default()),
//         )
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch() {
        let mut router: StandardRouter<String> = StandardRouter::default();
        router.add_route("/test", |_| "test".to_string());

        let result = router.dispatch("/test".to_string());
        assert_eq!(result, Some("test".to_string()));

        let not_found = router.dispatch("/notfound".to_string());
        assert_eq!(not_found, None);
    }
}
