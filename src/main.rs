mod webservice;
mod hourglass_state;

#[actix_web::main]
async fn main() {
    let webservice = webservice::start_webservice();
    loop {
        let received = webservice.hourglass_state_rx.recv().unwrap();
        if !received.finalize {
            println!("Received {} {}", received.ticking, received.remaining_seconds);
        } else {
            println!("Ending service...");
            webservice.server_control.stop(true).await;
            println!("Good bye!");
            break;
        }
    }
}