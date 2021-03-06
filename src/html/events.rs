//! https://developer.mozilla.org/en-US/docs/Web/Events
use log::*;
use mapper::*;

pub use sauron_vdom::{
    builder::{on, on_with_extractor},
    event::{
        Coordinate, InputEvent, KeyEvent, Modifier, MouseButton, MouseEvent,
    },
    Callback,
};
use wasm_bindgen::JsCast;

/// TODO: May not be needed if we can use fully generic event, when passed in the callback
///
/// This module convert browser events into sauron_vdom generic event
pub mod mapper {
    use log::*;

    use sauron_vdom::event::{
        Coordinate, InputEvent, KeyEvent, Modifier, MouseButton, MouseEvent,
    };
    use wasm_bindgen::JsCast;
    use web_sys::{self, EventTarget, HtmlInputElement, HtmlTextAreaElement};

    pub fn mouse_event_mapper(event: crate::Event) -> MouseEvent {
        let mouse: &web_sys::MouseEvent =
            event.0.dyn_ref().expect("Unable to cast to mouse event");

        let coordinate = Coordinate {
            client_x: mouse.client_x(),
            client_y: mouse.client_y(),
            movement_x: mouse.movement_x(),
            movement_y: mouse.movement_y(),
            offset_x: mouse.offset_x(),
            offset_y: mouse.offset_y(),
            screen_x: mouse.screen_x(),
            screen_y: mouse.screen_y(),
            x: mouse.x(),
            y: mouse.y(),
        };
        let modifier = Modifier {
            alt_key: mouse.alt_key(),
            ctrl_key: mouse.ctrl_key(),
            meta_key: mouse.meta_key(),
            shift_key: mouse.shift_key(),
        };
        let buttons = match mouse.button() {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Left,
            3 => MouseButton::WheelUp,
            4 => MouseButton::WheelDown,
            _ => Default::default(), // defaults to left
        };
        let r#type = match &*event.0.type_() {
            "click" => "click",
            "mouseup" => "mouseup",
            "mousedown" => "mousedown",
            "mousemove" => "mousemove",
            "dblclick" => "dblclick",
            _e => {
                error!("unhandled event type: {}", _e);
                panic!("unhandled event type: {}", _e);
            }
        };
        MouseEvent {
            r#type,
            coordinate,
            modifier,
            buttons,
        }
    }

    pub fn keyboard_event_mapper(event: crate::Event) -> KeyEvent {
        if let Some(key_event) = event.0.dyn_ref::<web_sys::KeyboardEvent>() {
            let modifier = Modifier {
                alt_key: key_event.alt_key(),
                ctrl_key: key_event.ctrl_key(),
                meta_key: key_event.meta_key(),
                shift_key: key_event.shift_key(),
            };
            KeyEvent {
                key: key_event.key(),
                modifier,
                repeat: key_event.repeat(),
                location: key_event.location(),
            }
        } else {
            //FIXME: not a keyboard event just make something up,
            //maybe make the return type optional?
            KeyEvent::default()
        }
    }

    pub fn input_event_mapper(event: crate::Event) -> InputEvent {
        let target: EventTarget =
            event.0.target().expect("Unable to get event target");
        let input_event = if let Some(input) =
            target.dyn_ref::<HtmlInputElement>()
        {
            Some(InputEvent {
                value: input.value(),
            })
        } else if let Some(textarea) = target.dyn_ref::<HtmlTextAreaElement>() {
            Some(InputEvent {
                value: textarea.value(),
            })
        } else {
            None
        };

        input_event.expect(
            "Expecting an input event from input element or textarea element",
        )
    }
}

