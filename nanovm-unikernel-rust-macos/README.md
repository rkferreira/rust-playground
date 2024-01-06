# NanoVms Unikernels

The intent here was to play with nanovms unikernels and rust.

Rust code is merely a copy from some place.

I'm running om MacOs, so I had to crosscompile it to elf so I could run using ops.

[Ops Nanovms](https://ops.city/)

MacOs curl installation works fine.


## Cross compiling rust to elf

[cross-compiling-rust-on-macos-to-run-as-a-unikernel](https://hackernoon.com/cross-compiling-rust-on-macos-to-run-as-a-unikernel-ff1w3ypi)

Two ways, the one that worked for was using musl static linked.

```
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu

brew install FiloSottile/musl-cross/musl-cross

```


```
:~ rkferreira$ cat ~/.cargo/config
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.x86_64-unknown-linux-gnu]
linker = "x86_64-unknown-linux-gnu-gcc"

```


```
TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

TARGET_CC=x86_64-unknown-linux-gnu cargo build --release --target x86_64-unknown-linux-gnu

```


```
ops run -p 80 target/x86_64-unknown-linux-musl/release/hello_world

```
