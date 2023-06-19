//! Demonstrates use of the ViewDispatcher module.
//!
//! This app prompts the user for a name then says hello.

#![no_main]
#![no_std]

extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use alloc::boxed::Box;
use core::ffi::{c_char, c_void, CStr};
use core::ptr::NonNull;
use flipperzero::furi::string::FuriString;
use flipperzero::gui::view_dispatcher::{self, ViewDispatcher, ViewDispatcherType};
use flipperzero::println;
use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;

manifest!(name = "Rust ViewDispatcher example");
entry!(main);

enum AppView {
    Widget = 0,
    TextInput = 1,
}

// struct App {
//     name: [c_char; 16],
//     view_dispatcher: ViewDispatcher,
//     widget: NonNull<sys::Widget>,
//     text_input: NonNull<sys::TextInput>,
// }

// impl App {
//     pub fn new() -> Self {
//         App {
//             name: Default::default(),
//             view_dispatcher: ViewDispatcher::new(ViewDispatcherType::Fullscreen),
//             widget: unsafe { NonNull::new_unchecked(sys::widget_alloc()) },
//             text_input: unsafe { NonNull::new_unchecked(sys::text_input_alloc()) },
//         }
//     }
// }

// impl Drop for App {
//     fn drop(&mut self) {
//         unsafe {
//             sys::widget_free(self.widget.as_ptr());
//             sys::text_input_free(self.text_input.as_ptr());
//         }
//     }
// }

// pub unsafe extern "C" fn text_input_callback(context: *mut c_void) {
//     //let app = context as *mut App;
//     let app = Rc::from_raw(context as *const RefCell<App>);
//     let mut message = FuriString::from("Hello ");
//     message.push_c_str(CStr::from_ptr(app.borrow().name.as_ptr()));
//     sys::widget_add_string_element(
//         app.borrow().widget.as_ptr(),
//         128 / 2,
//         64 / 2,
//         sys::Align_AlignCenter,
//         sys::Align_AlignCenter,
//         sys::Font_FontPrimary,
//         message.as_c_str().as_ptr(),
//     );
//     app.borrow_mut()
//         .view_dispatcher
//         .switch_to_view(AppView::Widget as u32);
// }

//pub unsafe extern "C" fn navigation_event_callback(context: *mut c_void) -> bool {
//    let view_dispatcher = context as *mut sys::ViewDispatcher;
//    sys::view_dispatcher_stop(view_dispatcher);
//    sys::view_dispatcher_remove_view(view_dispatcher, AppView::Widget as u32);
//    sys::view_dispatcher_remove_view(view_dispatcher, AppView::TextInput as u32);
//    true
//}

use alloc::rc::Rc;
use core::cell::RefCell;
fn main(_args: *mut u8) -> i32 {
    ///// TODO split up APP into separate components
    //let app = Rc::new(RefCell::new(App::new()));
    // let view_dispatcher = Rc::new(ViewDispatcher::new(ViewDispatcherType::Fullscreen));
    let mut view_dispatcher = ViewDispatcher::new(ViewDispatcherType::Fullscreen);
    let widget = unsafe { NonNull::new_unchecked(sys::widget_alloc()) };

    view_dispatcher.add_view(AppView::Widget as u32, unsafe {
        sys::widget_get_view(widget.as_ptr())
    });

    /*
        app.borrow_mut()
            .view_dispatcher
            .add_view(AppView::TextInput as u32, unsafe {
                sys::text_input_get_view(app.borrow().text_input.as_ptr())
            });
    */

    //let vd2 = Rc::clone(&view_dispatcher);
    view_dispatcher.set_navigation_event_callback(|vd2| {
        println!("navigation event callback");
        vd2.stop();
        vd2.remove_view(AppView::Widget as u32);
        //view_dispatcher.remove_view(AppView::TextInput as u32);
        true
    });

    /*
    unsafe {
        sys::text_input_reset(app.borrow().text_input.as_ptr());
        sys::text_input_set_header_text(
            app.borrow().text_input.as_ptr(),
            sys::c_string!("Enter your name"),
        );

        let app3 = Rc::clone(&app);
        sys::text_input_set_result_callback(
            app.borrow().text_input.as_ptr(),
            Some(text_input_callback),
            Rc::into_raw(app3) as *mut c_void,
            app.borrow_mut().name.as_mut_ptr(),
            app.borrow().name.len(),
            true,
        );
    }
    */

    view_dispatcher.switch_to_view(AppView::Widget as u32);
    view_dispatcher.run();

    unsafe { sys::widget_free(widget.as_ptr()) };

    0
}
