# cargo single-line

A simple cargo plugin that shrinks the visible cargo output to a single line
(okay, in the best case scenario).

In principle, the plugin works by intercepting the cargo's [stderr] and
replacing the [newline] characters in it with a [carriage return][carriage]
symbol.

When cargo prints a line which is not `Compiling`/`Checking`/etc., it probably
means an error/warning, so such a line is forwarded "as is" to the user so the
possibly useful output is not overwritten by the further data.

The tool can be used with any cargo subcommand, just insert `single-line`
between `cargo` and your command, like the following:

* `cargo build` → `cargo single-line build`,
* `cargo run` → `cargo single-line run`,
* `cargo clippy` → `cargo single-line clippy`,
* ... and so forth.

By default, when running from a terminal, the plugin enforces a colorful output
by running `cargo` with a `--color=always` argument. To override the behavior,
add an explicit `--color MODE` flag to your command line.

[![asciicast](https://asciinema.org/a/8PNaIBtegxlFb5RACVngEMQyK.svg)](https://asciinema.org/a/8PNaIBtegxlFb5RACVngEMQyK)

# Installation

To install the plugin from [crates.io][crates]:
```
$ cargo install cargo-single-line
```

To install the plugin from a checkout git repository:
```
$ cargo install --path .
```


[stderr]: https://en.wikipedia.org/wiki/Standard_error
[newline]: https://en.wikipedia.org/wiki/Newline
[carriage]: https://en.wikipedia.org/wiki/Carriage_return
[crates]: https://crates.io/
