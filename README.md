
# croc-look

croc-look is a tool to make testing and debuging proc macros easier by these two features

1. Printing the output of the procedural macro code to the console.
2. Giving a real time live view of the output of the macro on rust code.

croc-look allows you to narrow down your search and also provide real time view of the generated code.

[Blog post](https://dakshu.xyz/blog/dpmucl.html)

### Installation

croc-look requires a nightly toolchain _installed_, it does not need to be the default toolchain

```
rustup install nightly
```
Then
```
cargo install croc-look
```

### Flags

1. `--trait-impl` or `-t`: Defines the trait you want to see the expansion off, this is gonna be the trait your derive macro is implementing or it could be
any other trait you want to see the expansion off. For example
```
#[derive(Clone, Debug, MyTrait)]
struct<T> {
  ...
}
```
So the value for this flag can be either `Clone`, `Debug` or whatever trait your `MyTrait` derive macro is implementing

2. `--impl-for` or `-i`: This helps you narrow down your search for a trait impl for the flag mentioned above. If you have multiple structs deriving your trait then you can do `croc-look --trait-impl Clone -i <your-struct-name>` and get the impl for the struct you want.

3. `--structure` or `-s`: If you want to expand a _particlar_ struct. This is useful when a macro is manupilating the struct itself, like adding fields, etc.

4. `--path` or `-p`: (requies [cargo expand](https://github.com/dtolnay/cargo-expand) to be installed) Use `cargo expand <path>` internally to narrow down code to modules. eg `croc-look -p cmd -t Clone -i Context` (This finds the impl Clone for Context in cmd module)

5. `--function` or `-f`: For expanding a function.

6. `--binary` or `-b`: To expand a `cargo --bin BINARY`, if not specified then `--lib` is used

7. `--watch` or `-w`: This starts watching the directory/file you want to watch, this also opens up an interactive TUI which has support for **live reloading** changes as you do them in your proc-macro project. 

### How is this different from [cargo expand](https://github.com/dtolnay/cargo-expand)?
cargo expand doesn't allow you to view a whole trait impl to check generics or watch particular code blocks. The motive of croc-look is to narroy arry down your search to a simple single body and reduce cluter. 

You can use the `--path` flag to use `cargo expand <path>` to narrow down searches module level


#### License

<sup>
Licensed under <a href="LICENSE-MIT">MIT license</a>.
</sup>
