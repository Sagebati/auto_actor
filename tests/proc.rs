#[cfg(test)]
mod tests {
    use std::thread::spawn;
    use actrix::actrix;

    #[derive(Debug)]
    struct A {
        pub a: usize,
        pub b: &'static str,
    }

    #[actrix]
    impl A {
        fn set_a(&mut self, a: usize) {
            self.a = a;
        }
        fn read_a(&self) -> usize {
            self.a
        }

        fn read_b(&self) -> &'static str {
            self.b
        }
    }

    #[test]
    fn test() {

        let (client, server) = actor::new_server_client(A {
            a: 0,
            b: "azdji"
        });

        spawn(move || {
            server.event_loop();
        });

        assert_eq!(client.read_a(), 0);
        client.set_a(3);
        assert_eq!(client.read_a(), 3);
        assert_eq!(client.read_b(), "azdji");
    }
}