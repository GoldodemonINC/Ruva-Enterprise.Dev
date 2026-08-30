






use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn run_vm(source: &str) -> (String, String) {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!(
        "vm_tests_{}_{}.ruva",
        std::process::id(),
        id
    ));
    std::fs::write(&path, source).expect("write temp .ruva");

    let out = Command::new(env!("CARGO_BIN_EXE_ruva"))
        .arg("vm")
        .arg(&path)
        .output()
        .expect("run ruva vm");

    let _ = std::fs::remove_file(&path);

    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}


fn assert_output(source: &str, expected_stdout: &str) {
    let (stdout, stderr) = run_vm(source);
    assert_eq!(
        stdout.trim_end(),
        expected_stdout.trim_end(),
        "stdout mismatch\n--- stderr ---\n{}",
        stderr
    );
}

#[test]
fn hello_world() {
    assert_output(
        "fn main() { println!(\"Hello from the VM!\") }",
        "Hello from the VM!",
    );
}

#[test]
fn arithmetic_precedence_and_multi_arg_print() {
    assert_output(
        "fn main() {
    let x = 10 + 3 * 2
    println!(\"10 + 3 * 2 = \", x)
}",
        "10 + 3 * 2 = 16",
    );
}

#[test]
fn while_loop_terminates_and_accumulates() {

    assert_output(
        "fn main() {
    let mut i = 0
    let mut sum = 0
    while i < 10 {
        sum = sum + i
        i = i + 1
    }
    println!(\"sum = \", sum)
}",
        "sum = 45",
    );
}

#[test]
fn loop_body_mutation_persists() {

    assert_output(
        "fn main() {
    let mut n = 5
    let mut acc = 1
    while n > 0 {
        acc = acc * n
        n = n - 1
    }
    println!(\"5! = \", acc)
}",
        "5! = 120",
    );
}

