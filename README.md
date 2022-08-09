
# croc-look

croc-look is a tool to make testing and debuging proc macros easier by these two features

1. Printing the implementation specific generated code to the console
2. Giving a real time live view of the output of the macro on rust code

croc-look allows you to narrow down your search and also provide real time view of the generated code

### Installation

croc-look requires a nightly toolchain _installed_, it does not need to be the default toolchain

```
rustup install nightly
```

### Flags

1. `--trait_impl` or `-t`: Defines the trait you want to see the expansion off, this is gonna be the trait your derive macro is implementing or it could be 
any other trait you want to see the expansion off. For example
```
#[derive(Clone, Debug, MyTrait)]
struct<T> {
  ...
}
```
So the value for this flag can be either `Clone`, `Debug` or whatever trait your `MyTrait` derive macro is implementing

2. `--impl_for` or `-i`: This helps you narrow down your search for a trait impl for the flag mentioned above. If you have multiple structs deriving your trait then you can do `croc-look --trait_impl Clone -i <your-struct-name>` and get the impl for the struct you want.

3. `--structure` or `-s`: If you want to expand a _particlar_ struct. This is useful when a macro is manupilating the struct itself, like adding fields, etc.

4. `--function` or `-f`: For expanding a function.

5. `--watch` or `-w`: This starts watching the directory/file you want to watch, this also opens up an interactive TUI which has support for **live reloading** changes as you do them in your proc-macro project. 

There is currently no way to narrow down searches for functions and structs for a particular module but I'll soon implement that, for now you can create a new project and use the macro there to test.

#### License

<sup>
Licensed under <a href="LICENSE-MIT">MIT license</a>.
</sup>
