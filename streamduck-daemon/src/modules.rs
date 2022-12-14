use streamduck_core::parameters::{Color, DynamicChoice, ParameterImpl};

#[derive(ParameterImpl, Default)]
pub struct RendererParameters {
    background_parameters: BackgroundParams,
    text_parameters: TextParams,
    caching: bool,
}

#[derive(ParameterImpl, Default)]
struct BackgroundParams {
    #[param(flatten, choice, loc_key = "what")]
    background_type: BackgroundType,
}

#[derive(ParameterImpl)]
enum BackgroundType {
    SolidColor {
        background_color: Color,
    },
    HorizontalGradient {
        start_color: Color,
        end_color: Color,
    },
    Test(Color),
}

impl Default for BackgroundType {
    fn default() -> Self {
        Self::SolidColor {
            background_color: Default::default(),
        }
    }
}

#[derive(ParameterImpl, Default)]
struct TextParams {
    text_objects: Vec<TextObject>,
}

#[derive(ParameterImpl, Default)]
struct TextObject {
    text: String,
    font: DynamicChoice,
    text_scale: (f64, f64),
    #[param(choice)]
    text_alignment: TextAlignment,
    text_padding: u32,
    text_offset: (i32, i32),
    text_color: Color,
    #[param(flatten)]
    text_shadow: Option<TextShadow>,
}

#[derive(ParameterImpl, Default)]
struct TextShadow {
    text_shadow_color: Color,
    text_shadow_offset: (i32, i32),
}

#[derive(ParameterImpl)]
enum TextAlignment {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::MiddleCenter
    }
}
