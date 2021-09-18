use crate::hourglass_state::HourglassState;

use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder, rt::System};
use actix_web::dev::Server;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::sync::Mutex;
use std::thread;
use actix_files::Files;

pub struct WebServiceIO {
    pub server_control: actix_web::dev::Server,
    pub hourglass_state_rx: Receiver<HourglassState<bool, u16, bool>>
}

pub struct WebServiceData {
    hourglass_state: HourglassState<Mutex<bool>, Mutex<u16>, Mutex<bool>>,
    tx: Sender<HourglassState<bool, u16, bool>>
}

impl HourglassState<Mutex<bool>, Mutex<u16>, Mutex<bool>>{
    fn to_parcel(&self) -> HourglassState<bool, u16, bool> {
        HourglassState::new(
            *self.ticking.lock().unwrap(),
            *self.remaining_seconds.lock().unwrap(),
            *self.finalize.lock().unwrap()
        )
    }
}

pub fn start_webservice() -> WebServiceIO {
    let (web_service_tx, web_service_rx) = unbounded::<HourglassState<bool, u16, bool>>();
    let (control_extraction_tx, control_extraction_rx) = unbounded::<Server>();

    thread::spawn(move || {
        let sys = System::new("http-server");

        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(
                    WebServiceData {
                        hourglass_state: HourglassState::new(
                            Mutex::new(false),
                            Mutex::new(1200u16),
                            Mutex::new(false)
                        ),
                        tx: web_service_tx.clone()
                    }
                ))
                .route("/", web::get().to(index))
                .route("/start", web::get().to(start))
                .route("/stop", web::get().to(stop))
                .route("/minus_minute", web::get().to(minus_minute))
                .route("/plus_minute", web::get().to(plus_minute))
                .route("/get_ticking", web::get().to(get_ticking))
                .route("/get_remaining_seconds", web::get().to(get_remaining_seconds))
                .route("/set_remaining_seconds/{seconds}", web::get().to(set_remaining_seconds))
                .route("/end_service", web::get().to(end_service))
                .service(Files::new("/", "html/"))
        })
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run();

        control_extraction_tx.send(server).unwrap();
        sys.run()
    });

    let server = control_extraction_rx.recv().unwrap();

    WebServiceIO {
        hourglass_state_rx: web_service_rx,
        server_control: server
    }
}

async fn index(_data: web::Data<WebServiceData>) -> HttpResponse {
    HttpResponse::Found()
        .header("LOCATION", "/index.html")
        .finish()
}

async fn start(data: web::Data<WebServiceData>) -> impl Responder {
    *data.hourglass_state.ticking.lock().unwrap() = true;
    data.tx.send(data.hourglass_state.to_parcel()).unwrap();
    format!("Started.")
}

async fn stop(data: web::Data<WebServiceData>) -> impl Responder {
    *data.hourglass_state.ticking.lock().unwrap() = false;
    data.tx.send(data.hourglass_state.to_parcel()).unwrap();
    format!("Stopped.")
}

async fn plus_minute(data: web::Data<WebServiceData>) -> impl Responder {
    *data.hourglass_state.remaining_seconds.lock().unwrap() += 60;
    data.tx.send(data.hourglass_state.to_parcel()).unwrap();
    format!("Minute added.")
}

async fn minus_minute(data: web::Data<WebServiceData>) -> impl Responder {
    *data.hourglass_state.remaining_seconds.lock().unwrap() -= 60;
    data.tx.send(data.hourglass_state.to_parcel()).unwrap();
    format!("Minute subtracted.")
}

async fn get_ticking(data: web::Data<WebServiceData>) -> impl Responder {
    let ticking = *data.hourglass_state.ticking.lock().unwrap();
    format!("{}", if ticking { "true" } else { "false"})
}

async fn get_remaining_seconds(data: web::Data<WebServiceData>) -> impl Responder {
    let remaining_seconds = *data.hourglass_state.remaining_seconds.lock().unwrap();
    format!("{}", remaining_seconds)
}

async fn set_remaining_seconds(req: HttpRequest, data: web::Data<WebServiceData>) -> impl Responder {
    format!("called set rem");
    let seconds = req.match_info().get("seconds");
    if seconds.is_some() {
        match seconds.unwrap().parse::<u16>() {
            Ok(value) => {
                *data.hourglass_state.remaining_seconds.lock().unwrap() = value;
                data.tx.send(data.hourglass_state.to_parcel()).unwrap();
                format!("Set remaining seconds to {}", value)
            },
            Err(_) => {
                format!("Remaining seconds not changed, wrong argument. Try a positive number.")
            }
        }
    } else {
        format!("Remaining seconds not changed, missing argument.")
    }
}

async fn end_service(data: web::Data<WebServiceData> )-> impl Responder {
    *data.hourglass_state.finalize.lock().unwrap() = true;
    data.tx.send(data.hourglass_state.to_parcel()).unwrap();
    format!("Good bye!")
}
