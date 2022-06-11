use std::collections::HashMap;
use std::ops::Deref;

use crate::context::Context;
use crate::error::Error;
use crate::macros::{push_n, Macro};
use crate::value::{apply, Literal, MacroCall, MathOps, SizeCall, Value};

pub fn compile(main: &str, c: &mut Context) -> Result<String, Error> {
    let main = main.trim();
    let vv = match c.macros.get(main) {
        None => return Err(Error::new(&format!("main macro `{}` not found", main))),
        Some(Macro::Macro { args, content, .. }) => {
            if args.is_empty() {
                content.to_vec()
            } else {
                return Err(Error::new("main macro shouldnt have args"));
            }
        }
        Some(_) => panic!("shouldn't happen"),
    };

    match compile_macro(":", 0, &vv, c, &HashMap::new(), &mut HashMap::new()) {
        Ok((s, h)) => {
            if h.is_empty() {
                Ok(s)
            } else {
                Err(Error::new(&format!("jumpdest not defined {:?}", h)))
            }
        }
        Err(e) => Err(e),
    }
}

fn compile_value(
    namespace: &str,
    idx: usize,
    macro_start_idx: usize,
    s: &mut String,
    v: Value,
    c: &mut Context,
    arg_values: &HashMap<String, Value>,
    jumpdests: &mut HashMap<String, (bool, usize)>,
    missed_labels: &mut Vec<(String, usize)>,
    time_machine: &mut HashMap<(String, usize), (String, Value)>,
) -> Result<(String, bool), Error> {
    let key = (format!("{:?}", v), idx.clone());
    let count = c.compiles_at_idx.get(&key).map(|v| *v).unwrap_or(0);
    c.compiles_at_idx.insert(key, count + 1);
    // TODO: this is a hack, instead decrement count when current scope ends, and error at > 1
    if count > 100 {
        return Err(Error::new("cyclic dependency"));
    }

    let tm_v = time_machine.get(&(namespace.to_string(), macro_start_idx + idx));

    match v {
        Value::Opcode(o) => Ok((Literal::from(o as usize).to_hex(), false)),
        Value::Literal(l) => Ok((l.to_hex(), true)),
        Value::Quote(vl) => Ok((
            compile_value(
                namespace,
                idx,
                macro_start_idx,
                s,
                *vl,
                c,
                arg_values,
                jumpdests,
                missed_labels,
                time_machine,
            )?
            .0,
            false,
        )),
        Value::MacroCall(MacroCall {
            name,
            args: m_arg_values,
        }) => {
            let (aa, content) = prepare_compile_macro(
                &name,
                &namespace,
                c,
                arg_values,
                m_arg_values.clone(),
                jumpdests,
                time_machine,
            )?;

            let mut time_machine = HashMap::<(String, usize), (String, Value)>::new();
            let res;
            'macro_loop: loop {
                res = match compile_macro(
                    &ns_push(&namespace, &name),
                    macro_start_idx + idx,
                    &content,
                    c,
                    &aa,
                    &mut time_machine,
                ) {
                    Ok((b, missing)) => {
                        for (label, i) in missing {
                            let orig_label = label.to_string();
                            let label = ns_up(&label);
                            let label = match aa.get(&ns_name(&label)) {
                                Some(al) => match al {
                                    Value::Label { name, .. } => ns_push(&ns_pop(&label), name),
                                    v => {
                                        time_machine.insert(
                                            (ns_pop(&orig_label), idx + macro_start_idx),
                                            (ns_name(&orig_label), v.clone()),
                                        );
                                        continue 'macro_loop;
                                    }
                                },
                                None => label.clone(),
                            };

                            insert_missed_label(missed_labels, &label, idx + i);
                        }

                        Ok((b, false))
                    }
                    Err(e) => panic!("error: compiling macro{} ", e),
                };
                break 'macro_loop;
            }
            res
        }
        Value::Label { name, .. } => {
            if let Some(v) = tm_v {
                return compile_value(
                    namespace,
                    idx,
                    macro_start_idx,
                    s,
                    v.1.clone(),
                    c,
                    arg_values,
                    jumpdests,
                    missed_labels,
                    time_machine,
                );
            }

            match c.resolve_constant(&name) {
                Some(Value::Literal(l)) => Ok((l.to_hex(), false)),
                Some(v) => compile_value(
                    namespace,
                    idx,
                    macro_start_idx,
                    s,
                    v,
                    c,
                    arg_values,
                    jumpdests,
                    missed_labels,
                    time_machine,
                ),
                None => match compile_label(
                    namespace,
                    idx,
                    macro_start_idx,
                    &name,
                    c,
                    arg_values,
                    jumpdests,
                    missed_labels,
                    time_machine,
                ) {
                    Err(e) => return Err(e),
                    Ok(res) => Ok(res),
                },
            }
        }
        Value::JumpDest { name } => {
            let ns_n = ns_push(namespace, &name);
            jumpdests.insert(ns_n.to_string(), (false, idx + macro_start_idx));

            while let Some(i) = missed_labels
                .iter()
                .enumerate()
                .find(|(_, m)| m.0 == ns_n)
                .map(|m| m.0)
            {
                let j = missed_labels[i].1;
                let hex = push_n(2)?.to_hex() + &to_jumpdest(idx + macro_start_idx);
                let before = s[..j * 2].to_string();
                let after = s[j * 2 + 6..].to_string();
                assert_eq!(hex.len(), 6);
                *s = vec![before, hex.to_string(), after].join("");
                missed_labels.remove(i);
            }

            Ok(("5b".to_string(), false))
        }
        Value::JumpTableDest { name } => {
            if c.resolve_jumptabledest(&name).is_some() {
                return Err(Error::new(&format!("jumptable dest reused: `{}`", name)));
            }

            c.jumptabledests.insert(name, idx + macro_start_idx);
            Ok(("5b".to_string(), false))
        }
        Value::MathOp { op, left, right } => {
            let a = compile_mathop(
                &op,
                namespace,
                c,
                arg_values,
                macro_start_idx + idx,
                jumpdests,
                &left,
                &right,
                time_machine,
            )?;
            Ok((a, true))
        }

        Value::SizeCall(sc) => match sc {
            SizeCall::Macro(m) => {
                let (args, content) = prepare_compile_macro(
                    &m.name,
                    &namespace,
                    c,
                    arg_values,
                    m.args.clone(),
                    jumpdests,
                    time_machine,
                )?;
                match compile_macro(
                    namespace,
                    macro_start_idx + idx,
                    &content,
                    c,
                    &args,
                    time_machine,
                ) {
                    Ok((a, ..)) => Ok((Literal::from(a.len() / 2).to_hex(), true)),
                    Err(e) => Err(Error::new(&format!("compile maro sizecall error {}", e))),
                }
            }
            SizeCall::JumpTable { name } => match c.jumptables.iter().find(|j| j.0 == name) {
                Some((_, labels, _)) => Ok((to_hex(labels.len()), true)),
                None => Err(Error::new(&format!("jumptable `{}` not found", name))),
            },
        },
        Value::StartCall(sc) => {
            if let Some(jts) = c.jumptablestarts.get_mut(&sc.name) {
                jts.push(idx + macro_start_idx);
            } else {
                c.jumptablestarts.insert(sc.name, vec![idx + 1]); // TODO: + 1 only if it's being pushed
            }

            Ok(("????".to_string(), true))
        }

        _ => panic!("compile not implemented for {:?}", v),
    }
}

