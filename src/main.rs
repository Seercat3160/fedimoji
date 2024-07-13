use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use image::GenericImage;
use serde_json::json;
use tracing::{debug, error, warn};
use tracing_subscriber::FmtSubscriber;

const GLYPH_SIZE: u32 = 64;

fn main() {
    let args = Args::parse();

    // setup tracing
    let tracing_subscriber = FmtSubscriber::builder()
        .with_max_level({
            if args.verbose {
                tracing::Level::DEBUG
            } else {
                tracing::Level::INFO
            }
        })
        .finish();

    tracing::subscriber::set_global_default(tracing_subscriber)
        .expect("setting default subscriber failed");

    // Basic overview:
    // 1. figure out how many emoji files we have
    // 2. assign a codepoint to each
    // 3. allocate a big atlas image of the right size
    // 4. figure out the arrangement of emoji in the atlas
    // 5. read each emoji file in order, downscale it, and place it in the atlas in the right spot
    // 6. write the atlas to disk as `emoji.png`
    // 7. write the font provider definition to disk as `emoji.json`
    // 8. write the name->codepoint mapping to disk as `fedimoji.json`

    // ensure we can read the emoji directory
    let emoji_dir = args.emoji_dir;
    if !emoji_dir.is_dir() {
        error!("emoji directory `{}` does not exist", emoji_dir.display());
        return;
    }

    // get an iterator over all the PNG files in the emoji directory
    let images = emoji_dir
        .read_dir()
        .expect("reading emoji directory failed")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension() == Some("png".as_ref()) {
                Some(path)
            } else {
                None
            }
        })
        .filter_map(|path| {
            path.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .map(|s| (path, s))
        })
        .filter_map(|(path, name)| {
            // read the image
            match image::open(&path) {
                Err(err) => {
                    warn!("failed to read `{}` (skipping it): {}", path.display(), err);
                    None
                }
                Ok(image) => Some((name, image)),
            }
        })
        .map(|(name, image)| {
            // resize it
            let image = image.resize(
                GLYPH_SIZE,
                GLYPH_SIZE,
                image::imageops::FilterType::Triangle,
            );
            debug!("resized `{}`", name);
            (name, image)
        })
        .map(|(name, image)| {
            // strip ".png" from the name
            let new_name = name.trim_end_matches(".png").to_string();
            (new_name, image)
        })
        .zip((0xF0000..=0xFFFFD).filter_map(char::from_u32))
        .map(|((name, image), codepoint)| (name, codepoint, image))
        .collect::<Vec<_>>();

    if images.is_empty() {
        error!("no valid emoji provided!");
        return;
    }

    let num_glyphs: u32 = images.len() as u32;

    // allocate the atlas
    let mut atlas = image::RgbaImage::new(GLYPH_SIZE, GLYPH_SIZE * num_glyphs);
    debug!(
        "allocated {}x{} pixel atlas",
        GLYPH_SIZE,
        GLYPH_SIZE * num_glyphs
    );

    // mapping of name -> codepoint
    let mut names: HashMap<String, char> = HashMap::new();

    // set of glyph characters
    let mut chars: Vec<char> = Vec::new();

    // place the images in the atlas
    for ((name, codepoint, image), index) in images.into_iter().zip(0u32..) {
        let y = index * GLYPH_SIZE;
        atlas.copy_from(&image, 0, y).unwrap();
        debug!("copied `{}` to ({}, {})", name, 0, y);

        names.insert(name, codepoint);
        chars.push(codepoint);
    }

    // get the output directory, creating it if it doesn't exist
    let output_dir = args.output_dir;
    if !output_dir.is_dir() {
        std::fs::create_dir_all(&output_dir).unwrap();
    }

    // write the atlas
    atlas.save(&output_dir.join("emoji.png")).unwrap();
    debug!(
        "wrote atlas to `{}`",
        output_dir.join("emoji.png").display()
    );

    // write the font provider definition
    let font_provider = json!({
      "providers": [
        {
          "type": "bitmap",
          "file": "fedimoji:font/emoji.png",
          "height": 8,
          "ascent": 8,
          "chars": chars
        }
      ]
    });
    std::fs::write(
        output_dir.join("emoji.json"),
        serde_json::to_string_pretty(&font_provider).unwrap(),
    )
    .unwrap();
    debug!(
        "wrote font provider definition to `{}`",
        output_dir.join("emoji.json").display()
    );

    // write the name->codepoint mapping
    std::fs::write(
        output_dir.join("fedimoji.json"),
        serde_json::to_string_pretty(&names).unwrap(),
    )
    .unwrap();
    debug!(
        "wrote name->codepoint mapping to `{}`",
        output_dir.join("fedimoji.json").display()
    );
}

#[derive(clap::Parser)]
struct Args {
    /// Directory containing emoji images
    #[clap(long, default_value = "./emoji")]
    emoji_dir: PathBuf,

    /// Output directory
    #[clap(long, default_value = "./out")]
    output_dir: PathBuf,

    #[clap(short = 'v', long)]
    verbose: bool,
}
