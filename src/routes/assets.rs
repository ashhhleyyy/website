use std::fmt::Write;

use axum::{
    extract::Query,
    headers::{ContentType, HeaderMapExt},
    http::HeaderMap,
    Json,
};
use image::GenericImageView;
use mime_guess::mime::{APPLICATION_JAVASCRIPT_UTF_8, IMAGE_SVG};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BackgroundQuery {
    #[serde(default)]
    error: bool,
    #[serde(default)]
    small: bool,
    star_colour: Option<String>,
}

pub async fn background(Query(query): Query<BackgroundQuery>) -> (HeaderMap, String) {
    let (width, height) = if query.small {
        (512, 512)
    } else {
        (1024, 1024)
    };

    let mut headers = HeaderMap::new();
    headers.typed_insert(ContentType::from(IMAGE_SVG));

    let mut svg = String::new();

    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}">"#,
    )
    .expect("failed to write");

    let fill = if query.error {
        "orange".to_string()
    } else {
        query
            .star_colour
            .unwrap_or_else(|| "white".to_string())
    };

    write!(
        svg,
        r##"<defs><radialGradient id="star"><stop offset="0%" stop-color="{fill}" /><stop offset="90%" stop-color="#00000000" /></radialGradient></defs>"##
    )
    .expect("failed to write");

    write!(
        svg,
        r#"<rect x="0" y="0" width="{width}" height="{height}" fill='#13092b' />"#
    )
    .expect("failed to write");

    for _ in 0..256 {
        let x = fastrand::u32(0..width);
        let y = fastrand::u32(0..height);

        write!(
            svg,
            r#"<circle class="star" cx="{x}" cy="{y}" r="4" fill="url('#star')" />"#
        )
        .expect("failed to write");
    }

    svg.push_str("</svg>");

    (headers, svg)
}

pub async fn image_script() -> (HeaderMap, String) {
    const SIZE: usize = 48;
    const SCRIPT_FOOTER: &str = r#"console.log("Ooh wow someone's interested in how my website works! If that sounds like you, then you can check out the code on my GitHub: https://github.com/ashhhleyyy/website")"#;

    let image_data = include_bytes!("../../assets/images/pfp.png");
    let image = image::load_from_memory(image_data).unwrap();
    let resized = image.resize(
        SIZE as u32,
        SIZE as u32,
        image::imageops::FilterType::Nearest,
    );
    let mut pixels = [[0; SIZE]; SIZE];
    for (x, y, pixel) in resized.pixels() {
        let col = ((pixel.0[0] as u32) << 24)
            | ((pixel.0[1] as u32) << 16)
            | ((pixel.0[2] as u32) << 8)
            | (pixel.0[3] as u32);
        pixels[y as usize][x as usize] = col;
    }

    let mut dots = String::new();
    let mut colours = String::new();
    for row in pixels {
        let mut row_colours = String::new();
        let mut all_zero = true;
        for pixel in row {
            if pixel != 0 {
                all_zero = false;
            }
            write!(
                row_colours,
                ",\"color: #{:08x}; background-color: #{:08x}\"",
                pixel, pixel
            )
            .expect("failed to write");
        }
        if !all_zero {
            for _ in 0..SIZE {
                dots.push_str("%c●●");
            }
            dots.push_str("\\n");
            colours.push_str(&row_colours);
        }
    }

    let mut headers = HeaderMap::new();
    headers.typed_insert(ContentType::from(APPLICATION_JAVASCRIPT_UTF_8));

    let script = format!(r#"console.log("{}"{});{}"#, dots, colours, SCRIPT_FOOTER);

    (headers, script)
}

#[derive(serde::Serialize)]
pub struct Oembed {
    version: &'static str,
    author_name: &'static str,
    author_url: &'static str,
}

pub async fn oembed() -> Json<Oembed> {
    Json(Oembed {
        version: "1.0",
        author_name: "Ashhhleyyy",
        author_url: "https://ashhhleyyy.dev",
    })
}
