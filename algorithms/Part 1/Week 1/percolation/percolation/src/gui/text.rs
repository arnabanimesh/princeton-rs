use image::{GenericImage, GenericImageView, Pixel};

use conv::ValueInto;

use rusttype::{point, Font, PositionedGlyph, Rect, Scale};
use std::cmp::max;

/// A surface for drawing on - many drawing functions in this
/// library are generic over a `Canvas` to allow the user to
/// configure e.g. whether to use blending.
///
/// All instances of `GenericImage` implement `Canvas`, with
/// the behaviour of `draw_pixel` being equivalent to calling
/// `set_pixel` with the same arguments.
///
/// See [`Blend`](struct.Blend.html) for another example implementation
/// of this trait - its implementation of `draw_pixel` alpha-blends
/// the input value with the pixel's current value.
///
/// # Examples
/// ```
/// # extern crate image;
/// # #[macro_use]
/// # extern crate imageproc;
/// # fn main() {
/// use image::{Pixel, Rgba, RgbaImage};
/// use imageproc::drawing::{Canvas, Blend};
///
/// // A trivial function which draws on a Canvas
/// fn write_a_pixel<C: Canvas>(canvas: &mut C, c: C::Pixel) {
///     canvas.draw_pixel(0, 0, c);
/// }
///
/// // Background color
/// let solid_blue = Rgba([0u8, 0u8, 255u8, 255u8]);
///
/// // Drawing color
/// let translucent_red = Rgba([255u8, 0u8, 0u8, 127u8]);
///
/// // Blended combination of background and drawing colors
/// let mut alpha_blended = solid_blue;
/// alpha_blended.blend(&translucent_red);
///
/// // The implementation of Canvas for GenericImage overwrites existing pixels
/// let mut image = RgbaImage::from_pixel(1, 1, solid_blue);
/// write_a_pixel(&mut image, translucent_red);
/// assert_eq!(*image.get_pixel(0, 0), translucent_red);
///
/// // This behaviour can be customised by using a different Canvas type
/// let mut image = Blend(RgbaImage::from_pixel(1, 1, solid_blue));
/// write_a_pixel(&mut image, translucent_red);
/// assert_eq!(*image.0.get_pixel(0, 0), alpha_blended);
/// # }
/// ```
pub(super) trait Canvas {
    /// The type of `Pixel` that can be drawn on this canvas.
    type Pixel: Pixel;

    /// The width and height of this canvas.
    fn dimensions(&self) -> (u32, u32);

    /// The width of this canvas.
    fn width(&self) -> u32 {
        self.dimensions().0
    }

    /// The height of this canvas.
    fn height(&self) -> u32 {
        self.dimensions().1
    }

    /// Returns the pixel located at (x, y).
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel;

    /// Draw a pixel at the given coordinates. `x` and `y`
    /// should be within `dimensions` - if not then panicking
    /// is a valid implementation behaviour.
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

/// A canvas that blends pixels when drawing.
///
/// See the documentation for [`Canvas`](trait.Canvas.html)
/// for an example using this type.
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
    /// Clamp `x` to a valid value for this type.
    fn clamp(x: T) -> Self;
}

/// Creates an implementation of Clamp<From> for type To.
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

/// Get the width and height of the given text, rendered with the given font and scale.
///
/// Note that this function *does not* support newlines, you must do this manually.
pub(super) fn text_size(scale: Scale, font: &Font, text: &str) -> (i32, i32) {
    layout_glyphs(scale, font, text, |_, _| {})
}

/// Draws colored text on an image in place.
///
/// `scale` is augmented font scaling on both the x and y axis (in pixels).
///
/// Note that this function *does not* support newlines, you must do this manually.
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
