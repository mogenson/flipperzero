//! Demonstrates use of the ViewDispatcher module.
//!
//! This app prompts the user for a name then says hello.

#![no_main]
#![no_std]

extern crate flipperzero_alloc as alloc;
extern crate flipperzero_rt;

use alloc::boxed::Box;
use core::{
    ffi::{c_char, c_void, CStr},
    ptr::NonNull,
};
use flipperzero::furi::string::FuriString;
use flipperzero::gui::{
    canvas::{Align, Font},
    view_dispatcher::{
        ViewDispatcher, ViewDispatcherBuilder, ViewDispatcherCallbacks, ViewDispatcherType,
    },
};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;

manifest!(name = "ViewDispatcher example");
entry!(main);

enum Views {
    Widget = 0,
    TextInput = 1,
}

struct ViewDispatcherState();

impl ViewDispatcherCallbacks for ViewDispatcherState {
    fn on_navigation_event(&mut self, view_dispatcher: &mut ViewDispatcher) -> bool {
        view_dispatcher.stop();
        true
    }
}

struct TextInputState<'a> {
    view_dispatcher: &'a mut ViewDispatcher,
    widget: NonNull<sys::Widget>,
    name: [c_char; 16],
}

pub unsafe extern "C" fn text_input_callback(context: *mut c_void) {
    let text_input_state = context as *mut TextInputState;

    let mut message = FuriString::from("Hello ");
    message.push_c_str(CStr::from_ptr((*text_input_state).name.as_ptr()));

    sys::widget_add_string_element(
        (*text_input_state).widget.as_ptr(),
        64,
        32,
        Align::Center.into(),
        Align::Center.into(),
        Font::Primary.into(),
        message.as_c_str().as_ptr(),
    );

    (*text_input_state)
        .view_dispatcher
        .switch_to_view(Views::Widget as u32);
}

fn main(_args: *mut u8) -> i32 {
    let mut view_dispatcher =
        ViewDispatcherBuilder::new(ViewDispatcherType::Fullscreen, ViewDispatcherState());

    let widget = unsafe { NonNull::new_unchecked(sys::widget_alloc()) };
    view_dispatcher.add_view(Views::Widget as u32, unsafe {
        sys::widget_get_view(widget.as_ptr())
    });

    let text_input = unsafe { NonNull::new_unchecked(sys::text_input_alloc()) };
    view_dispatcher.add_view(Views::TextInput as u32, unsafe {
        sys::text_input_get_view(text_input.as_ptr())
    });

    let text_input_state = Box::into_raw(Box::new(TextInputState {
        view_dispatcher: &mut view_dispatcher,
        widget: widget,
        name: Default::default(),
    }));

    unsafe {
        sys::text_input_set_header_text(text_input.as_ptr(), sys::c_string!("Enter your name"));
        sys::text_input_set_result_callback(
            text_input.as_ptr(),
            Some(text_input_callback),
            text_input_state as *mut c_void,
            (*text_input_state).name.as_mut_ptr(),
            (*text_input_state).name.len(),
            true,
        );
    }

    view_dispatcher.switch_to_view(Views::TextInput as u32);
    view_dispatcher.run();

    view_dispatcher.remove_view(Views::Widget as u32);
    unsafe { sys::widget_free(widget.as_ptr()) };

    view_dispatcher.remove_view(Views::TextInput as u32);
    unsafe { sys::text_input_free(text_input.as_ptr()) };

    0
}
