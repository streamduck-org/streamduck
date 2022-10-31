use std::any::{TypeId};
use crate::events::{Event, EventInstance};

/// Casts event object into concrete type
pub fn cast_event<T: Event>(event: &dyn EventInstance) -> Option<T> {
    if event.type_id() == TypeId::of::<T>() {
        let event = unsafe {
            let ptr: *const dyn EventInstance = event;
            (&*(ptr as *const T)).clone()
        };

        Some(event)
    } else {
        None
    }
}