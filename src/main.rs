fn main() {
    if let Err(error) = ai_teamlead::app::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
