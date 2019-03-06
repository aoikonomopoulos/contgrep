# `contgrep`: toy `grep` with line continuations

It's often the case that you run into a line-oriented log or trace format, in which newlines occasionally find their way in the middle of a record, resulting in output like

```
Jun 07 01:02:13 foo
Jun 07 01:02:14 bar
baz
Jun 07 01:02:15 quux
```

In this case, searching for `bar` will come up with a stray match, for which you'll need to dig up the context. Even worse, if you search for `bar`, you might not realize that the match you get back is incomplete.

The only correct solution here is to enforce proper escaping in your logging (or, even better, use a framework that does this for you). `contgrep` is a hack that helps in one-off problems where you'll never have to deal with this system again.

More interestingly, it's also a useful hack when embedding pretty-printed data objects in your logging statements, e.g.

```rust
debug!("sometag foo={:#?} bar={:#?}", foo, bar)
```

in languages like rust. Here, you're likely to get something like

```
DEBUG 2022-06-06T01:02:03Z: some::thingy sometag foo=Foo {
  field: Some(X)
} bar=bar {
  field: Y
}
```

(or, in my case, 20+ lines of output for each object, which would be unreadable crammed in a single line). Using

```
contgrep -e sometag -e some_other_tag -C "^DEBUG "
```

makes for an easy way forward until you can come up with some logging solution that allows easy selection and folding.

See [manpage](doc/contgrep.md) for more usage information.
