use std::cell::RefCell;

use comrak::{Arena, parse_document, ComrakOptions, nodes::{AstNode, NodeValue, Ast}, format_html_with_plugins};
use regex::{Regex, Captures};
use reqwest::Url;

lazy_static::lazy_static!{
    static ref TITLE_REGEX: Regex = Regex::new(r"<h1>(.*)</h1>").unwrap();
    static ref ICON_REGEX: Regex = Regex::new(r"!--icon\((.*)\)--!").unwrap();
}

// Yes I know you're not *supposed* to parse html with regex
pub fn extract_title(content: &str) -> &str {
    TITLE_REGEX
        .captures(content)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
}

pub fn render_markdown(markdown: &str) -> (String, String) {
    // 'front matter' parsing
    let (description, body) = markdown.split_once("\n---\n").unwrap_or(("", markdown));

    let mut options = ComrakOptions::default();
    options.extension.autolink = true;
    options.extension.table = true;
    options.extension.description_lists = true;
    options.extension.superscript = true;
    options.extension.strikethrough = true;
    options.extension.footnotes = true;

    let arena = Arena::new();
    let root = parse_document(&arena, body, &options);

    fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
    where
        F: Fn(&'a AstNode<'a>),
    {
        f(node);
        for c in node.children() {
            iter_nodes(c, f);
        }
    }

    iter_nodes(root, &|node| {
        if let NodeValue::Link(ref mut link) = node.data.borrow_mut().value {
            let url = std::str::from_utf8(&link.url)
                .unwrap();
            if let Ok(url) = Url::parse(url) {
                let service = url.scheme();
                let target_url = match service {
                    "modrinth" => {
                        format!("https://modrinth.com/{}{}", url.host_str().unwrap_or("mod"), url.path())
                    }
                    "github" => {
                        format!("https://github.com/{}{}", url.host_str().unwrap_or("ashhhleyyy"), url.path())
                    }
                    _ => return,
                };
                link.url = target_url.into_bytes();
                let insert = format!("!--icon({})--! ", service);
                let new_node = arena.alloc(AstNode::new(RefCell::new(Ast::new(
                    NodeValue::Text(insert.into_bytes()),
                ))));
                node.prepend(new_node);
            }
        }
    });

    let mut html = vec![];
    format_html_with_plugins(root, &options, &mut html, &Default::default()).unwrap();

    let html = String::from_utf8(html).expect("post is somehow invalid UTF-8");

    (description.to_owned(), replace_icons(html))
}

fn replace_icons(html: String) -> String {
    ICON_REGEX.replace_all(&html, |captures: &Captures| {
        match &captures[1] {
            "modrinth" => r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="icon hover-modrinth" viewBox="0 0 512 514"><path fill-rule="evenodd" clip-rule="evenodd" d="M503.16 323.56C514.55 281.47 515.32 235.91 503.2 190.76C466.57 54.2299 326.04 -26.8001 189.33 9.77991C83.8101 38.0199 11.3899 128.07 0.689941 230.47H43.99C54.29 147.33 113.74 74.7298 199.75 51.7098C306.05 23.2598 415.13 80.6699 453.17 181.38L411.03 192.65C391.64 145.8 352.57 111.45 306.3 96.8198L298.56 140.66C335.09 154.13 364.72 184.5 375.56 224.91C391.36 283.8 361.94 344.14 308.56 369.17L320.09 412.16C390.25 383.21 432.4 310.3 422.43 235.14L464.41 223.91C468.91 252.62 467.35 281.16 460.55 308.07L503.16 323.56Z"></path><path d="M321.99 504.22C185.27 540.8 44.7501 459.77 8.11011 323.24C3.84011 307.31 1.17 291.33 0 275.46H43.27C44.36 287.37 46.4699 299.35 49.6799 311.29C53.0399 323.8 57.45 335.75 62.79 347.07L101.38 323.92C98.1299 316.42 95.39 308.6 93.21 300.47C69.17 210.87 122.41 118.77 212.13 94.7601C229.13 90.2101 246.23 88.4401 262.93 89.1501L255.19 133C244.73 133.05 234.11 134.42 223.53 137.25C157.31 154.98 118.01 222.95 135.75 289.09C136.85 293.16 138.13 297.13 139.59 300.99L188.94 271.38L174.07 231.95L220.67 184.08L279.57 171.39L296.62 192.38L269.47 219.88L245.79 227.33L228.87 244.72L237.16 267.79C237.16 267.79 253.95 285.63 253.98 285.64L277.7 279.33L294.58 260.79L331.44 249.12L342.42 273.82L304.39 320.45L240.66 340.63L212.08 308.81L162.26 338.7C187.8 367.78 226.2 383.93 266.01 380.56L277.54 423.55C218.13 431.41 160.1 406.82 124.05 361.64L85.6399 384.68C136.25 451.17 223.84 484.11 309.61 461.16C371.35 444.64 419.4 402.56 445.42 349.38L488.06 364.88C457.17 431.16 398.22 483.82 321.99 504.22Z"></path></svg>"#,
            "github" => r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" class="icon hover-github" preserveAspectRatio="xMidYMid meet" viewBox="0 0 24 24"><path d="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5c.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34c-.46-1.16-1.11-1.47-1.11-1.47c-.91-.62.07-.6.07-.6c1 .07 1.53 1.03 1.53 1.03c.87 1.52 2.34 1.07 2.91.83c.09-.65.35-1.09.63-1.34c-2.22-.25-4.55-1.11-4.55-4.92c0-1.11.38-2 1.03-2.71c-.1-.25-.45-1.29.1-2.64c0 0 .84-.27 2.75 1.02c.79-.22 1.65-.33 2.5-.33c.85 0 1.71.11 2.5.33c1.91-1.29 2.75-1.02 2.75-1.02c.55 1.35.2 2.39.1 2.64c.65.71 1.03 1.6 1.03 2.71c0 3.82-2.34 4.66-4.57 4.91c.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0 0 12 2Z"></path></svg>"#,
            _ => "",
        }
    }).to_string()
}
