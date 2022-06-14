# ruff

An experimental EVM macro language inspired by [Huff](https://github.com/huff-language/huff)

this language doesn't require knowledge of either Rust or solidity

⚠️ This is a WIP(work in progress), not ALL test cases have been handled. Proceed with caution ⚠️ 


## 


## TODO:

- [x] slick CLI, subcommand to check it compiles
- [ ] output as:
    - [x] json
    - [x] runtime binary
    - [ ] deployment binary
    - [ ] pretty print opcodes
- [x] jump table sizes `TABLE_1  1[  ]  2[  ]  TABLE_2 32[  ] `
- [ ] complete testing
- [ ] better error messages
- [ ] sourcemap
- [ ] math with macro size inside a macro
- [ ] math with jumptable size and start
- [ ] Use 1 byte for jumpdest positions when possible
- [ ] doc comments, more lib friendly
- [ ] `assert()` & `assert_eq()` in code, compile time error
- [ ] Testing subcommand?
- [ ] codetables? (just a macro that only accepts literals?)
- [ ] decompiler?

if a jumptable label isn't used dont include in bytecode?

Some code will not compile due to cyclic dependencies, such as:
```
a = size(m())
m = () {
    a
}
```