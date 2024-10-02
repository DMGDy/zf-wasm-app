use yew::prelude::*;
use gloo::net::http::Request;
use web_sys::wasm_bindgen::JsCast;
use serde::{Serialize,Deserialize};
use wasm_bindgen_futures::spawn_local;
use web_sys;
use std::fmt;

const ADDRESS: &str = "http://172.20.10.7:8080";

#[derive(Default,Serialize,Clone,PartialEq)]
struct TestData {
    device: String,
    check: bool,
    int_data: i32,
    float_data:f64,
}
 
#[derive(Deserialize,Clone)]
enum ServerResponse {
    Awake,
    InProgress,
    Done,
    Error,
    Idle,
}



#[derive(Properties,PartialEq)]
struct DataProp {
    data: TestData,
}

#[derive(Clone, PartialEq)]
enum Device {
    BST,
    CWS,
    PrS,
    ESCM,
}

impl ServerResponse {
    fn from_i32(n: i32) -> ServerResponse {
        match n {
            0 => Self::Idle,
            1 => Self::Awake,
            2 => Self::InProgress,
            3 => Self::Done,
            4 => Self:: Error,
            _ => Self:: Error,
        }
    }
    fn code(&self) -> i32 {
        match self {
            Self::Idle => 0,
            Self::Awake => 1,
            Self::InProgress => 2,
            Self::Done => 3,
            Self::Error => -1,
        }
    }

    fn message(&self) -> &str{
        match self {
            Self::Idle => "Awaiting input",
            Self::Error => "Error has occurred either on the server or microcontroller!",
            Self::Awake => "Server is up and has loaded microcontroller firmware",
            Self::InProgress => "Microcontroller has began testing",
            Self::Done => "Test results are available",
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Device::BST => write!(f, "Brake Signal Transmitter"),
            Device::CWS => write!(f, "Continuous Wear Sensor"),
            Device::PrS => write!(f, "Pressure Sensor"),
            Device::ESCM=> write!(f, "Electronic Stability Control Module"),
        }
    }
}

#[derive(Properties, PartialEq)]
struct DeviceRef {
    device: Device,
    on_click: Callback<Device>,
}

impl DeviceRef {
    fn new(device: Device, on_click: Callback<Device>) -> Self {
        Self {
            device,
            on_click,
        }
    }
}



#[function_component(SendData)]
fn send_data(props: &DataProp) -> Html {
    let status = use_state(|| ServerResponse::Idle);
    let onclick = {
        let data = props.data.clone();
        let status = status.clone();
        Callback::from(move |_ :MouseEvent| {
            let data = data.clone();
            let status = status.clone();
            spawn_local(async move {
                let response = Request::post(ADDRESS)
                    .json(&data)
                    .unwrap()
                    .send()
                    .await;
                match response.unwrap().json::<i32>().await{
                    Ok(code) => {
                        status.set(ServerResponse::from_i32(code));
                    }
                    Err(_) => {
                        status.set(ServerResponse::Error)
                    }
                }
            })
        })
    };
    html! {
        <div>
            <button {onclick}style="cursor: pointer;">
                {"Start Test"}             
            </button>
            <p>
                <strong> {"Code: "} </strong>
                {(*status).clone().code()}
                <br/>
                <strong> {"Status:"}</strong>
                {(*status).clone().message()}
            </p>
        </div>
    }
}


#[function_component(ClickDev)]
fn child(dev_prop: &DeviceRef) -> Html {
    let onclick = {
        let device = dev_prop.device.clone();

        let on_click = dev_prop.on_click.clone();
        Callback::from(move |_| {
            on_click.emit(device.clone());
        })
    };
    html! {
        <p onclick={onclick} style="cursor: pointer;">
            {dev_prop.device.to_string()}
        </p>
    }
}

#[function_component(App)]
fn app() -> Html {
    let dropdown_visible = use_state(|| false);
    let chosen_dev = use_state(|| "None".to_string());
    let use_string_pot = use_state(|| false);

    let toggle_dropdown = {
        let dropdown_visible = dropdown_visible.clone();
        Callback::from(move |_| {
            dropdown_visible.set(!*dropdown_visible);
        })
    };

    let testdata = use_state(TestData::default);

    let on_device_select = {
        let chosen_dev = chosen_dev.clone();
        let dropdown_visible = dropdown_visible.clone();
        let data = testdata.clone();
        Callback::from(move |device: Device| {
            chosen_dev.set(device.to_string());
            dropdown_visible.set(false);
            data.set(TestData {
                device: device.to_string(),
                ..(*data)
            });
        })
    };

    let on_string_pot_change = {
        let use_string_pot = use_string_pot.clone();
        let testdata = testdata.clone();
        Callback::from(move |e: web_sys::Event| {
            let checked = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().checked();
            use_string_pot.set(checked);
            testdata.set(TestData {
                check: checked,
                ..(*testdata).clone()  // Clone the contents of testdata
            });
        })
    };

    let devices = vec![
        Device::BST,
        Device::CWS,
        Device::PrS,
        Device::ESCM,
    ];

    html! {
        <>
            <h1>{"ZF Test Device"}</h1>
            <button onclick={toggle_dropdown}>
                {if *dropdown_visible { "Hide Devices" } else { "Show Devices" }}
            </button>
            <p><strong>{"Chosen Device: "}</strong>{(*chosen_dev).clone()}</p>
            if *dropdown_visible {
                <div>
                    {devices.into_iter().map(|device| {
                        html! {
                            <ClickDev ..DeviceRef::new(device, on_device_select.clone()) />
                        }
                    }).collect::<Html>()}
                 </div>
            }
            if (*chosen_dev) == Device::BST.to_string() {
                <div>
                    <label>
                        <input 
                            type="checkbox" 
                            checked  = {*use_string_pot} 
                            onchange = {on_string_pot_change.clone()} 
                        />
                        {" Use String Potentiometer"}
                    </label>
                </div>
            }
           <SendData data={(*testdata).clone()} />
       </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
