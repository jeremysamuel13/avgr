use std::collections::HashMap;

use crate::{
    dependency::container::DependencyContainer,
    route::{handler::Handler, path::RoutePath, Route},
};

use super::RouteStorage;

pub struct HashMapStorage<P, O, C>
where
    P: RoutePath,
    C: DependencyContainer,
{
    routes: HashMap<String, Route<P, O, C>>,
}

impl<P, O, C> Default for HashMapStorage<P, O, C>
where
    P: RoutePath,
    C: DependencyContainer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P, O, C> HashMapStorage<P, O, C>
where
    P: RoutePath,
    C: DependencyContainer,
{
    pub fn new() -> Self {
        HashMapStorage {
            routes: HashMap::new(),
        }
    }
}

impl<P, O, C> RouteStorage<P, O, C> for HashMapStorage<P, O, C>
where
    P: RoutePath,
    C: DependencyContainer,
{
    fn add_route(&mut self, path: impl Into<P>, handler: impl Handler<O, C>) {
        let path = path.into();
        self.routes
            .insert(path.string_repr(), Route::new(path, handler));
    }

    fn match_route(&self, path: impl Into<P>) -> Option<Route<P, O, C>> {
        let path = path.into();
        self.routes.get(&path.string_repr()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::container::dashmap::DashmapDependencyContainer;

    use super::*;

    #[test]
    fn test_add_and_match_route() {
        let mut storage = HashMapStorage::<String, (), DashmapDependencyContainer>::new();

        storage.add_route("/test", |_| ());

        let matched = storage.match_route("/test");
        assert!(matched.is_some());

        let not_matched = storage.match_route("/nonexistent");
        assert!(not_matched.is_none());
    }

    #[test]
    fn test_default() {
        let storage = HashMapStorage::<String, (), DashmapDependencyContainer>::default();
        assert!(storage.routes.is_empty());
    }
}
