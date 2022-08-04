
croc-look is a tool to make testing and debuging proc macros easier by these two features

1. Printing the implementation specific generated code to the console
2. Giving a real time live view of the output of the macro on rust code

croc-look is built on top of cargo-expand, it allows you to narrow down your search and provides
a real time view of the generated code

# Installation

croc-look requires

1. Nightly
2. cargo-expand

To be installed, nightly does not need to be the default toolchain, `croc-look` uses
cargo-expand underneath which uses nightly.

#### License

<sup>
Licensed under <a href="LICENSE-MIT">MIT license</a>.
</sup>
