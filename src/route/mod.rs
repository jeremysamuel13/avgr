use std::sync::Arc;

use handler::Handler;
use path::RoutePath;

pub mod handler;
pub mod path;

pub struct Route<P, O, C>
where
    P: RoutePath,
{
    pub path: P,
    handler: Arc<dyn Handler<O, C>>,
}

impl<P, O, C> Clone for Route<P, O, C>
where
    P: RoutePath,
{
    fn clone(&self) -> Self {
        Route {
            path: self.path.clone(),
            handler: self.handler.clone(),
        }
    }
}

impl<P, O, C> Route<P, O, C>
where
    P: RoutePath,
{
    pub fn new(path: impl Into<P>, handler: impl Handler<O, C>) -> Route<P, O, C> {
        Route {
            path: path.into(),
            handler: Arc::new(handler),
        }
    }

    pub fn handle(&self, container: C) -> O
    where
        C: 'static,
        O: 'static,
    {
        self.handler.handle(container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_creation() {
        let route: Route<String, _, _> = Route::new("/test", |_| 42);

        assert_eq!(route.path.as_str(), "/test");
        assert_eq!(route.handle(()), 42);
    }

    #[test]
    fn test_route_cloning() {
        let route: Route<String, _, _> = Route::new("/test", |_| 42);
        let cloned = route.clone();

        assert_eq!(route.path.as_str(), cloned.path.as_str());
        assert_eq!(route.handle(()), cloned.handle(()));
    }
}
