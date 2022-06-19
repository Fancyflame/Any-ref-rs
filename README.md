# Any-ref-rs

### 📖 Documentation
**[documentation](https://docs.rs/any_ref/latest/any_ref/)**

### ❓ What is any_ref
`any_ref` is a crate that allows you to capture things that are **not** `'static` with memory safety.

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
    let ar: AnyRef<ReturnVec<u16>, _> = AnyRef::new(
        num, |x| vec![&x.0, &x.1, &x.2, &x.3]
    );

    // Move out of this scope
    moved_ar = ar;
}

// Read the reference of the value
assert_eq!(moved_ar.get(), &vec![&1, &2, &3, &4]);
```

### 📣 Compatibility Notice
Below the version of rustc 1.61, due to a problem with type inferer, we cannot use `AnyRef::new` to initialize `AnyRef` in stable rust,
using `new_any_ref` as an temporary alternative solution.
**Please upgrade your rustc to 1.61.0 or later if possible!**

### 🤔 Troubleshooting
Everyone is welcomed to find out bugs or put forward your ideas of improvement, and please feel free to open an issue if you have any questions.

This crate is maintained (Written in 2022/6/19).

### ⚖️ License
MIT

### ❤️ Acknowledgements
Thanks to people in the rust forum who helped me find out serveral problems and bugs, thanks for your effort!