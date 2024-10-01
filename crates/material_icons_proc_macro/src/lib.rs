use proc_macro::TokenStream;
use std::collections::HashSet;

#[proc_macro]
pub fn code_points(_item: TokenStream) -> TokenStream {
    let codepoints = include_str!("../MaterialSymbolsRounded-Regular.codepoints");

    let mut names = HashSet::new();

    let code: String = codepoints
        .split("\n")
        .map(str::trim)
        .filter_map(|point| {
            let split_point: Vec<&str> = point.split(" ").collect();

            if split_point.len() > 1 {
                let name = split_point[0].to_uppercase();
                let addr = split_point[1];

                if !names.contains(&name) {
                    let token = Some(format!(
                        "pub const ICON_{name}: &str = \"\\u{{{addr}}}\";\n"
                    ));
                    names.insert(name);
                    token
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    code.parse().unwrap()
}
