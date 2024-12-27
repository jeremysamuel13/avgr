use crate::route::{handler::Handler, path::RoutePath, Route};

pub mod hashmap;

pub trait RouteStorage<P, O, C>
where
    P: RoutePath,
{
    fn add_route(&mut self, path: impl Into<P>, handler: impl Handler<O, C>);

    fn match_route(&self, path: impl Into<P>) -> Option<Route<P, O, C>>;
}
