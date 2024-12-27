use std::sync::{atomic::AtomicU32, Arc};

use avgr::{
    dependency::container::{scoped::system::SystemScope, DependencyContainer},
    route::handler::Handler,
    router::StandardRouter,
};

fn ping<C: DependencyContainer>(container: C) {
    let request_counter = container
        .resolve::<RequestCounter>()
        .expect("Could not resolve dependency: RequestCounter");

    request_counter
        .count
        .fetch_add(1, std::sync::atomic::Ordering::AcqRel);

    println!("pong");
}

#[derive(Default)]
struct RequestCounter {
    count: AtomicU32,
}

struct StatefulHandler {
    count: AtomicU32,
}

impl<C> Handler<(), C> for StatefulHandler {
    fn handle(&self, _container: C) {
        println!(
            "State: {}",
            self.count.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
        );
    }
}

fn main() {
    let mut router = StandardRouter::<()>::default();
    router
        .container
        .register_with_default_scope(SystemScope::Global, RequestCounter::default());

    router.add_route("ping", ping);
    router.add_route("hello", |_| println!("hello"));
    router.add_route(
        "stateful",
        StatefulHandler {
            count: AtomicU32::new(0),
        },
    );

    let router = Arc::new(router);

    let routes = ["ping", "hello", "stateful"];
    let mut handles = vec![];

    for _ in 0..50 {
        let route = routes[rand::random::<usize>() % routes.len()].to_string();
        let router_clone = router.clone();

        handles.push(std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(rand::Rng::gen_range(
                &mut rand::thread_rng(),
                2..=10,
            )));
            router_clone.dispatch(route).expect("Failed to dispatch");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "Request count: {:?}",
        router.container.resolve::<RequestCounter>().unwrap().count
    );
}
