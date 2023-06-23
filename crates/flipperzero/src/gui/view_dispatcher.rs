//! ViewDispatcher APIs

use crate::gui::Gui;
use alloc::boxed::Box;
use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use flipperzero_sys::{
    self as sys, ViewDispatcher as SysViewDispatcher, ViewDispatcherType as SysViewDispatcherType,
};

/// ViewDispatcherType
#[derive(Debug, Clone, Copy)]
pub enum ViewDispatcherType {
    /// Desktop view dispatcher type.
    Desktop,
    /// Window view dispatcher type.
    Window,
    /// Fullscreen view dispatcher type.
    Fullscreen,
}

impl From<ViewDispatcherType> for SysViewDispatcherType {
    fn from(value: ViewDispatcherType) -> Self {
        match value {
            ViewDispatcherType::Desktop => sys::ViewDispatcherType_ViewDispatcherTypeDesktop,
            ViewDispatcherType::Window => sys::ViewDispatcherType_ViewDispatcherTypeWindow,
            ViewDispatcherType::Fullscreen => sys::ViewDispatcherType_ViewDispatcherTypeFullscreen,
        }
    }
}

/// System ViewDispatcher
pub struct ViewDispatcher {
    raw: NonNull<SysViewDispatcher>,
}

pub struct ViewDispatcherBuilder<C: ViewDispatcherCallbacks> {
    raw: NonNull<SysViewDispatcher>,
    callbacks: NonNull<C>,
}

type Context<C> = ViewDispatcherBuilder<C>;

impl<C: ViewDispatcherCallbacks> Context<C> {
    /// Creates and initializes a new `ViewDispatcher` with callbacks
    pub fn new(view_dispatcher_type: ViewDispatcherType, callbacks: C) -> ViewDispatcher {
        let raw = unsafe { NonNull::new_unchecked(sys::view_dispatcher_alloc()) };
        let callbacks = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(callbacks))) };

        let view_dispatcher = ViewDispatcher { raw };
        let context = Box::into_raw(Box::new(Context { raw, callbacks })).cast();

        let raw = raw.as_ptr();
        unsafe { sys::view_dispatcher_enable_queue(raw) };
        unsafe { sys::view_dispatcher_set_event_callback_context(raw, context) };

        let gui = Gui::new();
        unsafe {
            sys::view_dispatcher_attach_to_gui(raw, gui.as_raw(), view_dispatcher_type.into())
        };

        pub unsafe extern "C" fn dispatch_custom_event<C: ViewDispatcherCallbacks>(
            context: *mut c_void,
            event: u32,
        ) -> bool {
            let context: *mut Context<C> = context.cast();
            let mut view_dispatcher = ViewDispatcher {
                raw: (*context).raw,
            };
            (*context)
                .callbacks
                .as_mut()
                .on_custom_event(&mut view_dispatcher, event)
        }

        if !ptr::eq(
            C::on_custom_event as *const c_void,
            <() as ViewDispatcherCallbacks>::on_custom_event as *const c_void,
        ) {
            let callback = Some(dispatch_custom_event::<C> as _);
            unsafe { sys::view_dispatcher_set_custom_event_callback(raw, callback) };
        }

        pub unsafe extern "C" fn dispatch_navigation_event<C: ViewDispatcherCallbacks>(
            context: *mut c_void,
        ) -> bool {
            let context: *mut Context<C> = context.cast();
            let mut view_dispatcher = ViewDispatcher {
                raw: (*context).raw,
            };
            (*context)
                .callbacks
                .as_mut()
                .on_navigation_event(&mut view_dispatcher)
        }

        if !ptr::eq(
            C::on_navigation_event as *const c_void,
            <() as ViewDispatcherCallbacks>::on_navigation_event as *const c_void,
        ) {
            let callback = Some(dispatch_navigation_event::<C> as _);
            unsafe { sys::view_dispatcher_set_navigation_event_callback(raw, callback) };
        }

        pub unsafe extern "C" fn dispatch_tick_event<C: ViewDispatcherCallbacks>(
            context: *mut c_void,
        ) {
            let context: *mut Context<C> = context.cast();
            let mut view_dispatcher = ViewDispatcher {
                raw: (*context).raw,
            };
            (*context)
                .callbacks
                .as_mut()
                .on_tick_event(&mut view_dispatcher);
        }

        if !ptr::eq(
            C::on_tick_event as *const c_void,
            <() as ViewDispatcherCallbacks>::on_tick_event as *const c_void,
        ) {
            let callback = Some(dispatch_tick_event::<C> as _);
            unsafe {
                sys::view_dispatcher_set_tick_event_callback(raw, callback, C::get_tick_period())
            };
        }

        view_dispatcher
    }
}

impl ViewDispatcher {
    /// Start view dispatcher event loop. This method blocks until `stop()` is called.
    pub fn run(&mut self) {
        unsafe { sys::view_dispatcher_run(self.raw.as_ptr()) };
    }

    /// Send message to exit view dispatcher event loop.
    pub fn stop(&mut self) {
        unsafe { sys::view_dispatcher_stop(self.raw.as_ptr()) };
    }

    /// Register a view
    pub fn add_view(&mut self, view_id: u32, view: *mut sys::View) {
        unsafe { sys::view_dispatcher_add_view(self.raw.as_ptr(), view_id, view) };
    }

    /// Remove a view
    pub fn remove_view(&mut self, view_id: u32) {
        unsafe { sys::view_dispatcher_remove_view(self.raw.as_ptr(), view_id) };
    }

    /// Switch to a view
    pub fn switch_to_view(&mut self, view_id: u32) {
        unsafe { sys::view_dispatcher_switch_to_view(self.raw.as_ptr(), view_id) };
    }

    /// Send view dispatcher's view port to front
    pub fn send_to_front(&mut self) {
        unsafe { sys::view_dispatcher_send_to_front(self.raw.as_ptr()) };
    }

    /// Send view dispatcher's view port to back
    pub fn send_to_back(&mut self) {
        unsafe { sys::view_dispatcher_send_to_back(self.raw.as_ptr()) };
    }

    /// Send a custom event to the the custom event callback handler
    pub fn send_custom_event(&mut self, event: u32) {
        unsafe { sys::view_dispatcher_send_custom_event(self.raw.as_ptr(), event) };
    }
}

pub trait ViewDispatcherCallbacks {
    fn get_tick_period() -> u32 {
        0
    }
    fn on_custom_event(&mut self, _view_dispatcher: &mut ViewDispatcher, _event: u32) -> bool {
        false
    }
    fn on_navigation_event(&mut self, _view_dispatcher: &mut ViewDispatcher) -> bool {
        false
    }
    fn on_tick_event(&mut self, _view_dispatcher: &mut ViewDispatcher) {}
}

impl ViewDispatcherCallbacks for () {}
