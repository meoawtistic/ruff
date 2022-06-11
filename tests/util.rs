use ruff::compile;
use ruff::parse_top_level;

#[cfg(test)]
pub fn must_compile(input: &str) -> String {
    let mut c = match parse_top_level(input) {
        Err(e) => panic!("parse error {:?}", e),
        Ok(c) => c,
    };

    match compile("MAIN", &mut c) {
        Err(e) => panic!("compile macro failed {}", e),
        Ok(compiled) => compiled,
    }
}

#[cfg(test)]
pub fn must_not_compile(input: &str) {
    match parse_top_level(input) {
        Err(_) => {}
        Ok(mut c) => match compile("MAIN", &mut c) {
            Err(_) => {}
            Ok(compiled) => panic!("should not compile, gave: {}", compiled),
        },
    }
}
