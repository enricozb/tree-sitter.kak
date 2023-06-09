# tree-sitter.kak

> **Warning**
> This currently just a proof-of-concept, and isn't stable yet.

A [Tree-sitter][1] server that keeps parsed ASTs in memory. This is useful for generalizing plugins that
want to query the AST. Highlighting and structural selections are two main motivating examples.

## Highlighting Demo
[![asciicast](https://asciinema.org/a/1udaqC8SwHaJuyRKPjuOsqJe0.svg)](https://asciinema.org/a/1udaqC8SwHaJuyRKPjuOsqJe0)

### Usage
The package **does not** install a [configuration file](./config/config.toml) automatically. So you must
put one wherever you like and tell the plugin where that configuration file is:
```kak
set-option global tree_sitter_config <path_to_your_config>
```

Then, in buffers where you want to use `tree-sitter.kak`, call `tree-sitter-enable-buffer`. Furthermore,
if you want to enable highlighting, you should remove the default highlighters. For example, for rust:
```kak
hook buffer BufSetOption filetype=rust %{
  rmhl window/rust
  tree-sitter-enable-buffer
}
```

### Configuration
The configuration file currently only controls highlights, and maps tree-sitter captures to kakoune's faces.
For example, from [`config/config.toml`](./config/config.toml):
```toml
[language.rust.faces]
attribute = "meta"
comment = "comment"
function = "function"
keyword = "keyword"
operator = "operator"
string = "string"
type = "type"
type-builtin = "type"
constructor = "value"
constant = "value"
constant-builtin = "value"
```
The captures are defined in the languages [query file](src/languages/highlight/), which are currently shipped
with the library instead of being configurable.

### Supported Languages
Currently, only rust is supported. Other languages are easy to add though, adding them as a dependency
in the [`Cargo.toml`](./Cargo.toml) file and matching against them in [`languages/mod.rs`](./src/languages/mod.rs).

### TODO
- [ ] add more languages
- [ ] make the query files configurable
- [ ] autoload the default config
- [ ] add commands for querying the AST
- [ ] add documentation for commands & requests
- [ ] remove `new_buffer` and `set_language` commands, and send language along with
      buffer information for `parse_buffer`. if the buffer doesn't exist, it should
      just be created. this will simplify the request logic and make it more robust.

[1]: https://tree-sitter.github.io/tree-sitter/
