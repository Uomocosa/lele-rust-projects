use image::{Rgba, RgbaImage};
use imageproc::geometric_transformations::{rotate_about_center, Border, Interpolation};

/// Straighten + de-stretch a photo taken of a screen at an angle.
///
/// Usage: image_fix_atomium <rotation_deg> <scale_y> <keystone> <out_name>
///   rotation_deg  negative = counter-clockwise
///   scale_y       >1 stretches vertically (fixes squashed/wide people)
///   keystone      horizontal taper, 0.0 = none; 0.05 = top 5% wider than bottom
struct Params {
    rotation_deg: f32,
    scale_y: f32,
    keystone: f32,
    out_name: String,
}

// Crop insets applied after rotation, tuned to clear the monitor bezel on the
// left, the app's back-arrow overlay, and the blank wedges rotation leaves behind.
const CROP_LEFT: u32 = 200;
const CROP_TOP: u32 = 70;
const CROP_RIGHT: u32 = 30;
const CROP_BOTTOM: u32 = 50;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let params = Params {
        rotation_deg: args.first().map_or(-5.84, |v| v.parse().unwrap()),
        scale_y: args.get(1).map_or(1.15, |v| v.parse().unwrap()),
        keystone: args.get(2).map_or(0.0, |v| v.parse().unwrap()),
        out_name: args
            .get(3)
            .cloned()
            .unwrap_or_else(|| "atomium_fixed.jpg".to_string()),
    };

    let img = image::open("input/atomium_original.jpg")
        .expect("failed to open input image")
        .to_rgba8();

    let rotated: RgbaImage = rotate_about_center(
        &img,
        params.rotation_deg.to_radians(),
        Interpolation::Bilinear,
        Border::Constant(Rgba([0, 0, 0, 0])),
    );

    let (w, h) = rotated.dimensions();
    let crop_w = w - (CROP_LEFT + CROP_RIGHT);
    let crop_h = h - (CROP_TOP + CROP_BOTTOM);
    let cropped =
        image::imageops::crop_imm(&rotated, CROP_LEFT, CROP_TOP, crop_w, crop_h).to_image();

    let deskewed = apply_keystone(&cropped, params.keystone);

    let new_h = (crop_h as f32 * params.scale_y).round() as u32;
    let resized = image::imageops::resize(
        &deskewed,
        crop_w,
        new_h,
        image::imageops::FilterType::Lanczos3,
    );

    let out_path = format!("output/{}", params.out_name);
    image::DynamicImage::ImageRgba8(resized)
        .to_rgb8()
        .save(&out_path)
        .expect("failed to save output image");
    println!("saved {out_path} ({crop_w}x{new_h})");
}

/// Correct keystoning by scaling each row horizontally about the image centre:
/// the top row is widened by `amount`, the bottom narrowed by the same, which
/// undoes the taper you get shooting a flat screen from off-axis.
fn apply_keystone(img: &RgbaImage, amount: f32) -> RgbaImage {
    if amount.abs() < f32::EPSILON {
        return img.clone();
    }
    let (w, h) = img.dimensions();
    let cx = w as f32 / 2.0;
    let mut out = RgbaImage::new(w, h);

    for y in 0..h {
        // t goes -1 at the top to +1 at the bottom.
        let t = (y as f32 / (h - 1) as f32) * 2.0 - 1.0;
        let row_scale = 1.0 + amount * t;
        for x in 0..w {
            let src_x = cx + (x as f32 - cx) * row_scale;
            let px = if src_x >= 0.0 && src_x < w as f32 {
                *img.get_pixel(src_x as u32, y)
            } else {
                Rgba([0, 0, 0, 255])
            };
            out.put_pixel(x, y, px);
        }
    }
    out
}
