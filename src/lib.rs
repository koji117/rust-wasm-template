mod sierpinski;
use std::rc::Rc;
use std::sync::Mutex;
use sierpinski::sierpinski;
use wasm_bindgen::prelude::*;
use web_sys::{console, window, CanvasRenderingContext2d};

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let context = create_context();

    wasm_bindgen_futures::spawn_local(async move {
        // Create a channel
        // The part <Result<(), JsValue>> specifies the type of value that the oneshot channel will transmit.
        // Result is a type that represents either a success (Ok) or an error (Err).
        // Here, () (an empty tuple) is used to signify that on success, no value is returned.
        // JsValue is used in the error case to represent any JavaScript value that could be returned as an error
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        // Make sure that one thread can access success_tx at a time with Mutex
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();

        // ok() returns Option and this Option will be accessed though and_then()
        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok()
                .and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok()
                .and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        image.set_src("Idle (1).png");

        // Wait draw image until loaded
        success_rx.await;
        context.draw_image_with_html_image_element(&image, 0.0, 0.0);

        sierpinski(
            &context,
            [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)],
            (0, 255, 0),
            10,
        );
    });

    Ok(())
}

fn create_context() -> CanvasRenderingContext2d {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    context
}
