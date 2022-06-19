use any_ref::{AnyRef, Reference};
use std::rc::Rc;

type Bytes = Reference<[u8]>;

fn main() {
    let bytes: Rc<[u8]> = Rc::from((vec![1u8, 2, 3, 4, 5, 6]).into_boxed_slice());
    let mut _dropped_val: Option<AnyRef<Bytes, Rc<[u8]>>> = None;
    let _short_lived_bytes = vec![1, 2, 3];

    let (first_half, second_half) = any_ref::build(bytes.clone(), |array, mut builder| {
        let split_at = array.len() / 2;
        (
            builder.build::<Bytes>(&array[..split_at]),
            builder.build::<Bytes>(&array[split_at..]),
            // try uncomment the following code and comment the above, you'll find it cannot pass the compilation
            // builder.build::<Bytes>(&*_short_lived_bytes),
        )
    });

    assert_eq!(first_half.get(), &[1, 2, 3]);
    assert_eq!(second_half.get(), &[4, 5, 6]);
}