#[test]
fn if_else_false_branch() {
    assert_output(
        "fn main() {
    let a = 3
    let b = 7
    if a > b {
        println!(\"A\")
    } else {
        println!(\"B\")
    }
}",
        "B",
    );
}

#[test]
fn if_else_true_branch() {

    assert_output(
        "fn main() {
    let x = 8
    if x > 5 {
        println!(\"bigger\")
    } else {
        println!(\"smaller\")
    }
}",
        "bigger",
    );
}

#[test]
fn function_call_and_recursion() {
    assert_output(
        "fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

fn main() {
    let r = factorial(5)
    println!(\"5! = \", r)
}",
        "5! = 120",
    );
}

#[test]
fn fibonacci_recursion() {
    assert_output(
        "fn fib(n: i64) -> i64 {
    if n < 2 {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    println!(\"fib(7) = \", fib(7))
}",
        "fib(7) = 13",
    );
}

#[test]
fn for_in_loop_over_array() {
    assert_output(
        "fn main() {
    let arr = [2, 4, 6, 8]
    let mut total = 0
    for k in arr {
        total = total + k
    }
    println!(\"sum = \", total)
}",
        "sum = 20",
    );
}

#[test]
fn nested_loops_inner_resets_each_outer_iteration() {
    assert_output(
        "fn main() {
    let mut outer = 0
    let mut total = 0
    while outer < 3 {
        let mut inner = 0
        while inner < 4 {
            total = total + 1
            inner = inner + 1
        }
        outer = outer + 1
    }
    println!(\"nested = \", total)
}",
        "nested = 12",
    );
}

#[test]
fn if_inside_while_both_jumps() {
    assert_output(
        "fn main() {
    let mut n = 10
    let mut evens = 0
    while n > 0 {
        if n % 2 == 0 {
            evens = evens + 1
        }
        n = n - 1
    }
    println!(\"evens = \", evens)
}",
        "evens = 5",
    );
}

#[test]
fn negative_numbers_and_modulo() {
    assert_output(
        "fn main() {
    let neg = 0 - 10
    println!(\"-10 = \", neg)
    println!(\"-10 + 25 = \", neg + 25)
    println!(\"17 % 5 = \", 17 % 5)
}",
        "-10 = -10\n-10 + 25 = 15\n17 % 5 = 2",
    );
}

#[test]
fn string_concat_and_multiple_args() {
    assert_output(
        "fn main() {
    println!(\"Hello\" + \" \" + \"World\")
    println!(\"a\", 1, \"b\", 2)
}",
        "Hello World\na1b2",
    );
}

#[test]
fn function_taking_param_uses_local_correctly() {

    assert_output(
        "fn add(a: i64, b: i64) -> i64 {
    return a + b
}

fn main() {
    println!(\"add = \", add(3, 4))
}",
        "add = 7",
    );
}

#[test]
fn rve_extension_is_accepted() {

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("vm_rv_{}_{}.rve", std::process::id(), id));
    std::fs::write(&path, "fn main() { let g = 7 println!(\"rve-alias:\", g) }").expect("write temp .rve");
    let out = Command::new(env!("CARGO_BIN_EXE_ruva"))
        .arg("vm")
        .arg(&path)
        .output()
        .expect("run ruva vm on .rve");
    let _ = std::fs::remove_file(&path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rve-alias:"),
        "expected .rve file to run, got stdout: {:?}",
        stdout
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("Expected a .ruva or .rve"),
        ".rve file was unexpectedly rejected"
    );
}

#[test]
fn rgu_drives_the_vm_directly() {



    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("rgu_vm_{}_{}.rve", std::process::id(), id));
    std::fs::write(
        &path,
        "fn main() { let x = 6 let y = 7 println!(\"rgu-result:\", x * y) }",
    )
    .expect("write temp .rve");
    let out = Command::new(env!("CARGO_BIN_EXE_rgu"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("run rgu on .rve");
    let _ = std::fs::remove_file(&path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("rgu-result:"),
        "expected RGu to run the file through the VM, got stdout: {:?}",
        stdout
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("cargo"),
        "RGu must not invoke cargo at runtime"
    );
}



fn assert_vm_error(source: &str, msg: &str) {
    let (stdout, stderr) = run_vm(source);
    assert!(stdout.is_empty(), "expected no stdout, got: {}", stdout);
    assert!(
        stderr.contains(msg),
        "expected error containing {:?} in stderr, got: {}",
        msg,
        stderr
    );
    assert!(
        !stderr.contains("panicked"),
        "VM panicked instead of returning a clean error: {} -- {}",
        msg,
        stderr
    );
}

#[test]
fn integer_add_overflow_errors_not_panics() {
    assert_vm_error(
        "fn main() {
    let x = 9223372036854775807
    let y = x + 1
    println!(y)
}",
        "Integer overflow",
    );
}

#[test]
fn integer_mul_overflow_errors_not_panics() {
    assert_vm_error(
        "fn main() {
    let x = 9223372036854775807
    let y = x * 2
    println!(y)
}",
        "Integer overflow",
    );
}

#[test]
fn div_by_zero_errors_not_panics() {
    assert_vm_error(
        "fn main() {
    let y = 42 / 0
    println!(y)
}",
        "Integer division error",
    );
}

#[test]
fn huge_string_repeat_errors_not_panics() {
    assert_vm_error(
        "fn main() {
    let s = \"ab\"
    let big = s * 1000000000000000000
    println!(big)
}",
        "String repeat too large",
    );
}

#[test]
fn while_break_exits_loop_and_skips_remaining_body() {


    assert_output(
        "fn main() {
    let mut i = 0
    let mut hits = 0
    while i < 10 {
        i = i + 1
        if i == 3 {
            break
        }
        hits = hits + 1
    }
    println!(\"i=\", i, \" hits=\", hits)
}",
        "i=3 hits=2",
    );
}

#[test]
fn while_continue_skips_to_condition_which_is_reevaluated() {


    assert_output(
        "fn main() {
    let mut j = 0
    let mut evens = 0
    while j < 6 {
        j = j + 1
        if j % 2 == 1 {
            continue
        }
        evens = evens + 1
    }
    println!(\"evens=\", evens)
}",
        "evens=3",
    );
}

#[test]
fn for_in_break_and_continue() {


    assert_output(
        "fn main() {
    let mut total = 0
    let mut seen = 0
    for k in [1, 2, 3, 4, 5] {
        if k == 4 {
            break
        }
        if k == 2 {
            continue
        }
        total = total + k
        seen = seen + 1
    }
    println!(\"total=\", total, \" seen=\", seen)
}",
        "total=4 seen=2",
    );
}

#[test]
fn for_in_loop_variable_survives_nested_block() {


    assert_output(
        "fn main() {
    let mut total = 0
    for k in [1, 2, 3, 4] {
        if k == 2 {
            println!(\"skip \")
        }
        total = total + k
    }
    println!(\"sum=\", total)
}",
        "skip \nsum=10",
    );
}

#[test]
fn loop_expression_break_returns_and_terminates() {

    assert_output(
        "fn main() {
    let mut i = 0
    let mut acc = 0
    loop {
        i = i + 1
        if i > 5 {
            break
        }
        acc = acc + i
    }
    println!(\"acc=\", acc)
}",
        "acc=15",
    );
}

#[test]
fn for_in_continue_in_nested_if_keeps_increment() {


    assert_output(
        "fn main() {
    let mut out = 0
    for k in [0, 1, 2, 3, 4, 5] {
        if k % 2 == 1 {
            continue
        }
        out = out + k
    }
    println!(\"out=\", out)
}",
        "out=6",
    );
}

#[test]
fn closure_captures_local_and_survives_outer_return() {


    assert_output(
        "fn make_adder(base: i64) -> fn(i64) -> i64 {
    let offset = base * 2
    return |x: i64| -> i64 { x + offset }
}

fn main() {
    let add6 = make_adder(3)
    println!(\"add6(4)=\", add6(4))
}",
        "add6(4)=10",
    );
}

#[test]
fn closure_passed_as_argument_and_called() {

    assert_output(
        "fn apply(f: fn(i64) -> i64, v: i64) -> i64 {
    return f(v)
}

fn main() {
    let m = 4
    let quad = |a: i64| -> i64 { a * m }
    println!(\"apply=\", apply(quad, 5))
}",
        "apply=20",
    );
}

#[test]
fn closure_mutates_captured_local_across_calls() {


    assert_output(
        "fn make_counter() -> fn(i64) -> i64 {
    let mut n = 0
    return |u: i64| -> i64 {
        n = n + 1
        return n
    }
}

fn main() {
    let c = make_counter()
    println!(\"c1=\", c(0), \" c2=\", c(0), \" c3=\", c(0))
}",
        "c1=1 c2=2 c3=3",
    );
}

#[test]
fn zero_arg_closure_captures_and_is_called() {



    assert_output(
        "fn main() {
    let name = \"Ruva\"
    let greet = || -> string {
        return \"Hello \" + name
    }
    println!(\"z=\", greet())
}",
        "z=Hello Ruva",
    );
}

#[test]
fn zero_arg_closure_returned_from_function() {


    assert_output(
        "fn mk() -> fn() -> i64 {
    let base = 3
    return || -> i64 {
        return base * 2
    }
}
\nfn main() {
    let g = mk()
    println!(\"r=\", g())
}",
        "r=6",
    );
}

#[test]
fn zero_arg_stateful_closure_counter() {

    assert_output(
        "fn make_counter() -> fn() -> i64 {
    let mut n = 0
    return || -> i64 {
        n = n + 1
        return n
    }
}
\nfn main() {
    let c = make_counter()
    println!(\"c1=\", c(), \" c2=\", c(), \" c3=\", c())
}",
        "c1=1 c2=2 c3=3",
    );
}

#[test]
fn logical_or_and_zero_arg_closure_coexist() {


    assert_output(
        "fn main() {
    let a = true
    let b = false
    let ping = || -> bool {
        return true
    }
    println!(\"or=\", a || b, \" closure=\", ping())
}",
        "or=true closure=true",
    );
}

#[test]
fn nested_closure_mutates_outer_captured_state_across_calls() {



    assert_output(
        "fn make_accumulator(step: i64) -> fn() -> fn() -> i64 {
    let mut n = 0
    return || -> fn() -> i64 {
        return || -> i64 {
            n = n + step
            return n
        }
    }
}
\nfn main() {
    let a = make_accumulator(3)
    let inc = a()
    println!(\"v1=\", inc(), \" v2=\", inc(), \" v3=\", inc())
}",
        "v1=3 v2=6 v3=9",
    );
}

#[test]
fn nested_closure_captures_local_and_upvalue_from_enclosing_closure() {



    assert_output(
        "fn make_pair(seed: i64) -> fn(i64) -> fn(i64) -> i64 {
    return |a: i64| -> fn(i64) -> i64 {
        let local = seed + a
        return |b: i64| -> i64 {
            return local + seed + b
        }
    }
}
\nfn main() {
    let g = make_pair(7)
    let h = g(3)
    println!(\"v10=\", h(10), \" v1=\", h(1))
}",
        "v10=27 v1=18",
    );
}


#[test]
fn println_format_string_interpolation() {
    assert_output(
        "fn main() {
    println!(\"hello {} world {}\", 1, 2)
}",
        "hello 1 world 2",
    );
}

#[test]
fn println_format_with_explicit_indices() {
    assert_output(
        "fn main() {
    println!(\"{1} {0}\", 10, 20)
}",
        "20 10",
    );
}

#[test]
fn println_format_escaped_braces() {
    assert_output(
        "fn main() {
    println!(\"literal {{braces}} {}\", 42)
}",
        "literal {braces} 42",
    );
}
