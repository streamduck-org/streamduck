use mlua::{UserData, UserDataFields};
use crate::api::value_type::ValueType;
use crate::api::device::input::InputEvent;

pub struct CommonLuaStatic;

impl UserData for CommonLuaStatic {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_function_get("buttonEvent", |_, _| {
            Ok(InputEvent {
                name: "button".to_string(),
                description: Some("A simple button up or down event".to_string()),
                signature: ValueType::Boolean,
            })
        });

        fields.add_field_function_get("toggleEvent", |_, _| {
            Ok(InputEvent {
                name: "toggle".to_string(),
                description: Some("Toggle event with state of the toggle".to_string()),
                signature: ValueType::Boolean,
            })
        });

        fields.add_field_function_get("analogButtonEvent", |_, _| {
            Ok(InputEvent {
                name: "analogButton".to_string(),
                description: Some("Button with pressure sensitivity, should pass values between 0 and 1".to_string()),
                signature: ValueType::Double,
            })
        });

        fields.add_field_function_get("sliderEvent", |_, _| {
            Ok(InputEvent {
                name: "slider".to_string(),
                description: Some("Slider event, should pass values between 0 and 1".to_string()),
                signature: ValueType::Double,
            })
        });

        fields.add_field_function_get("knobEvent", |_, _| {
            Ok(InputEvent {
                name: "knob".to_string(),
                description: Some("Finite knob event, should pass values between 0 and 1".to_string()),
                signature: ValueType::Double,
            })
        });

        fields.add_field_function_get("encoderEvent", |_, _| {
            Ok(InputEvent {
                name: "encoder".to_string(),
                description: Some("Infinite knob/encoder event, should pass positive integer when turned clockwise, negative when counter clockwise".to_string()),
                signature: ValueType::Integer,
            })
        });

        fields.add_field_function_get("touchScreenPressEvent", |_, _| {
            Ok(InputEvent {
                name: "touchScreenPress".to_string(),
                description: Some("Touch screen press event, should pass two unsigned integers and potential press state".to_string()),
                signature: ValueType::Tuple(vec![ValueType::UnsignedInteger, ValueType::UnsignedInteger, ValueType::Nullable(ValueType::Boolean.into())]),
            })
        });

        fields.add_field_function_get("touchScreenLongPressEvent", |_, _| {
            Ok(InputEvent {
                name: "touchScreenLongPress".to_string(),
                description: Some("Touch screen long press event, should pass two unsigned integers".to_string()),
                signature: ValueType::Tuple(vec![ValueType::UnsignedInteger, ValueType::UnsignedInteger]),
            })
        });

        fields.add_field_function_get("touchScreenDragEvent", |_, _| {
            Ok(InputEvent {
                name: "touchScreenDrag".to_string(),
                description: Some("Touch screen drag event, drag should start from touchScreenPress and end with that having false state, should pass two unsigned integers".to_string()),
                signature: ValueType::Tuple(vec![ValueType::UnsignedInteger, ValueType::UnsignedInteger]),
            })
        });

        fields.add_field_function_get("touchScreenSwipeEvent", |_, _| {
            Ok(InputEvent {
                name: "touchScreenSwipe".to_string(),
                description: Some("Touch screen swipe event, for touch screens that can't tell current state of dragging, just indicates finger moving from one spot to another".to_string()),
                signature: ValueType::Tuple(vec![
                    ValueType::Tuple(vec![ValueType::UnsignedInteger, ValueType::UnsignedInteger]),
                    ValueType::Tuple(vec![ValueType::UnsignedInteger, ValueType::UnsignedInteger])
                ]),
            })
        });

        fields.add_field_function_get("joystickEvent", |_, _| {
            Ok(InputEvent {
                name: "joystick".to_string(),
                description: Some("Joystick move event, should be passed whenever joystick changes position, expecting floating values from -1 to 1".to_string()),
                signature: ValueType::Tuple(vec![ValueType::Double, ValueType::Double]),
            })
        });

        fields.add_field_function_get("touchpadEvent", |_, _| {
            Ok(InputEvent {
                name: "touchpad".to_string(),
                description: Some("Touchpad relative move event, expecting 2 integers indicating distance in pixels".to_string()),
                signature: ValueType::Tuple(vec![ValueType::Integer, ValueType::Integer]),
            })
        });
    }
}