fn append_macro_start(m: &str, i: usize) -> String {
    if m.ends_with("|0") {
        panic!("!!   {}  {} ", m, i);
    }

    match m {
        ":" => ":".to_string(),
        m => format!("{}|{}", m, i),
    }
}

fn compile_label(
    namespace: &str,
    idx: usize,
    macro_start_idx: usize,
    name: &str,
    c: &mut Context,
    arg_values: &HashMap<String, Value>,
    jumpdests: &mut HashMap<String, (bool, usize)>,
    missed_labels: &mut Vec<(String, usize)>,
    time_machine: &mut HashMap<(String, usize), (String, Value)>,
) -> Result<(String, bool), Error> {
    if let Some(d) = jumpdests.get_mut(&ns_push(&namespace, &name)) {
        let b = to_jumpdest(d.1.to_owned());

        if b.len() != 4 {
            return Err(Error::new(&format!(
                "jump dest size != 2 {} {}",
                name,
                b.len() / 2
            )));
        }
        d.0 = true;
        return Ok((b, true));
    }

    if let Some(v) = arg_values.get(name) {
        return match v {
            Value::Opcode(o) => Ok((Literal::from(*o as usize).to_hex(), false)),
            Value::Literal(l) => Ok((l.to_hex(), true)),
            Value::Quote(vl) => match vl.deref() {
                Value::Literal(l) => Ok((l.to_hex(), false)),
                Value::Opcode(o) => Ok((Literal::from(*o as usize).to_hex(), false)),
                v => panic!("compile quote not supported for {:?}", v),
            },
            Value::Label { name, ns } => {
                insert_missed_label(
                    missed_labels,
                    &match ns {
                        None => ns_push(&namespace, name),
                        Some(ns) => ns_push(ns, name),
                    },
                    idx,
                );

                Ok(("????".to_string(), true))
            }
            Value::MathOp { op, left, right } => {
                match compile_mathop(
                    &op,
                    namespace,
                    c,
                    arg_values,
                    macro_start_idx + idx,
                    jumpdests,
                    &left,
                    &right,
                    time_machine,
                ) {
                    Ok(b) => Ok((b, true)),
                    Err(e) => Err(e),
                }
            }
            _ => Err(Error::new("failed to parse args")),
        };
    }

    let new_name = ns_push(&namespace, &name);
    insert_missed_label(missed_labels, &new_name, idx);

    Ok(("f1f1".to_string(), true))
}

