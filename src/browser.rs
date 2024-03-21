use anyhow::anyhow;
use std::future::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{closure::WasmClosureFnOnce, JsCast, JsValue};
use wasm_bindgen::closure::WasmClosure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlImageElement, Response, Window,
};
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )*).into());
    }
}

pub fn window() -> Result<Window, anyhow::Error> {
    web_sys::window().ok_or_else(|| anyhow!("No Window Found"))
}

pub fn document() -> Result<Document, anyhow::Error> {
    window()?
        .document()
        .ok_or_else(|| anyhow!("No Document Found"))
}

pub fn canvas() -> Result<HtmlCanvasElement, anyhow::Error> {
    document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| anyhow!("No Canvas Element found with ID 'canvas'"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|element| anyhow!("Error converting {:#?} to HtmlCanvasElement", element))
}

pub fn context() -> Result<CanvasRenderingContext2d, anyhow::Error> {
    canvas()?
        .get_context("2d")
        .map_err(|js_value| anyhow!("Error getting 2d context {:#?}", js_value))?
        .ok_or_else(|| anyhow!("No 2d context found"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|element| {
            anyhow!(
                "Error converting {:#?} to CanvasRenderingContext2d",
                element
            )
        })
}

pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

pub async fn fetch_with_str(resource: &str) -> Result<JsValue, anyhow::Error> {
    JsFuture::from(window()?.fetch_with_str(resource))
        .await
        .map_err(|err| anyhow!("error fetching {:#?}", err))
}

pub async fn fetch_json(json_path: &str) -> Result<JsValue, anyhow::Error> {
    let resp_value = fetch_with_str(json_path).await?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|element| anyhow!("Error converting {:#?} to Response", element))?;

    JsFuture::from(
        resp.json()
            .map_err(|err| anyhow!("Could not get JSON from response {:#?}", err))?,
    )
    .await
    .map_err(|err| anyhow!("error fetching JSON {:#?}", err))
}

pub fn new_image() -> Result<HtmlImageElement, anyhow::Error> {
    HtmlImageElement::new().map_err(|err| anyhow!("Could not create HtmlImageElement: {:#?}", err))
}

// FnMut: Allows the closure to mutate the captured variables,
// meaning it can change the state of the environment.
// However, because of the potential mutation, access to the closure might be restricted to ensure safety, especially in concurrent scenarios.
pub fn closure_once<F, A, R>(fn_once: F) -> Closure<F::FnMut>
where
    F: 'static + WasmClosureFnOnce<A, R>,
{
    Closure::once(fn_once)
}


pub type LoopClosure = Closure<dyn FnMut(f64)>;
pub fn request_animation_frame(callback: &LoopClosure) -> Result<i32, anyhow::Error> {
    window()?
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|err| anyhow!("Cannot request animation frame {:#?}", err))
}

// f: impl FnMut() means it can be called multiple times and may mutate its captured variables.
// Use 'static to allow the closure to safely outlive the call to create_raf_closure
pub fn create_raf_closure(f: impl FnMut(f64) + 'static) -> LoopClosure {
    // By moving a closure to the heap (in other word, by using Box),
    // (1) ensure that it has a 'static lifetime, allowing it to live as long as needed, regardless of the original scope.
    // (2) allow for dynamic dispatch, where the exact type of the boxed value can vary at runtime as long as it implements the specified trait, in this case FnMut(f64)
    closure_wrap(Box::new(f))
}

// T must implement the WasmClosure trait. The + ?Sized part means that T can also be a dynamically sized type.
pub fn closure_wrap<T: WasmClosure + ?Sized>(data: Box<T>) -> Closure<T> {
    Closure::wrap(data)
}






