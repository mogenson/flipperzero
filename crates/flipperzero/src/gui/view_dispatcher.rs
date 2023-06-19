//! View Dispatcher.

use flipperzero_sys as sys;

use core::ffi::{c_char, c_void};
use core::ptr::NonNull;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

const RECORD_GUI: *const c_char = sys::c_string!("gui");

//pub type ViewDispatcherCustomEventCallback = dyn FnMut(u32) -> bool;
type NavigationEventCallback = Box<dyn FnMut(&mut ViewDispatcher) -> bool>;

/// ViewDispatcherType
#[derive(Debug, Clone, Copy)]
pub enum ViewDispatcherType {
    Desktop,
    Window,
    Fullscreen,
}

impl ViewDispatcherType {
    pub fn to_sys(&self) -> sys::ViewDispatcherType {
        match self {
            Self::Desktop => sys::ViewDispatcherType_ViewDispatcherTypeDesktop,
            Self::Window => sys::ViewDispatcherType_ViewDispatcherTypeWindow,
            Self::Fullscreen => sys::ViewDispatcherType_ViewDispatcherTypeFullscreen,
        }
    }
}

// struct Context {
//     custom_event_callback: Option<fn(u32) -> bool>,
//     navigation_event_callback: Option<fn() -> bool>,
// }

/// ViewDispatcher
pub struct ViewDispatcher {
    data: NonNull<sys::ViewDispatcher>,
    navigation_event_callback: Option<NavigationEventCallback>,
}

impl Drop for ViewDispatcher {
    fn drop(&mut self) {
        unsafe {
            sys::view_dispatcher_free(self.data.as_ptr());
            sys::furi_record_close(RECORD_GUI);
        }
    }
}

impl ViewDispatcher {
    /// Creates and initializes a new `ViewDispatcher`
    pub fn new(view_dispatcher_type: ViewDispatcherType) -> Self {
        unsafe {
            let mut view_dispatcher = ViewDispatcher {
                data: NonNull::new_unchecked(sys::view_dispatcher_alloc()),
                navigation_event_callback: None,
            };
            sys::view_dispatcher_set_event_callback_context(
                view_dispatcher.data.as_ptr(),
                &mut view_dispatcher as *mut _ as *mut c_void,
            );
            sys::view_dispatcher_enable_queue(view_dispatcher.data.as_ptr());
            let gui = sys::furi_record_open(RECORD_GUI) as *mut sys::Gui;
            sys::view_dispatcher_attach_to_gui(
                view_dispatcher.data.as_ptr(),
                gui,
                view_dispatcher_type.to_sys(),
            );
            view_dispatcher
        }
    }

    /// Register a view
    pub fn add_view(&self, view_id: u32, view: *mut sys::View) {
        unsafe { sys::view_dispatcher_add_view(self.data.as_ptr(), view_id, view) };
    }

    /// Remove a view
    pub fn remove_view(&self, view_id: u32) {
        unsafe { sys::view_dispatcher_remove_view(self.data.as_ptr(), view_id) };
    }

    /// Switch to a view
    pub fn switch_to_view(&self, view_id: u32) {
        unsafe { sys::view_dispatcher_switch_to_view(self.data.as_ptr(), view_id) };
    }

    /// Start view dispatcher event loop. This method blocks until `stop()` is called.
    pub fn run(&mut self) {
        unsafe { sys::view_dispatcher_run(self.data.as_ptr()) };
    }

    /// Send message to exit view dispatcher event loop.
    pub fn stop(&self) {
        unsafe { sys::view_dispatcher_stop(self.data.as_ptr()) };
    }

    /// Send view dispatcher's view port to front
    pub fn send_to_front(&self) {
        unsafe { sys::view_dispatcher_send_to_front(self.data.as_ptr()) };
    }

    /// Send view dispatcher's view port to back
    pub fn send_to_back(&self) {
        unsafe { sys::view_dispatcher_send_to_back(self.data.as_ptr()) };
    }

    /// Send a custom event to the the custom event callback handler
    pub fn send_custom_event(&self, event: u32) {
        unsafe { sys::view_dispatcher_send_custom_event(self.data.as_ptr(), event) };
    }

    /// Register a custom event callback handler
    // pub fn set_custom_event_callback(&self, closure: fn(u32) -> bool) {
    //     unsafe extern "C" fn trampoline(context: *mut c_void, event: u32) -> bool {
    //         let context = context as *mut Context;
    //         if let Some(custom_event_callback) = &mut (*context).custom_event_callback {
    //             return custom_event_callback(event);
    //         }
    //         false
    //     }

    //     self.context.custom_event_callback = Some(closure);
    //     unsafe {
    //         sys::view_dispatcher_set_custom_event_callback(self.data.as_ptr(), Some(trampoline))
    //     };
    // }

    /// Register a navigation event callback handler
    #[cfg(feature = "alloc")]
    pub fn set_navigation_event_callback<F>(&mut self, f: F)
    where
        F: FnMut(&mut ViewDispatcher) -> bool + 'static,
    {
        let callback: NavigationEventCallback = Box::new(f);
        unsafe extern "C" fn trampoline(context: *mut c_void) -> bool {
            //let mut closure = unsafe { Box::from_raw(context as *mut Closure) };
            // closure()
            let view_dispatcher = &mut *(context as *mut ViewDispatcher);
            let mut navigation_event_callback = view_dispatcher.navigation_event_callback.take();
            let mut ret = false;
            if let Some(ref mut navigation_event_callback) = navigation_event_callback {
                ret = navigation_event_callback(view_dispatcher);
            }
            view_dispatcher.navigation_event_callback = navigation_event_callback;
            return ret;
        }

        self.navigation_event_callback = Some(callback);
        unsafe {
            // sys::view_dispatcher_set_event_callback_context(
            //     self.data.as_ptr(),
            //     Box::into_raw(closure) as *mut c_void,
            // );
            sys::view_dispatcher_set_navigation_event_callback(self.data.as_ptr(), Some(trampoline))
        };
    }
}
