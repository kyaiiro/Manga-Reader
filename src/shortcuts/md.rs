pub fn normalize_markdown(input: &str) -> String {
    input
        .split("\n\n")                    // split into real paragraphs/blocks
        .map(|block| block.replace('\n', " ")) // join soft-broken lines within a block
        .collect::<Vec<_>>()
        .join("\n\n")
}