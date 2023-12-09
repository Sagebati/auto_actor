#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::thread::spawn;
    use actrix::actrix;


    #[test]
    fn test() {
        struct A {
            a: usize,
            b: &'static str,
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

        let (s, r) = channel();
        let server = AServer {
             inner: A { a: 0, b: "" },
             channel: r,
         };

        let client = AClient {
            channel: s
        };

        spawn(move || {
            server.event_loop();
        });

        assert_eq!(client.read_a(), 0);
        client.set_a(3);
        assert_eq!(client.read_a(), 3);
    }
}