fn insert_missed_label(missed: &mut Vec<(String, usize)>, label: &str, idx: usize) {
    assert!(
        label.chars().filter(|c| *c == ':').count() - label.chars().filter(|c| *c == '|').count()
            <= 1
    );

    if !missed.iter().any(|(s, i)| s == label && *i == idx) {
        missed.push((label.to_string(), idx));
    }
}

fn prepare_compile_macro(
    name: &str,
    namespace: &str,
    c: &Context,
    parent_arg_valuse: &HashMap<String, Value>,
    arg_values: Option<Vec<Value>>,
    jumpdests: &mut HashMap<String, (bool, usize)>,
    time_machine: &HashMap<(String, usize), (String, Value)>,
) -> Result<(HashMap<String, Value>, Vec<Value>), Error> {
    let (name, args, content) = match c.macros.get(name) {
        Some(Macro::Macro {
            name,
            args,
            content,
            ..
        }) => (name, args, content),
        _ => return Err(Error::new(&format!("macro not found `{}`", name))),
    };

    let arg_values = match arg_values {
        None => vec![],
        Some(v) => v.to_vec(),
    };

    if args.len() != arg_values.len() {
        return Err(Error::new(&format!(
            "marco `{}` called with {} arguments {} expected",
            name,
            arg_values.len(),
            args.len()
        )));
    }

    let mut aa = HashMap::new();
    for a in args.iter().enumerate() {
        // TODO: timemachine should be vec not hashmap, can have multiple at the same index

        let mut found = false;

        let mut val = if let Some((_, q)) = time_machine.iter().find(|(_, (s, v))| s == a.1) {
            found = true;
            q.1.clone()
        } else {
            arg_values[a.0].clone()
        };

        if !found {
            if let Value::Label { name, .. } = &val {
                if let Some((_, q)) = time_machine.iter().find(|(_, (s, v))| s == name) {
                    val = q.1.clone();
                    found = true;
                } else {
                    match parent_arg_valuse.get(name) {
                        None => {}
                        Some(Value::Label { name: n2, .. }) => {
                            val = Value::Label {
                                name: n2.to_string(),
                                ns: Some(namespace.to_string()),
                            };
                            found = true;
                        }
                        Some(v) => {
                            val = v.clone();
                            found = true;
                        }
                    };
                }
            }
        }

        aa.insert(a.1.to_string(), val);

        if let Value::Label { name, .. } = &arg_values[a.0] {
            if let Some(jd) = jumpdests.get_mut(name) {
                jd.0 = true;
            }
        }
    }

    Ok((aa, content.clone()))
}

