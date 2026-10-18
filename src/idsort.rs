fn main() {
    let mut args = vec![String::from("sort")];
    args.extend(std::env::args().skip(1));
    let code = idpeek::run(
        &args,
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}
