use std::any::Any;
use std::cmp::Ordering;

pub trait DynOrd: Any {
    fn as_any(&self) -> &dyn Any;
    fn dyn_cmp(&self, other: &dyn DynOrd) -> Ordering;
}

// 3. Implement `DynPartialOrd` for any concrete type that will be used in trait objects.
impl<T: Any + Ord> DynOrd for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_cmp(&self, other: &dyn DynOrd) -> Ordering {
        // First, check if the other object has the same underlying concrete type.
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            // If the types match, use the standard `PartialEq` implementation.
            self.cmp(other)
        } else {
            self.type_id().cmp(&other.type_id())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dyn_ord::DynOrd;
    use std::cmp::Ordering;

    trait MyTrait: DynOrd {}

    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    struct MyStructA {
        value: i32,
    }

    impl MyTrait for MyStructA {}

    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    struct MyStructB {
        value: i32,
    }

    impl MyTrait for MyStructB {}

    #[test]
    fn order_of_different_types_is_determined_by_its_type_id() {
        let a: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let b: Box<dyn MyTrait> = Box::new(MyStructB { value: 1 });
        assert_eq!(a.dyn_cmp(b.as_ref()), (*a).type_id().cmp(&(*b).type_id()));
    }

    #[test]
    fn partial_ord_works() {
        let a1: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let a2: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });
        let b1: Box<dyn MyTrait> = Box::new(MyStructB { value: 1 });
        let b2: Box<dyn MyTrait> = Box::new(MyStructB { value: 2 });

        assert_eq!(a1.dyn_cmp(a2.as_ref()), Ordering::Equal);
        assert_eq!(b1.dyn_cmp(b2.as_ref()), Ordering::Less);
        assert_eq!(b2.dyn_cmp(b1.as_ref()), Ordering::Greater);
    }
}
