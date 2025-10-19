use std::any::Any;
use std::cmp::Ordering;

pub trait DynPartialOrd: Any {
    fn as_any(&self) -> &dyn Any;
    fn dyn_partial_cmp(&self, other: &dyn DynPartialOrd) -> Option<Ordering>;
}

// 3. Implement `DynPartialOrd` for any concrete type that will be used in trait objects.
impl<T: Any + PartialOrd> DynPartialOrd for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_partial_cmp(&self, other: &dyn DynPartialOrd) -> Option<Ordering> {
        // First, check if the other object has the same underlying concrete type.
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            // If the types match, use the standard `PartialEq` implementation.
            self.partial_cmp(other)
        } else {
            // If the types don't match, they are not equal.
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dyn_partial_ord::DynPartialOrd;
    use std::cmp::Ordering;

    trait MyTrait: DynPartialOrd {}

    #[derive(PartialEq, PartialOrd)]
    struct MyStructA {
        value: i32,
    }

    impl MyTrait for MyStructA {}

    #[derive(PartialEq, PartialOrd)]
    struct MyStructB {
        value: i32,
    }

    impl MyTrait for MyStructB {}

    #[test]
    fn partial_ord_works() {
        let a1: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let a2: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let b1: Box<dyn MyTrait> = Box::new(MyStructB { value: 1 });
        let b2: Box<dyn MyTrait> = Box::new(MyStructB { value: 2 });

        assert_eq!(a1.dyn_partial_cmp(a2.as_ref()), Some(Ordering::Equal));
        assert_eq!(a1.dyn_partial_cmp(b1.as_ref()), None);
        assert_eq!(b1.dyn_partial_cmp(b2.as_ref()), Some(Ordering::Less));
        assert_eq!(b2.dyn_partial_cmp(b1.as_ref()), Some(Ordering::Greater));
    }
}
