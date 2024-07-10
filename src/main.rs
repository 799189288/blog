fn main() {
    let result = api::main();
    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
