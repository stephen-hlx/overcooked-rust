use std::any::Any;

pub trait DynPartialEq: Any {
    fn as_any(&self) -> &dyn Any;
    fn dyn_eq(&self, other: &dyn DynPartialEq) -> bool;
}

impl<T: Any + PartialEq> DynPartialEq for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn DynPartialEq) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DynPartialEq;

    trait MyTrait: DynPartialEq {}

    #[derive(PartialEq)]
    struct MyStructA {
        value: i32,
    }

    impl MyTrait for MyStructA {}

    #[derive(PartialEq)]
    struct MyStructB {
        value: i32,
    }

    impl MyTrait for MyStructB {}

    #[test]
    fn works() {
        let a1: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let a2: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let b1: Box<dyn MyTrait> = Box::new(MyStructB { value: 1 });
        let b2: Box<dyn MyTrait> = Box::new(MyStructB { value: 2 });

        assert!(a1.dyn_eq(a2.as_ref()));
        assert!(!a1.dyn_eq(b1.as_ref()));
        assert!(!b1.dyn_eq(b2.as_ref()));
    }
}
