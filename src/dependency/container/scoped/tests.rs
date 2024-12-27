use crate::dependency::container::dashmap::DashmapDependencyContainer;
use priority::ScopePriority;

use super::*;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct CustomScope {
    priority: u8,
}

impl ScopeDefinition for CustomScope {
    fn priority(&self) -> ScopePriority {
        ScopePriority::User(self.priority)
    }
}

mod scopes {

    use super::*;

    fn setup_container() -> ScopedDependencyContainer<DashmapDependencyContainer, CustomScope> {
        ScopedDependencyContainer::new_empty()
    }

    #[test]
    fn test_scope_isolation() {
        let container = setup_container();

        let scope1 = CustomScope { priority: 1 };
        let scope2 = CustomScope { priority: 2 };

        container.create_default_scope(scope1.clone());
        container.create_default_scope(scope2.clone());

        container.register_with_scope(42, scope1.clone());
        container.register_with_scope(24, scope2.clone());

        assert_eq!(
            *container
                .resolve_from_scope::<i32>(&SystemScope::User(scope1))
                .unwrap(),
            42
        );
        assert_eq!(
            *container
                .resolve_from_scope::<i32>(&SystemScope::User(scope2))
                .unwrap(),
            24
        );
    }

    #[test]
    fn test_scope_inheritance() {
        let container = setup_container();

        container.create_default_scope(CustomScope { priority: 1 });
        container.create_default_scope(CustomScope { priority: 2 });

        container.register_with_default_scope(CustomScope { priority: 1 }, "parent value");

        assert_eq!(*container.resolve::<&str>().unwrap(), "parent value");
    }

    #[test]
    fn test_scope_shadowing() {
        let container = setup_container();

        let parent_scope = CustomScope { priority: 2 };
        let child_scope = CustomScope { priority: 1 };

        container.create_default_scope(parent_scope.clone());
        container.create_default_scope(child_scope.clone());

        container.register_with_scope("parent value", parent_scope);
        container.register_with_scope("child value", child_scope);

        assert_eq!(*container.resolve::<&str>().unwrap(), "child value");
    }
}

mod container {
    use super::*;

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let container = Arc::new(ScopedDependencyContainer::<
            DashmapDependencyContainer,
            CustomScope,
        >::new_empty());
        let scope = CustomScope { priority: 1 };
        container.create_default_scope(scope.clone());

        let threads: Vec<_> = (0..10)
            .map(|i| {
                let container = Arc::clone(&container);
                thread::spawn({
                    let value = scope.clone();
                    move || {
                        container.register_with_default_scope(value.clone(), i);
                        container.resolve::<i32>().unwrap()
                    }
                })
            })
            .collect();

        for handle in threads {
            handle.join().unwrap();
        }

        assert!(container.resolve::<i32>().is_some());
    }

    #[test]
    fn test_type_safety() {
        let container =
            ScopedDependencyContainer::<DashmapDependencyContainer, CustomScope>::new_empty();
        let scope = CustomScope { priority: 1 };
        container.create_default_scope(scope.clone());

        container.register_with_default_scope(scope.clone(), 42i32);

        assert!(container.resolve::<i64>().is_none());
        assert!(container.resolve::<String>().is_none());
    }

    #[test]
    fn test_scope_cleanup() {
        let container =
            ScopedDependencyContainer::<DashmapDependencyContainer, CustomScope>::new_empty();
        let scope = CustomScope { priority: 1 };

        container.create_default_scope(scope.clone());
        container.register_with_default_scope(scope.clone(), 42);

        assert!(container.delete_scope(scope.clone()));
        assert!(container.resolve::<i32>().is_none());
    }

    #[test]
    fn test_multiple_registrations() {
        let container =
            ScopedDependencyContainer::<DashmapDependencyContainer, CustomScope>::new_empty();
        let scope = CustomScope { priority: 1 };
        container.create_default_scope(scope.clone());

        container.register_with_default_scope(scope.clone(), 1);
        container.register_with_default_scope(scope.clone(), 2);
        container.register_with_default_scope(scope.clone(), 3);

        assert_eq!(*container.resolve::<i32>().unwrap(), 3);
    }

    #[test]
    fn test_complex_type() {
        #[derive(Debug, PartialEq)]
        struct ComplexType {
            value: String,
            number: i32,
        }

        let container =
            ScopedDependencyContainer::<DashmapDependencyContainer, CustomScope>::new_empty();
        let scope = CustomScope { priority: 1 };
        container.create_default_scope(scope.clone());

        let complex = ComplexType {
            value: "test".to_string(),
            number: 42,
        };

        container.register_with_default_scope(scope, complex);

        let resolved = container.resolve::<ComplexType>().unwrap();
        assert_eq!(resolved.value, "test");
        assert_eq!(resolved.number, 42);
    }

    #[test]
    fn test_empty_container() {
        let container =
            ScopedDependencyContainer::<DashmapDependencyContainer, CustomScope>::new_empty();
        assert!(container.resolve::<i32>().is_none());
        assert!(container.scopes.is_empty());
    }
}

mod integration {
    use super::*;

    #[test]
    fn test_realistic_usage() {
        trait UserService: Send + Sync {
            fn get_user_name(&self) -> String;
        }

        struct UserServiceImpl {
            prefix: String,
        }

        impl UserService for UserServiceImpl {
            fn get_user_name(&self) -> String {
                format!("{}_user", self.prefix)
            }
        }

        let container =
            ScopedDependencyContainer::<DashmapDependencyContainer, CustomScope>::new_empty();

        let global_scope = CustomScope { priority: 1 };
        let request_scope = CustomScope { priority: 2 };

        container.create_default_scope(global_scope.clone());
        container.create_default_scope(request_scope.clone());

        let user_service = UserServiceImpl {
            prefix: "test".to_string(),
        };

        container.register_with_default_scope(global_scope, user_service);

        let service = container.resolve::<UserServiceImpl>().unwrap();
        assert_eq!(service.get_user_name(), "test_user");
    }
}