fn compile_macro(
    namespace: &str,
    start_idx: usize,
    vv: &Vec<Value>,
    c: &mut Context,
    arg_values: &HashMap<String, Value>,
    time_machine: &mut HashMap<(String, usize), (String, Value)>,
) -> Result<(String, Vec<(String, usize)>), Error> {
    let namespace = append_macro_start(namespace, start_idx);

    let mut s = String::new();
    let mut idx = 0;

    // todo: use 1 byte for jumpdest index if possible

    let mut jumpdests = HashMap::<String, (bool, usize)>::new();
    let mut missed_labels = Vec::<(String, usize)>::new();

    let push = |s: &mut String, idx: &mut usize, b: &str| {
        s.push_str(b);
        *idx = *idx + b.len() / 2;
    };

    for v in vv.iter() {
        match compile_value(
            &namespace,
            idx,
            start_idx,
            &mut s,
            v.clone(),
            c,
            arg_values,
            &mut jumpdests,
            &mut missed_labels,
            time_machine,
        ) {
            Ok((b, true)) => push(
                &mut s,
                &mut idx,
                &(push_n(b.len() / 2).unwrap().to_hex() + &b),
            ),
            Ok((b, false)) => push(&mut s, &mut idx, &b),
            Err(e) => return Err(e),
        }
    }

    let mut not_missed = vec![];
    for (label, i) in &missed_labels {
        if let Some((touched, jdidx)) = jumpdests.get_mut(label.as_str()) {
            let hex = push_n(2)?.to_hex() + &to_jumpdest(*jdidx);
            // TODO:   only replace the actual 2 bytes of the label
            let (before, after) = (&s[..*i * 2], &s[*i * 2 + 6..]);
            s = before.to_string() + &(hex + after);
            *touched = true;
            not_missed.push(label.to_string());
        }
    }

    not_missed.iter().for_each(|l| {
        while let Some(i) = missed_labels
            .iter()
            .enumerate()
            .find(|(_, m)| &m.0 == l)
            .map(|m| m.0)
        {
            missed_labels.remove(i);
        }
    });

    if namespace == ":" {
        for (jt_name, labels, position) in c.jumptables.iter_mut() {
            *position = Some(s.len() / 2);
            for k in labels {
                let dest = match c.jumptabledests.get(k) {
                    Some(i) => to_jumpdest(*i),
                    None => {
                        return Err(Error::new(&format!(
                            "jumptable error table={} label={}",
                            jt_name, k
                        )))
                    }
                };
                s += &dest;
            }
        }

        for (k, vv) in &c.jumptablestarts {
            let dest = match c.jumptables.iter().find(|j| &j.0 == k) {
                Some((_, _, Some(i))) => to_jumpdest(*i),
                Some((_, _, None)) => {
                    return Err(Error::new(&format!("jumptable start error {}", k)))
                }
                None => return Err(Error::new(&format!("jumptable start error {}", k))),
            };
            for v in vv {
                let (before, after) = (&s[..*v * 2], &s[*v * 2 + 4..]);
                s = before.to_string() + &(dest.clone() + after);
            }
        }
    }

    Ok((s, missed_labels))
}

fn compile_mathop(
    op: &MathOps,
    namespace: &str,
    c: &mut Context,
    parent_arg_values: &HashMap<String, Value>,
    idx: usize,
    jumpdests: &mut HashMap<String, (bool, usize)>,
    left: &Value,
    right: &Value,
    time_machine: &mut HashMap<(String, usize), (String, Value)>,
) -> Result<String, Error> {
    fn eval_mathop(
        op: MathOps,
        namespace: &str,
        parent_arg_values: &HashMap<String, Value>,
        idx: usize,
        c: &mut Context,
        jumpdests: &mut HashMap<String, (bool, usize)>,
        left: Value,
        right: Value,
        time_machine: &mut HashMap<(String, usize), (String, Value)>,
    ) -> Result<Literal, Error> {
        match (&left, &right) {
            (Value::Literal(l), Value::Literal(r)) => apply(&op, &l, &r),
            _ => apply(
                &op,
                &eval(
                    left,
                    namespace,
                    parent_arg_values,
                    idx,
                    c,
                    jumpdests,
                    time_machine,
                )?,
                &eval(
                    right,
                    namespace,
                    parent_arg_values,
                    idx,
                    c,
                    jumpdests,
                    time_machine,
                )?,
            ),
        }
    }
    fn eval(
        v: Value,
        namespace: &str,
        parent_arg_values: &HashMap<String, Value>,
        idx: usize,
        c: &mut Context,
        jumpdests: &mut HashMap<String, (bool, usize)>,
        time_machine: &mut HashMap<(String, usize), (String, Value)>,
    ) -> Result<Literal, Error> {
        match v {
            Value::MathOp { op, left, right } => eval_mathop(
                op,
                namespace,
                parent_arg_values,
                idx,
                c,
                jumpdests,
                *left,
                *right,
                time_machine,
            ),
            Value::Literal(l) => Ok(l),
            Value::Constant { value, .. } => Ok(eval(
                *value,
                namespace,
                parent_arg_values,
                idx,
                c,
                jumpdests,
                time_machine,
            )?),
            Value::Label { name, .. } => match &jumpdests.get_mut(&name) {
                Some((mut b, s)) => {
                    b = true;
                    Ok(Literal::from(*s))
                }
                None => match c.resolve_constant(&name) {
                    None => Err(Error::new(&format!("not found: `{}`  {:?}", name, c))),
                    Some(v) => Ok(eval(
                        v,
                        namespace,
                        parent_arg_values,
                        idx,
                        c,
                        jumpdests,
                        time_machine,
                    )?),
                },
            },
            Value::SizeCall(s) => match s {
                SizeCall::Macro(mc) => eval(
                    Value::MacroCall(mc),
                    namespace,
                    parent_arg_values,
                    idx,
                    c,
                    jumpdests,
                    time_machine,
                ),
                _ => panic!("not impl"),
            }, // <<<<< TODO: hack, math op shouldn't contain macrocall, use sizecall with only left, no right
            Value::MacroCall(m) => {
                // TODO: what if macro not known here? can that even happen?
                let (aa, content) = prepare_compile_macro(
                    &m.name,
                    namespace,
                    c,
                    parent_arg_values,
                    m.args,
                    jumpdests,
                    time_machine,
                )?;
                Ok(Literal::from(
                    compile_macro(":", idx, &content, c, &aa, time_machine)?
                        .0
                        .len()
                        / 2,
                ))
            }

            v => Err(Error::new(&format!(
                "math not supported for type of value {:?}",
                v
            ))),
        }
    }

    Ok(eval_mathop(
        op.clone(),
        namespace,
        parent_arg_values,
        idx,
        c,
        jumpdests,
        left.clone(),
        right.clone(),
        time_machine,
    )?
    .to_hex())
}

