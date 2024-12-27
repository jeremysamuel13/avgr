pub trait RoutePath: Clone {
    fn string_repr(&self) -> String;
}

impl RoutePath for String {
    fn string_repr(&self) -> String {
        self.clone()
    }
}

impl RoutePath for &str {
    fn string_repr(&self) -> String {
        self.to_string()
    }
}
