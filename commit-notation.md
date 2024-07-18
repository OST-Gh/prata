# Notation:
- Each commit header is `[[read description]]`
- KW.MOVE: Renames–Moves all types of name/location manipulation.
- KW.REMOVE: Deletion of element.
- KW.CREATE: Creation of element.
- KW.BREAK: Known Compilation/Interpretation error source
```
[[read description]]

move = { -- whole block comment
  foo.rs -> baz::bar.py;
  qux.rs::struct.SomeType -> qux.rs::mod.sub::struct.SomeType;
}
create*break = fn.volatile -- comment.
```

Both the src and . directories can be omitted.
