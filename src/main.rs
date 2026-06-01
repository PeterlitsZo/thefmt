fn main() {
    if let Err(error) = thefmt::run(thefmt::args_without_binary_name()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
