use util::must_not_compile;

mod util;

#[test]
fn test0() {
    let s = "one = 0x1";

    must_not_compile(s);
}

#[test]
fn test1() {
    let s = " MAIN = () { 0 3 mstore here: 0xdead here jump 0x55 asdf 0 0 revert } ";

    must_not_compile(s);
}

#[test]
fn test2() {
    let s = "a = size(MAIN())
MAIN = () { a }";

    must_not_compile(s);
}

#[test]
fn test3() {
    let s = "size = () {  }
MAIN = () {  }";

    must_not_compile(s);
}

#[test]
fn test4() {
    let s = "start = () {  }
MAIN = () {  }";

    must_not_compile(s);
}

#[test]
fn test5() {
    let s = "MAIN = () { 1 }
MAIN = () { 2 }";

    must_not_compile(s);
}

#[test]
#[ignore]
fn test6() {
    let s = "MAIN = 1
MAIN = () { 2 }";

    must_not_compile(s);
}

#[test]
#[ignore]
fn test7() {
    let s = "const =  1 
const =  2
MAIN = () { 2 }";

    must_not_compile(s);
}

#[test]
fn test8() {
    let s = "
macro = (a) { there jump there: a: a }

MAIN = () {
    macro(1)
    macro(2)
}
";
    // unclear if a refers to arg or jumpdest
    must_not_compile(s);
}
#[test]
fn test9() {
    let s = "
macro = (a) { a 0 a:  }

MAIN = () {
    macro(1)
}
";
    // unclear if a refers to arg or jumpdest
    must_not_compile(s);
}
// jump tables
