use any_ref::{make_any_ref, AnyRef};
use std::rc::Rc;

make_any_ref! {
    type Bytes=for<'a> &'a [u8];
}

fn main() {
    let bytes: Rc<[u8]> = Rc::from((vec![1u8, 2, 3, 4, 5, 6]).into_boxed_slice());
    let mut first_half: Option<AnyRef<Bytes, Rc<[u8]>>> = None;
    let mut second_half: Option<AnyRef<Bytes, Rc<[u8]>>> = None;
    let mut _dropped_val: Option<AnyRef<Bytes, Rc<[u8]>>> = None;
    let _short_lived_bytes = vec![1, 2, 3];

    any_ref::build(bytes.clone(), |array, mut builder| {
        let split_at = array.len() / 2;

        first_half = Some(builder.build::<Bytes>(&array[..split_at]));
        second_half = Some(builder.build::<Bytes>(&array[split_at..]));
        //_dropped_val = Some(builder.build::<Bytes>(&*_short_lived_bytes));
    });

    println!("{}", Rc::strong_count(&bytes));
    //println!("{:?}", _dropped_val.unwrap().get());
    assert_eq!(first_half.unwrap().get(), &[1, 2, 3]);
    assert_eq!(second_half.unwrap().get(), &[4, 5, 6]);
}
