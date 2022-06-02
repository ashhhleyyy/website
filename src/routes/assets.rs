use axum::{
    extract::{Path, Query, TypedHeader},
    headers::{ContentType, ETag, HeaderMapExt, IfNoneMatch},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use hex::ToHex;
use image::GenericImageView;
use mime_guess::mime::{APPLICATION_JAVASCRIPT_UTF_8, IMAGE_SVG};
use rust_embed::{EmbeddedFile, RustEmbed};
use serde::Deserialize;
use time::{OffsetDateTime, Month};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub struct AutoContentType(String, ETag, EmbeddedFile);

impl IntoResponse for AutoContentType {
    fn into_response(self) -> axum::response::Response {
        let mut res = self.2.data.into_response();
        res.headers_mut().remove(CONTENT_TYPE);
        res.headers_mut().typed_insert(self.1);
        if let Some(mime) = mime_guess::from_path(&self.0).first_raw() {
            res.headers_mut()
                .append(CONTENT_TYPE, HeaderValue::from_static(mime));
        }
        res
    }
}

pub async fn get_asset(
    Path(path): Path<String>,
    if_none_match: Option<TypedHeader<IfNoneMatch>>,
) -> Result<AutoContentType, StatusCode> {
    match Asset::get(&path[1..]) {
        Some(asset) => {
            let hash = asset.metadata.sha256_hash().encode_hex::<String>();
            let etag = format!(r#"{:?}"#, hash).parse::<ETag>().unwrap();
            if let Some(if_none_match) = if_none_match {
                if !if_none_match.precondition_passes(&etag) {
                    return Err(StatusCode::NOT_MODIFIED);
                }
            }
            Ok(AutoContentType(path[1..].to_string(), etag, asset))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

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

    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
        width, height
    ));

    svg.push_str(&format!(
        r#"<rect x="0" y="0" width="{}" height="{}" fill='#13092b' />"#,
        width, height
    ));

    let colours = if use_colours() { *random_choice(COLOURS) } else { &["#ccc"] };

    for _ in 0..256 {
        let x = fastrand::u32(0..width);
        let y = fastrand::u32(0..height);
        let fill = if query.error {
            "orange".to_string()
        } else {
            query.star_colour.clone().unwrap_or_else(|| random_choice(colours).to_string())
        };
        svg.push_str(&format!(
            r#"<circle class="star" cx="{}" cy="{}" r="2" fill="{}" />"#,
            x, y, fill
        ));
    }

    svg.push_str("</svg>");

    (headers, svg)
}

pub async fn image_script() -> (HeaderMap, String) {
    const SIZE: usize = 64;
    const SCRIPT_FOOTER: &str = r#"console.log("Ooh wow someone's interested in how my website works! If that sounds like you, then you can check out the code on my GitHub: https://github.com/ashhhleyyy/website")"#;

    let image_data = Asset::get("images/pfp.png").unwrap().data;
    let image = image::load_from_memory(&image_data).unwrap();
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
            row_colours.push_str(&format!(",\"color: #{:08x}; background-color: #{:08x}\"", pixel, pixel));
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

#[rustfmt::skip]
const COLOURS: &[&[&'static str]] = &[&["#a3a3a3","#ffffff","#800070",],&["#d60270","#d60270","#9b4f96","#0038A8","#0038A8",],&["#d62900","#ff9b55","#ffffff","#d461a6","#a50062",],&["#fff430","#ffffff","#9c59d1",],&["#ff1b8d","#ffda00","#1bb3ff",],&["#ff0018","#ffa52c","#ffff41","#008018","#0000f9","#86007d",],&["#55cdfc","#f7a8b8","#ffffff","#f7a8b8","#55cdfc",],];

fn random_choice<T: Sized>(arr: &[T]) -> &T {
    &arr[fastrand::usize(..arr.len())]
}

fn use_colours() -> bool {
    OffsetDateTime::now_utc().month() == Month::June
}
