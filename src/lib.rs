mod sierpinski;
#[macro_use]
mod browser;

use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}
// [derive(Deserialize)] macro means we can use Sheet as a target for deserializing the JSON,  and HashMap and String work automatically
#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

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

    let context = browser::context().expect("Could not get context");

    // Spawning the local future
    browser::spawn_local(async move {
        let sheet: Sheet = browser::fetch_json("rhb.json")
            .await
            .expect("Could not convert rhb.json into a Sheet struct")
            .into_serde()
            .expect("Could not convert rhb.json into a Sheet struct");

        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        // Make sure that one thread can access success_tx at a time with Mutex
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();

        // ok() returns Option and this Option will be accessed though and_then()
        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        image.set_src("rhb.png");
        success_rx.await;

        // Box<T>: A pointer type for heap allocation. Box allows you to store data on the heap rather than the stack. What remains on the stack is the pointer to the heap data. Using Box is a way to allocate large amounts of data or to keep ownership of data across different parts of your program.
        // dyn: A keyword used to denote a dynamic dispatch to a trait object. When you use dyn, you're telling Rust that you want to call methods on a type that implements a particular trait, but you're not specifying what the concrete type is. This enables polymorphism in Rust.
        // FnMut(): A trait bound that specifies the closure or function pointer takes no parameters (()) and returns nothing (()). The FnMut trait is used for closures that might need to mutate their captured variables. It's one of the three "callable" traits, alongside Fn and FnOnce.
        //  Fn: Requires that the closure does not mutate any captured variables or move out of them and can be called multiple times.
        //  FnMut: Allows the closure to mutate its captured variables and can be called multiple times.
        //  FnOnce: Allows the closure to consume (move) its captured variables and can be called only once.
        //  wrap() requires Box, and there isn't enough information for the compiler to infer the type.
        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);
            context.clear_rect(0.0, 0.0, 600.0, 600.0);

            let sprite = sheet.frames.get(&frame_name).expect("Cell not found");
            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                300.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );
        }) as Box<dyn FnMut()>);
        let window = browser::window().expect("No Window Found");
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
        // Forget the closure that we passed into setInterval so that Rust doesn't destroy it when we leave the scope of this future
        interval_callback.forget();
    });

    Ok(())
}
