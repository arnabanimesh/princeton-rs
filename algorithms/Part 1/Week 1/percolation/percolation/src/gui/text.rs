use image::{GenericImage, GenericImageView, Pixel};

use conv::ValueInto;

use rusttype::{point, Font, PositionedGlyph, Rect, Scale};
use std::cmp::max;
pub(super) trait Canvas {
    type Pixel: Pixel;

    fn dimensions(&self) -> (u32, u32);

    fn width(&self) -> u32 {
        self.dimensions().0
    }

    fn height(&self) -> u32 {
        self.dimensions().1
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel;

    fn draw_pixel(&mut self, x: u32, y: u32, color: Self::Pixel);
}

impl<I> Canvas for I
where
    I: GenericImage,
{
    type Pixel = I::Pixel;

    fn dimensions(&self) -> (u32, u32) {
        <I as GenericImageView>::dimensions(self)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.get_pixel(x, y)
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Self::Pixel) {
        self.put_pixel(x, y, color)
    }
}

struct Blend<I>(I);

impl<I: GenericImage> Canvas for Blend<I> {
    type Pixel = I::Pixel;

    fn dimensions(&self) -> (u32, u32) {
        self.0.dimensions()
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.0.get_pixel(x, y)
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Self::Pixel) {
        let mut pix = self.0.get_pixel(x, y);
        pix.blend(&color);
        self.0.put_pixel(x, y, pix);
    }
}

pub(super) trait Clamp<T> {
    fn clamp(x: T) -> Self;
}

macro_rules! implement_clamp {
    ($from:ty, $to:ty, $min:expr, $max:expr, $min_from:expr, $max_from:expr) => {
        impl Clamp<$from> for $to {
            fn clamp(x: $from) -> $to {
                if x < $max_from as $from {
                    if x > $min_from as $from {
                        x as $to
                    } else {
                        $min
                    }
                } else {
                    $max
                }
            }
        }
    };
}
implement_clamp!(f32, u8, u8::MIN, u8::MAX, u8::MIN as f32, u8::MAX as f32);

fn weighted_sum<P: Pixel>(left: P, right: P, left_weight: f32, right_weight: f32) -> P
where
    P::Subpixel: ValueInto<f32> + Clamp<f32>,
{
    left.map2(&right, |p, q| {
        weighted_channel_sum(p, q, left_weight, right_weight)
    })
}

#[inline(always)]
fn weighted_channel_sum<C>(left: C, right: C, left_weight: f32, right_weight: f32) -> C
where
    C: ValueInto<f32> + Clamp<f32>,
{
    Clamp::clamp(cast(left) * left_weight + cast(right) * right_weight)
}

fn cast<T, U>(x: T) -> U
where
    T: ValueInto<U>,
{
    match x.value_into() {
        Ok(y) => y,
        Err(_) => panic!("Failed to convert"),
    }
}

fn layout_glyphs(
    scale: Scale,
    font: &Font,
    text: &str,
    mut f: impl FnMut(PositionedGlyph, Rect<i32>),
) -> (i32, i32) {
    let v_metrics = font.v_metrics(scale);

    let (mut w, mut h) = (0, 0);

    for g in font.layout(text, scale, point(0.0, v_metrics.ascent)) {
        if let Some(bb) = g.pixel_bounding_box() {
            w = max(w, bb.max.x);
            h = max(h, bb.max.y);
            f(g, bb);
        }
    }

    (w, h)
}

pub(super) fn text_size(scale: Scale, font: &Font, text: &str) -> (i32, i32) {
    layout_glyphs(scale, font, text, |_, _| {})
}

pub(super) fn draw_text_mut<'a, C>(
    canvas: &'a mut C,
    color: C::Pixel,
    x: i32,
    y: i32,
    scale: Scale,
    font: &'a Font<'a>,
    text: &'a str,
) where
    C: Canvas,
    <C::Pixel as Pixel>::Subpixel: ValueInto<f32> + Clamp<f32>,
{
    let image_width = canvas.width() as i32;
    let image_height = canvas.height() as i32;

    layout_glyphs(scale, font, text, |g, bb| {
        g.draw(|gx, gy, gv| {
            let gx = gx as i32 + bb.min.x;
            let gy = gy as i32 + bb.min.y;

            let image_x = gx + x;
            let image_y = gy + y;

            if (0..image_width).contains(&image_x) && (0..image_height).contains(&image_y) {
                let pixel = canvas.get_pixel(image_x as u32, image_y as u32);
                let weighted_color = weighted_sum(pixel, color, 1.0 - gv, gv);
                canvas.draw_pixel(image_x as u32, image_y as u32, weighted_color);
            }
        })
    });
}
