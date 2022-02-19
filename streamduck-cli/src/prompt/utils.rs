pub fn print_table(table: Vec<Vec<&str>>, first_separator: &str, separator: &str) {
    let mut max_len = vec![];

    // Calculating max size for each column
    for column in &table {
        let mut len = 0;

        for item in column {
            len = len.max(item.len());
        }

        max_len.push(len);
    }

    // Printing table
    if table.len() > 0 {
        for y in 0..table[0].len() {
            let separator = if y == 0 {
                first_separator
            } else {
                separator
            };

            for x in 0..table.len() {
                if y == 0 {
                    print!("{} {: <w$} ", separator, table[x][y], w = max_len[x])
                } else {
                    print!("{} {: >w$} ", separator, table[x][y], w = max_len[x])
                }
            }

            println!("{}", separator);
        }
    }
}

pub fn print_table_with_strings(table: Vec<Vec<String>>, first_separator: &str, separator: &str) {
    print_table(
        table.iter()
            .map(|v| {
                v.iter().map(|s| s.as_str()).collect()
            })
            .collect(),
        first_separator,
        separator
    );
}