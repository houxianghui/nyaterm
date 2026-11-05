use gpui::{Pixels, px};
use gpui_kit::component::Size;

/// Standard height for ordinary single-line form controls.
pub const NYA_FORM_CONTROL_HEIGHT_PX: f32 = 32.;

pub(crate) fn form_control_height() -> Pixels {
    px(NYA_FORM_CONTROL_HEIGHT_PX)
}

pub(crate) fn form_control_size() -> Size {
    Size::Medium
}

/// App-wide corner radii, aligned with the React-line fork scale
/// (`--radius: 0.625rem`, i.e. 6 / 8 / 10 / 14 px).
pub const NYA_RADIUS_SM_PX: f32 = 6.;
pub const NYA_RADIUS_MD_PX: f32 = 8.;
pub const NYA_RADIUS_LG_PX: f32 = 10.;
pub const NYA_RADIUS_XL_PX: f32 = 14.;

pub fn radius_sm() -> Pixels {
    px(NYA_RADIUS_SM_PX)
}

pub fn radius_md() -> Pixels {
    px(NYA_RADIUS_MD_PX)
}

pub fn radius_lg() -> Pixels {
    px(NYA_RADIUS_LG_PX)
}

pub fn radius_xl() -> Pixels {
    px(NYA_RADIUS_XL_PX)
}
