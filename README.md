# Any-ref-rs

### :book: Documentation
**[documentation](https://docs.rs/any_ref/latest/any_ref/)**

### :question: What is any_ref?
`any_ref` is a crate that allows you to move a struct with lifetime annotation together with its owner anywhere, such as returning it from a function.

Here's a simple demo to see how it works.
```rust
// Create the type `ReturnVec` stands for `Vec<&'_ T>`
make_any_ref! {
    type ReturnVec<T: 'static> = for<'a> Vec<&'a T>;
}

let moved_ar;
{
    // This is the owner
    let num: Box<(u16, u16, u16, u16)> = Box::new((1, 2, 3, 4));

    // Initialize an `AnyRef`
    let ar = new_any_ref::<ReturnVec<u16>, _, _>(
        num, |x| vec![&x.0, &x.1, &x.2, &x.3]
    );

    // Move out of this scope
    moved_ar = ar;
}

// Read the reference of the value
assert_eq!(moved_ar.get(), &vec![&1, &2, &3, &4]);
```

### :mega: Notice
At present version(rustc 1.60.0), due to a problem with type inferer, we cannot use `AnyRef::new` to initialize `AnyRef` in stable rust now, but it can in nightly. Thus we use `new_any_ref` as an temporary alternative solution. *This problem is expected to be settle in ructc 1.61.0*.

`AnyRef::new` expected to be the formal way to initialize `AnyRef` in the future, *use `AnyRef::new` if possible*.

### :confused: Troubleshooting
Everyone is welcomed to find out bugs or put forward your ideas of improvement, and please feel free to open an issue if you have any questions.

This crate is maintained.

### :page_facing_up: License
MIT

### :heart: Acknowledgements
Thanks to people in the rust forum who helped me find out serveral problems and bugs, thanks for your effort!