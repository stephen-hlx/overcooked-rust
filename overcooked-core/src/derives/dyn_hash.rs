use std::hash::{Hash, Hasher};

pub trait DynHash {
    fn dyn_hash(&self, hasher: &mut dyn Hasher);
}

impl<T: Hash> DynHash for T {
    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        Hash::hash(self, &mut hasher);
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hasher};

    use super::DynHash;

    trait MyTrait: DynHash {}

    #[derive(Hash)]
    struct MyStructA {
        value: i32,
    }

    impl MyTrait for MyStructA {}

    #[test]
    fn works() {
        let a: Box<dyn MyTrait> = Box::new(MyStructA { value: 1 });

        let mut hasher = DefaultHasher::new();
        a.dyn_hash(&mut hasher);
        let _ = hasher.finish();
    }
}
