# Any-ref-rs
Please take look at [owning_ref](https://crates.io/crates/owning_ref) before using this crate, which maybe more simple, but can only contains reference. The following scene cannot be done using `owning_ref`.

```rust
// How to realize this???
fn bar(s:String)->Bytes<'_>{
    s.bytes()
}
```

## Example
```rust
use any_ref::{new_any_ref,Borrowed};

// Define struct `Foo`
#[derive(Debug)]
struct Foo<'a> {
    _borrowed_slice: &'a [u8],
    _borrowed_iterator: std::slice::Iter<'a, u8>,
}

// Define `ReturnFoo` and `ReturnBorrowedVec` using macro
make_any_ref! {
    struct ReturnFoo = for<'f> Foo<'f>;
    pub struct ReturnBorrowedVec<T:'static> = for<'a> Vec<&'a T>; // Define complex struct
}

let moved_ar;
{
    // Create a vector stores some numbers
    let q = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    // Store a `Foo` which borrows the vector
    let ar = new_any_ref::<ReturnFoo, _, _>(q, |s| Foo {
        _borrowed_slice: &s[..5],
        _borrowed_iterator: s.iter(),
    });

    // Move out of the scope
    moved_ar = ar;
}
println!("{:?}", moved_ar.get());

// Let's do again, but this time it's a slice
let moved_ar;
{
    // Create a vector stores some numbers
    let q = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    // `Borrow` is the replacement of slice, defined in the crate. Let's store a slice
    let ar = new_any_ref::<Borrowed<[u8]>, _, _>(q, |s| &s[3..]);

    // Let's re-borrow it (consumes the origin `ar`)
    let ar = ar.map::<Borrowed<[u8]>, _>(|s| &s[..3]);

    // Move out of the scope
    moved_ar = ar;
}
println!("{:?}", moved_ar.get()); // &[3, 4, 5]
```

### License
MIT