macro_rules! declare_events {
    ( $(
         $(#[$attr:meta])*
         $name:ident : $event:ident => $ret:ty =>  $mapper:expr;
       )*
     ) => {
        $(
            $(#[$attr])*
            #[inline]
            pub fn $name<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
                where CB: Fn($ret)-> MSG +'static,
                      MSG: 'static,
                {
                    on_with_extractor(stringify!($event), $mapper, cb)
                }
         )*
    };

    ( $(
         $(#[$attr:meta])*
         $name:ident : $event:ident;
       )*
     ) => {
        $(
            $(#[$attr])*
            #[inline]
            pub fn $name<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
                where CB: Fn(()) -> MSG + 'static,
                      MSG: 'static,
                {
                    on_with_extractor(stringify!($event), |_|{}, cb)
                }
         )*
    }
}

#[inline]
pub fn onscroll<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
where
    CB: Fn((i32, i32)) -> MSG + 'static,
    MSG: 'static,
{
    let webevent_to_scroll_offset = |event: crate::Event| {
        let target = event.0.target().expect("can't get target");
        let element: &web_sys::Element =
            target.dyn_ref().expect("Cant cast to Element");
        let scroll_top = element.scroll_top();
        let scroll_left = element.scroll_left();
        (scroll_top, scroll_left)
    };
    on_with_extractor("scroll", webevent_to_scroll_offset, cb)
}

pub fn onresize<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
where
    CB: Fn((i32, i32)) -> MSG + 'static,
    MSG: 'static,
{
    trace!("resizing..");
    let target_size_fn = |event: crate::Event| {
        let target = event.0.target().expect("can't get target");
        let element: &web_sys::Element =
            target.dyn_ref().expect("Cant cast to Element");
        let target_width = element.client_width();
        let target_height = element.client_height();
        (target_width, target_height)
    };
    on_with_extractor("resize", target_size_fn, cb)
}

/// on click with both prevent_default and stop_propagation turned on
pub fn onclick_prevent_all<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
where
    CB: Fn(MouseEvent) -> MSG + 'static,
    MSG: 'static,
{
    onclick_with(true, true, cb)
}

/// on click with prevent default on
pub fn onclick_prevent_default<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
where
    CB: Fn(MouseEvent) -> MSG + 'static,
    MSG: 'static,
{
    onclick_with(true, false, cb)
}

/// on click with stop_propagation on
pub fn onclick_stop_propagation<CB, MSG>(cb: CB) -> crate::Attribute<MSG>
where
    CB: Fn(MouseEvent) -> MSG + 'static,
    MSG: 'static,
{
    onclick_with(false, true, cb)
}

/// a version of on_click where you can choose to manipulate the event
/// whether to stop_progating that event to parent elements or
/// prevent_default the default behavior as with the case for a href links
pub fn onclick_with<CB, MSG>(
    prevent_default: bool,
    stop_propagation: bool,
    cb: CB,
) -> crate::Attribute<MSG>
where
    CB: Fn(MouseEvent) -> MSG + 'static,
    MSG: 'static,
{
    on_with_extractor(
        "click",
        move |event: crate::Event| {
            if prevent_default {
                event.prevent_default();
            }
            if stop_propagation {
                event.stop_propagation();
            }
            mouse_event_mapper(event)
        },
        cb,
    )
}

// Mouse events
declare_events! {
    onclick : click => MouseEvent => mouse_event_mapper;
    onauxclick : auxclick => MouseEvent => mouse_event_mapper;
    oncontextmenu : contextmenu => MouseEvent => mouse_event_mapper ;
    ondblclick  : dblclick => MouseEvent => mouse_event_mapper;
    onmousedown : mousedown => MouseEvent => mouse_event_mapper;
    onmouseenter : mouseenter => MouseEvent => mouse_event_mapper;
    onmouseleave : mouseleave => MouseEvent => mouse_event_mapper;
    onmousemove : mousemove => MouseEvent => mouse_event_mapper;
    onmouseover : mouseover => MouseEvent => mouse_event_mapper;
    onmouseout : mouseout => MouseEvent => mouse_event_mapper;
    onmouseup : mouseup => MouseEvent => mouse_event_mapper;
    onpointerlockchange : pointerlockchange =>MouseEvent => mouse_event_mapper;
    onpointerlockerror : pointerlockerror =>MouseEvent => mouse_event_mapper;
    onselect : select => MouseEvent => mouse_event_mapper;
    onwheel : wheel => MouseEvent => mouse_event_mapper;
    ondoubleclick : dblclick => MouseEvent => mouse_event_mapper;
}

// keyboard events
declare_events! {
    onkeydown : keydown => KeyEvent => keyboard_event_mapper;
    onkeypress : keypress => KeyEvent => keyboard_event_mapper;
    onkeyup : keyup =>KeyEvent => keyboard_event_mapper;
}

// focus events
declare_events! {
    onfocus : focus;
    onblur : blur;
}

// form events
declare_events! {
    onreset : reset;
    onsubmit : submit;
}

declare_events! {
    oninput : input => InputEvent => input_event_mapper;
    onchange : change => InputEvent => input_event_mapper;
}
declare_events! {
    onbroadcast : broadcast;
    //CheckboxStateChange
    onhashchange : hashchange;
    //RadioStateChange
    onreadystatechange : readystatechange;
    //ValueChange
}
