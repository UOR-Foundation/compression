// Read-only diagnostic adapter to the official SDK. This is not product behavior.
fn main() {
    let source_path = std::env::args().nth(1).expect("usage: export-semantic-json SOURCE.lex.tex");
    let source = std::fs::read_to_string(source_path).expect("read authoritative source");
    let module = lexlean::sdk::read_module(&source).expect("official SDK parses authoring notation");
    let round_trip = lexlean::sdk::write_module(&source, &module).expect("official SDK writes notation");
    assert_eq!(round_trip, source, "SDK round trip changed authoritative source bytes");
    println!("{module}");
}
