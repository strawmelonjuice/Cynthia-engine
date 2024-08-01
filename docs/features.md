# Features

The Rust language, in which Cynthia was written, has the
ability to compile certain features on-demand only.

This means that the binary size of the compiled program
can be reduced by only compiling the features that are
needed.

## Node

> If disabled no node (bun) runtime is needed.

Node is a default feature. It can be disabled by running the
compiler with the `--no-default-features` flag.

The `node` feature allows Cynthia to offload some parts of
it's rendering process to Node. This enables some features like advanced
Handlebars templating.
