use util::must_compile;

mod util;

#[test]
fn test0() {
    let s = " MAIN = () { 0 3 mstore here: 0xdead here jump 0x55 asdf 0 0 revert }
asdf  =  0xffff
";
    let b = "60006003525b61dead61000556605561ffff60006000fd";
    //            |          ^^      ^^
    assert_eq!(must_compile(s), b);
}

#[test]
fn test1() {
    let s = " MAIN = () { 0 3 mstore here: 0xdead here jump \"0x55\" \"asdf\" 0 0 revert }
asdf  =  0xffff
";
    let b = "60006003525b61dead6100055655ffff60006000fd";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test2() {
    let s = "
MAIN = () { 0 3 mstore here: 0xdead here jump \"0x55\" \"asdf\" 0 0 revert macro1(5,5) }
macro1 = (one,two) { one 0 mstore }
asdf  =  0xffff
";
    //             60006003525b61dead6100055655ffff60006000fd6105600052
    let b = "60006003525b61dead6100055655ffff60006000fd6005600052";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test3() {
    let s = "
MAIN = () { 0 3 mstore here: 0xdead here jump \"0x55\" \"asdf\" 0 0 revert macro1(5,5) }
macro1 = (one,two) { \"one\" 0 mstore }
asdf  =  0xff_ff
";

    let b = "60006003525b61dead6100055655ffff60006000fd05600052";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test4() {
    let s = "
MAIN = () {
    here:
    
    macro2(here,there)
    
    here jump
    there jump

    there:
        0 0 revert
}

asdf = 0xffff

macro2 = (something, error) { 
    0x03 0x04 mstore
    something
    error jumpi    
}
";
    let b = "5b60036004526100006100155761000056610015565b60006000fd";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test5() {
    let s = "
MAIN = () {
    macro2(error1)
    error1: 0 0 revert
}
macro2 = (error2) { macro3(error2) }
macro3 = (error3) { macro4(error3) }
macro4 = (error4) { macro5(error4) }
macro5 = (error5) {    error5 jump }
";
    let b = "610004565b60006000fd";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test6() {
    let s = "
result = (0xff + 0x01) / 2 

MAIN = () {
    0 macro2(0xff + 0x01)  mstore 
    result 0 sstore
}

macro2 = (param) {
    0 param mstore
}
";
    let b = "6000600061010052526080600055";

    println!("RUNNING TEST 6");
    assert_eq!(must_compile(s), b);
}

#[test]
fn test7() {
    let s = "
MAIN = () { const2 }
const2 = const1 - 0xff
const1  = 0xffff
";
    let b = "61ff00";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test8() {
    let s = "
MAIN = () { \"const2\" }
const2 = const1 - 0xff
const1  = 0xffff
";
    let b = "ff00";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test9() {
    let s = "
result = size(macro1(5))

MAIN = () {
    result size(macro2())
}

macro2 = () {
    0 4 mstore 
}

macro1 = (a) {
    0 a 0 
}
";
    let b = "60066005";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test10() {
    let s = "
result = size1 + size2
size2 = size1 - size(macro2(1,2))
size1 = size(macro2(1000,2000))

MAIN = () {
    size(macro2(1000,2000)) result sub 
}

macro2 = (n,m) {
    n m revert 
}
";
    let b = "6007600903";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test11() {
    let s = "
MAIN = () {
    macro2(here)
    here:
}

macro2 = (there) {
    macro3(there, where)
    where:
}

macro3 = (n, m) {
    m n mstore 
}
";
    let b = "610007610008525b5b";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test12() {
    let s = "
MAIN = () {
    macro2(here,there)
    macro3(there,here)

    here:
    there:
}

macro2 = (there, other) {
    macro3(there, where)
    where:
}

macro3 = (n, m) {
    m n mstore 
}
";
    let b = "61000761000f525b61000f610010525b5b";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test13a() {
    let s = "
MAIN = () { macro2(here) here: }
macro2 = (there) { macro3(there, here) here: }
macro3 = (m, n) { n m mstore }
";
    assert_eq!(must_compile(s), "610007610008525b5b");
}

#[test]
fn test13b() {
    let s1 = "
MAIN = () { macro2(here) here: }
macro2 = (there) { macro3(there, where) where: }
macro3 = (n, m) { m n mstore }
";

    let s2 = "
MAIN = () { macro2(here) here: }
macro2 = (there) { macro3(there, here) here: }
macro3 = (m, n) { n m  macro4() }
macro4 = () { mstore }
";

    assert_eq!(must_compile(s1), must_compile(s2));
}

#[test]
fn test14() {
    let s = "
result = size1 + size2
size2 = size1 - size(macro2(1,2))
size1 = size(macro2(1000,2000))

MAIN = () {
    \"size(macro2(1000,2000))\" \"result\" 
}

macro2 = (n,m) {
    n m revert 
}
";
    let b = "0709";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test15() {
    let s = "
jumptable1 = {
    a1 a2 a3
}

MAIN = () {
    1 2 swap1 mstore
    :a1: 
    :a2: :a3: 
}
";
    let b = "6001600290525b5b5b000600070008";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test16() {
    let s = "
TABLE_3 = { c3 c2 c1 }
TABLE_2 = { b3 b2 b1 }
TABLE_1 = { a3 a2 a1 }

MAIN = () {
    selfdestruct size(TABLE_1)
    selfdestruct start(TABLE_1) selfdestruct 
    :a1: :b1: :a2: :b2: :b3: :a3: :c1: :c2: :c3:
}
";
    let b = "ff6003ff61001dff5b5b5b5b5b5b5b5b5b0010000f000e000c000b0009000d000a0008";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test17() {
    let s = "
macro = (op) { op }

MAIN = () {
    macro(dup1)
}
";
    let b = "80";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test18() {
    let s = "
macro = (op) { op }
macro2 = (op) { macro(op) }

MAIN = () {
    \"0x00\" macro2(dup1)
}
";
    let b = "0080";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test19() {
    let s = "
macro = (a) { a there jump there: a }

MAIN = () {
    macro(1)
    macro(2)
}
";
    let b = "6001610006565b6001600261000f565b6002";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test20a() {
    let s = "
macro = (a)  { macro2(a) }
macro2 = (a) { macro3(a) }
macro3 = (a) { macro4(a) }
macro4 = (a) { macro5(a) }
macro5 = (a) { a }

MAIN = () {
    macro(dup1)
}
";
    let b = "80";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test20b() {
    let s = "
macro  = (a) { macro2(a) }
macro2 = (a) { \"0x00\" a }

MAIN = () {
     macro(dup1)
}
";
    let b = "0080";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test20c() {
    let s = "
macro  = (a) { macro2(a) }
macro2 = (a) { a a }

MAIN = () {
     macro(dup1)
}
";
    let b = "8080";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test20d() {
    let s = "
macro = (a) {   macro2(a) }
macro2 = (a) {  macro3(a) a }
macro3 = (a) {  macro4(a) }
macro4 = (a) {  macro5(a) a  }
macro5 = (a) { a }

MAIN = () {
    \"0x00\" macro(dup1)
}
";
    let b = "00808080";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test20e() {
    let s = "
macro = (a) { \"0x00\"  macro2(a) }
macro2 = (a) {  macro3(a) a }
macro3 = (a) {  macro4(a) }
macro4 = (a) {  macro5(a) a  }
macro5 = (a) { a }

MAIN = () {
    \"0x00\" macro(dup1)
}
";
    let b = "0000808080";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test21a() {
    let s = "
macro = (op) { macro2(op, dup2) }
macro2 = (a, b) { a b }

MAIN = () {
    macro(dup1)
}
";
    let b = "8081";
    assert_eq!(must_compile(s), b);
}

// TODO: this should work without the time machine   <<
#[test]
fn test21b() {
    let s = "
macro  = (op) { \"0x00\" macro2(op, dup2) }
macro2 = (a, b) {  a macro3(b) }
macro3 = (c) {  c }

MAIN = () {
    \"0x00\" macro(dup1)
}
";
    let b = "00008081";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test21c() {
    let s = "
macro = (op) { \"0x00\"  macro2(op) }
macro2 = (op) {  macro3(op, dup2) }
macro3 = (op,op2) {  macro4(op2,op) }
macro4 = (op2,op) {  macro5(op) op2  }
macro5 = (op) { op }

MAIN = () {
    \"0x00\" macro(dup1)
}
";
    let b = "00008081";
    assert_eq!(must_compile(s), b);
}

#[test]
fn test21d() {
    let s = "
macro = (a, b) { \"0x00\" macro2(a, b) macro2(b, a) }
macro2 = (b, a) {  a macro3(b) }
macro3 = (c) { c }

MAIN = () {
    macro(dup1, dup2) macro(dup2, dup1)
}
";
    let b = "00818080810080818180";
    assert_eq!(must_compile(s), b);
}

#[test]
#[ignore]
fn todos() {
    let mut oks = 0;
    let mut errs = 0;
    vec![
        "
        table = { a1 a2 a3 a4 }
        MAIN = () { (size(table) + 1) :a1: :a2: :a3: :a4: }
        ",
    ]
    .iter()
    .for_each(|t| {
        match std::panic::catch_unwind(|| {
            must_compile(t);
        }) {
            Ok(_) => oks += 1,
            Err(_) => errs += 1,
        };
    });

    if errs > 0 {
        panic!("todo tests: {} / {}", oks, oks + errs);
    }
}
