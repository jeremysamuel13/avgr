pub trait Handler<O, C>: Send + Sync + 'static {
    fn handle(&self, container: C) -> O;
}

impl<F, O, C> Handler<O, C> for F
where
    F: Fn(C) -> O + 'static + Send + Sync,
{
    fn handle(&self, container: C) -> O {
        (self)(container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_with_closure() {
        let handler = |x: i32| x * 2;
        let result = handler.handle(5);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_handler_with_string() {
        let handler = |s: String| s.to_uppercase();
        let result = handler.handle("hello".to_string());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_handler_with_vec() {
        let handler = |v: Vec<i32>| v.iter().sum::<i32>();
        let result = handler.handle(vec![1, 2, 3, 4, 5]);
        assert_eq!(result, 15);
    }
}
