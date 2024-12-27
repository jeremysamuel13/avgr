use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use dashmap::DashMap;

use super::DependencyContainer;

#[derive(Default)]
pub struct DashmapDependencyContainer {
    dashmap: DashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl DependencyContainer for DashmapDependencyContainer {
    type DependencyRef<T> = Arc<T>;
    type DependencyOwned<T> = Arc<T>;

    fn resolve<T: Any + Send + Sync>(&self) -> Option<Self::DependencyRef<T>> {
        self.dashmap
            .get(&TypeId::of::<T>())
            .and_then(|value| value.clone().downcast().ok())
    }

    fn register<T: Any + Send + Sync>(&self, dependency: T) -> Option<Self::DependencyOwned<T>> {
        self.dashmap
            .insert(TypeId::of::<T>(), Arc::new(dependency))
            .and_then(|value| value.downcast().ok())
    }

    fn deregister<T: Any + Send + Sync>(&self) -> Option<Self::DependencyOwned<T>> {
        self.dashmap
            .remove(&TypeId::of::<T>())
            .and_then(|(_, value)| value.downcast().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn create_container() -> DashmapDependencyContainer {
        DashmapDependencyContainer::default()
    }

    #[test]
    fn test_basic_resolve_register() {
        let container = create_container();

        let value = 42;
        container.register(value);
        
        let resolved = container.resolve::<i32>().unwrap();
        assert_eq!(*resolved, 42);
    }

    #[test]
    fn test_multiple_types() {
        let container = create_container();

        container.register(42i32);
        container.register("hello".to_string());
        container.register(123.45f64);

        assert_eq!(*container.resolve::<i32>().unwrap(), 42);
        assert_eq!(*container.resolve::<String>().unwrap(), "hello");
        assert_eq!(*container.resolve::<f64>().unwrap(), 123.45);
    }

    #[test]
    fn test_override_registration() {
        let container = create_container();

        container.register(42i32);
        assert_eq!(*container.resolve::<i32>().unwrap(), 42);

        container.register(84i32);
        assert_eq!(*container.resolve::<i32>().unwrap(), 84);
    }

    #[test]
    fn test_deregister() {
        let container = create_container();

        container.register(42i32);
        assert_eq!(*container.resolve::<i32>().unwrap(), 42);

        let deregistered = container.deregister::<i32>().unwrap();
        assert_eq!(*deregistered, 42);
        assert!(container.resolve::<i32>().is_none());
    }

    #[test]
    fn test_missing_dependency() {
        let container = create_container();
        assert!(container.resolve::<i32>().is_none());
    }

    #[test]
    fn test_complex_type() {
        #[derive(Debug, PartialEq)]
        struct ComplexType {
            value: String,
            number: i32,
        }

        let container = create_container();
        let complex = ComplexType {
            value: "test".to_string(),
            number: 42,
        };

        container.register(complex);

        let resolved = container.resolve::<ComplexType>().unwrap();
        assert_eq!(resolved.value, "test");
        assert_eq!(resolved.number, 42);
    }

    #[test]
    fn test_thread_safety() {
        let container = Arc::new(create_container());
        let threads: Vec<_> = (0..10)
            .map(|i| {
                let container = Arc::clone(&container);
                thread::spawn(move || {
                    container.register(i);
                    let resolved = container.resolve::<i32>().unwrap();
                    *resolved
                })
            })
            .collect();

        for handle in threads {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_trait_objects() {
        trait TestTrait: Send + Sync {
            fn get_value(&self) -> i32;
        }

        struct TestImpl {
            value: i32,
        }

        impl TestTrait for TestImpl {
            fn get_value(&self) -> i32 {
                self.value
            }
        }

        let container = create_container();
        container.register(TestImpl { value: 42 });

        let resolved = container.resolve::<TestImpl>().unwrap();
        assert_eq!(resolved.get_value(), 42);
    }

    #[test]
    fn test_arc_handling() {
        let container = create_container();
        let value = Arc::new(42i32);
        
        container.register(value.clone());
        
        let resolved = container.resolve::<Arc<i32>>().unwrap();
        assert_eq!(**resolved, 42);
    }

    #[test]
    fn test_multiple_references() {
        let container = create_container();
        container.register(42i32);

        let ref1 = container.resolve::<i32>().unwrap();
        let ref2 = container.resolve::<i32>().unwrap();

        assert_eq!(*ref1, 42);
        assert_eq!(*ref2, 42);
    }

    #[test]
    fn test_type_safety() {
        let container = create_container();
        container.register(42i32);

        assert!(container.resolve::<i64>().is_none());
        assert!(container.resolve::<String>().is_none());
    }

    #[test]
    fn test_deregister_missing() {
        let container = create_container();
        assert!(container.deregister::<i32>().is_none());
    }

    #[test]
    fn test_concurrent_operations() {
        let container = Arc::new(create_container());
        let threads: Vec<_> = (0..10)
            .map(|i| {
                let container = Arc::clone(&container);
                thread::spawn(move || {
                    container.register(i);
                    let _resolved = container.resolve::<i32>();
                    let _deregistered = container.deregister::<i32>();
                })
            })
            .collect();

        for handle in threads {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_zero_sized_types() {
        #[derive(Debug, PartialEq)]
        struct ZeroSized;

        let container = create_container();
        container.register(ZeroSized);

        assert!(container.resolve::<ZeroSized>().is_some());
    }

    #[test]
    fn test_large_values() {
        let container = create_container();
        let large_vec = vec![0; 1000000];
        container.register(large_vec);

        let resolved = container.resolve::<Vec<i32>>().unwrap();
        assert_eq!(resolved.len(), 1000000);
    }

    #[test]
    fn test_drop_behavior() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        struct DropCheck {
            dropped: Arc<AtomicBool>,
        }

        impl Drop for DropCheck {
            fn drop(&mut self) {
                self.dropped.store(true, Ordering::SeqCst);
            }
        }

        let container = create_container();
        let dropped = Arc::new(AtomicBool::new(false));
        
        container.register(DropCheck {
            dropped: dropped.clone(),
        });

        {
            let _resolved = container.resolve::<DropCheck>();
            assert!(!dropped.load(Ordering::SeqCst));
        }

        container.deregister::<DropCheck>();
        assert!(dropped.load(Ordering::SeqCst));
    }
}

