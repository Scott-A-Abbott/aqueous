# UsefulObject

- Maybe a useful object with substitutes can be implemented with an enum
- Maybe this has to be something that is done by the developer, not by the library
  - The difficulty comes in the useful object enum having the same interface as the primary object and substitute

```rust
    trait Write {
        type Error;
        fn write_messages() -> Result<(), Self::Error> {}
    }
    trait Substitute {
        type Substitute;
    }

    enum Writer 
    where
        T: Substitute + Write,
        T::Substitute: Write,
    {
        Useful(T),
        Substitute(T::Substitute)
    }

    impl Write for PgConnection { .. }
    impl Write for PgSubstitute { .. }
    
    impl Dependency for PgConnection {
        type Substitute = PgSubstitute;
    };

    impl Write for Writer

    let writer = Writer<PgConnection>;
```
