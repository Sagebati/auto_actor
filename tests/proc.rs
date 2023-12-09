#[cfg(test)]
mod tests {
    use actrix::actrix;

    #[test]
    fn test() {
        struct A {
            a: usize,
            b: &'static str,
        }

        #[actrix]
        impl A {
            fn test(self, a: usize) {
                todo!()
            }
            fn test1(&self, a: usize) { todo!() }

            fn test_ref(&self, a: &usize) -> usize { todo!() }
        }

        assert!(true)
    }
}