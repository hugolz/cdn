use gloo::console;
use js_sys::Date;
use wasm_bindgen::prelude::wasm_bindgen;
use yew::{html, Component, Context, Html};

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

/* NOTE :
    When making a request, if you do not specifiy http:// at the start of the url
    The url will be interpreted as an endpoint of the server 
*/

// Define the possible messages which can be sent to the component
pub enum Msg {
    Increment,
    Decrement,
    FetchDashboard,
    SetDashboardFetchState(FetchState<DashboardData>),
}
#[derive(Debug)]
pub struct DashboardData {
    count: i32,
}

pub struct App {
    value: i64, // This will store the counter value
    dashboard_data: FetchState<DashboardData>,
}

/// The possible states a fetch request can be in.
pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(FetchError),
}

async fn fetch_dashboard(url: &'static str) -> Result<DashboardData, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    console::log!(opts.clone());

    let request = Request::new_with_str_and_init(url, &opts)?;

    console::log!(request.url());
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    let text = JsFuture::from(resp.text()?).await?;
    console::log!(text.as_string().unwrap());

    Ok(
        DashboardData { count: text.as_string().unwrap().parse().unwrap() }
    )
}

/// Something wrong has occurred while fetching an external resource.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    err: JsValue,
}
impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            value: 0,
            dashboard_data: FetchState::NotFetching,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Increment => {
                self.value += 1;
                console::log!("plus one"); // Will output a string to the browser console
                true // Return true to cause the displayed change to update
            }
            Msg::Decrement => {
                self.value -= 1;
                console::log!("minus one");
                true
            }
            Msg::SetDashboardFetchState(fetch_state) => {
                self.dashboard_data = fetch_state;
                true
            }
            Msg::FetchDashboard => {
                ctx.link().send_future(async {
                    match fetch_dashboard("/dashboard/cache_count").await {
                        Ok(db) => Msg::SetDashboardFetchState(FetchState::Success(db)),
                        Err(err) => Msg::SetDashboardFetchState(FetchState::Failed(err)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetDashboardFetchState(FetchState::Fetching));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.dashboard_data{
            FetchState::NotFetching => html!{
                <div> 
                    <p>{"NotFetching"}</p>
                    <button onclick={ctx.link().callback(|_| Msg::FetchDashboard)}>
                        { "Get Dashboard" }
                    </button>
                </div>
            },
            FetchState::Fetching => html!{<p>{"Fetching"}</p> },
            FetchState::Success(ref data) => html!{
                <div>
                    <p>{"Success"}</p>
                    <p>{format!("{data:?}")}</p>
                </div>
            },
            FetchState::Failed(_) =>html!{<p>{"Failed"}</p> },
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
