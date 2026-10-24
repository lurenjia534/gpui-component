#[cfg(test)]
#[test]
fn sparse_table_evidence() {
    for (columns, rows) in [(32, 64), (64, 128), (128, 256)] {
        let source = format!(
            "{}|\n{}|\n{}",
            "|h".repeat(columns),
            "|---".repeat(columns),
            "|x|\n".repeat(rows)
        );
        let mut cx = NodeContext::default();
        let document = parse(&source, &mut cx).unwrap();
        let BlockNode::Table(table) = &document.blocks[0] else {
            panic!("expected table")
        };
        let cells: usize = table.children.iter().map(|row| row.children.len()).sum();
        println!(
            "columns={columns} body_rows={rows} bytes={} retained_cells={cells} source_cells={} generated_cells={}",
            source.len(),
            columns + rows,
            cells - columns - rows
        );
    }
}
