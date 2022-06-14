## Language Specification

First and foremost ruff supports both single line `//` and multiline `/*` `*/` comments.
You'll be using these a lot!

Ruff has 5 value types and 3 object types:

| Value Type              | Syntax                          | Behaviour                                                                                                                                       | 
|-------------------------|---------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| Opcodes                 | `swap1` `mul` `calldataload`    | Opcodes compile to their respective bytecodes                                                                                                   |
| Literals                | `1337` `0x0007cf` `0x_01_02_03` | As in huff, the `push` opcodes don't exist, literals are prefixed with the appropriate `push` opcode; leading zeros in hex values are preserved |
| Jump destinations       | `error:`                        | Defining a jump destination inserts a `jumpdest` opcode                                                                                         |
| Labels                  | `error`                         | Labels push the position of the corresponding jump destination                                                                                  |
| Jump table destinations | `:error:`                       | Same as jump destinations, but accessed through a jumptable compiled at the end of the bytecode                                                 |

| Object Type | Syntax                                      | Behaviour                                                                                                       | 
|-------------|---------------------------------------------|-----------------------------------------------------------------------------------------------------------------|
| Constant    | `c = 0x12` `a = size(main) - 32`            | Constants are defined at the top level and must be knowable at compile time                                     |
| Macro       | `m = (p) { 0x20 p return }`                 | Macros are templates that help reuse code by duplicating it, there must be a main macro that takes no arguments |
| Jump Table  | `t = { approve transfer allowance \*..*\ }` | Jump Tables are compiled at the end of the contract                                                             |

### Constants

Constants are defined  

### Quote operator

Since ruff automatically prepends `push` opcodes before literals, it provides an operator to disable that behaviour.   
So `"0x6000"` compiles to itself (`push1` `0x00`) whereas without the quotes, it compiles to `616000` (`push2 0x6000`).  
Quotes can also be used around labels, expressions, constants

### Macros

Define macro: `transfer = (token, to, amount) { /* ... */ }`  
Use macro: `transfer(token, to, amount)`  
macros expand with arguments for templating

Context in macros

example:
```
main = () {
  // macro body
}
```

* calling macros 
* arguments can include:
  * literals
  * opcodes, labels and math operations between those

### Size and start functions
* `size()` called on macros and jumptables, compiles to the number of bytes that the macro/jumptable would compile into
* `start()` called on jumptables compiles to the bytecode index where the jumptable starts 
* both size and start can be used in:
  * math operations
  * macro bodies
  * arguments when calling macros
  * constant definitions


### Math
Supported operators: `+` `-` `*` `/`

### Jumptables



---
