use std::net;
use actix_web::{
    App,
    HttpServer,
    web,
    dev::{Server, ServerHandle}
};
use parking_lot::Mutex;


pub fn run<F: Fn(&mut web::ServiceConfig) -> () + Send + Clone + 'static>(addrs: impl net::ToSocketAddrs, cfg: F) -> Result<Server, Box<dyn std::error::Error>> {
    let stop_handle = web::Data::new(StopHandle::default());

    let server = HttpServer::new({
        let stop_handle = stop_handle.clone();
        let cfg = cfg.clone();

        move || {
            App::new()
                .app_data(stop_handle.clone())
                .configure(cfg.clone())
        }
    })
    .bind(addrs)?
    .run();

    stop_handle.register(server.handle());

    Ok(server)
}

#[derive(Default)]
pub struct StopHandle {
    inner: Mutex<Option<ServerHandle>>,
}

impl StopHandle {
    pub fn register(&self, handle: ServerHandle) {
        *self.inner.lock() = Some(handle);
    }

    pub fn stop(&self, graceful: bool) {
        let _ = self.inner.lock().as_ref().unwrap().stop(graceful);
    }
}