pub fn to_hex(i: usize) -> String {
    let mut s = format!("{:x}", i);
    if s.len() % 2 == 0 {
        s
    } else {
        "0".to_string() + &s
    }
}

fn to_jumpdest(i: usize) -> String {
    let mut s = format!("{:x}", i);
    if s.len() < 4 {
        s = "0".repeat(4 - s.len()) + &s;
    }
    s
}

fn ns_pop(ns: &str) -> String {
    if ns.len() < 2 || ns.chars().nth(0).unwrap() == ':' && !ns[1..].chars().any(|x| x == ':') {
        return ":".to_string();
    }

    let lastcolon = ns
        .chars()
        .rev()
        .enumerate()
        .find(|c| c.1 == ':')
        .map(|x| ns.len() - x.0 - 1)
        .unwrap();

    return ns[..lastcolon].to_string();
}

fn ns_push(ns: &str, n: &str) -> String {
    let ns = ":".to_string()
        + &ns
            .split(":")
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(":");

    match ns.chars().rev().nth(0).unwrap_or(' ') {
        ':' => ns + n,
        _ => ns + ":" + n,
    }
}

fn ns_up(ns: &str) -> String {
    let ns = &ns
        .split(":")
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    ":".to_string()
        + &match ns.len() {
            0 => "".to_string(),
            1 => ns[0].to_string(),
            2 => ns[1].to_string(),
            _ => {
                let last = &ns[ns.len() - 1];
                let ns = &ns[..ns.len() - 2].join(":");
                ns.to_string() + if ns.len() == 0 { "" } else { ":" } + last
            }
        }
}

fn ns_name(s: &str) -> String {
    let mut s1 = s.to_string();
    let mut s2 = ns_up(&s1);
    while s1 != s2 {
        s1 = s2.to_string();
        s2 = ns_up(&s2);
    }
    s2[1..].to_string()
}

#[test]
fn test_ns() {
    assert_eq!(ns_push(":a:", "b"), ":a:b");
    assert_eq!(ns_push("::a:", "b"), ":a:b");
    assert_eq!(ns_push(":a:b", "c"), ":a:b:c");
    assert_eq!(ns_push(":", "c"), ":c");
    assert_eq!(ns_pop(":a:b:c"), ":a:b");
    assert_eq!(ns_pop(":a:b"), ":a");
    assert_eq!(ns_pop(":a"), ":");

    assert_eq!(ns_up(":a:b"), ":b");
    assert_eq!(ns_up(":a:b:c"), ":a:c");
    assert_eq!(ns_up(":a:b:c:d:e:f"), ":a:b:c:d:f");
    assert_eq!(ns_up(":"), ":");
    assert_eq!(ns_up(":b"), ":b");

    assert_eq!(ns_name(":a:b"), "b");
    assert_eq!(ns_name(":a:b:c"), "c");
    assert_eq!(ns_name(":a:b:c:d:e:f"), "f");
    assert_eq!(ns_name(":"), "");
    assert_eq!(ns_name(":b"), "b");
}
