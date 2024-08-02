# Features

The Rust language, in which Cynthia was written, has the
ability to compile certain features on-demand only.

This means that the binary size of the compiled Cyntia 
can be reduced by only compiling the features that are
needed.

## List of features currently available

### JS runtime environment server: `js_runtime`

> [!TIP]  
> If disabled no Bun or Node runtime is needed.

> [!NOTE]  
> `js_runtime` is a default feature. It can be disabled by running the
> compiler with the `--no-default-features` flag.

#### Functionality

The `js_runtime` feature allows Cynthia to offload some parts of
it's rendering process to Node. This enables some features like advanced
Handlebars templating.

It is also a necessary part of Lumina for plugins written in JS. This is currently the only kind of plugins available, so this will mean Cynthia is no longer able to operate with plugins.
