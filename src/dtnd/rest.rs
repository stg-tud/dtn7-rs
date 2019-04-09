use super::daemon::*;
use crate::core::application_agent::ApplicationAgentData;
use crate::core::helpers::rnd_peer;
use crate::DTNCORE;
use bp7::dtntime::CreationTimestamp;
use bp7::helpers::rnd_bundle;
use futures::future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use log::{debug, error, info, trace, warn};
use tokio::prelude::*;

// Just a simple type alias
type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn rest_handler(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());

    info!("{} {}", req.method(), req.uri().path());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("dtn7 ctrl interface");
        }
        (&Method::GET, "/status/eids") => {
            *response.body_mut() =
                Body::from(serde_json::to_string_pretty(&DTNCORE.lock().unwrap().eids()).unwrap());
        }
        (&Method::GET, "/status/bundles") => {
            *response.body_mut() = Body::from(
                serde_json::to_string_pretty(&DTNCORE.lock().unwrap().bundles()).unwrap(),
            );
        }
        (&Method::GET, "/status/peers") => {
            *response.body_mut() =
                Body::from(serde_json::to_string_pretty(&DTNCORE.lock().unwrap().peers).unwrap());
        }
        (&Method::GET, "/status/info") => {
            *response.body_mut() =
                Body::from(serde_json::to_string_pretty(&DTNCORE.lock().unwrap().stats).unwrap());
        }
        (&Method::GET, "/debug/rnd_bundle") => {
            println!("generating debug bundle");
            let b = rnd_bundle(CreationTimestamp::now());
            *response.body_mut() = Body::from(b.id());
            DTNCORE.lock().unwrap().push(b);
        }
        (&Method::GET, "/debug/rnd_peer") => {
            println!("generating debug peer");
            let p = rnd_peer();
            *response.body_mut() = Body::from(serde_json::to_string_pretty(&p).unwrap());
            DTNCORE.lock().unwrap().peers.insert(p.addr, p);
        }
        (&Method::POST, "/echo") => {
            // we'll be back
        }
        (&Method::GET, "/register") => {
            // TODO: support non-node-specific EIDs
            // we'll be back
            if let Some(params) = req.uri().query() {
                if params.chars().all(char::is_alphanumeric) {
                    dbg!(params);
                    let eid = format!("dtn://{}/{}", DTNCORE.lock().unwrap().nodeid, params);
                    DTNCORE.lock().unwrap().register_application_agent(
                        ApplicationAgentData::new_with(eid.clone().into()),
                    );
                    *response.body_mut() = Body::from(format!("Registered {}", eid));
                }
            }
        }
        (&Method::GET, "/unregister") => {
            // TODO: support non-node-specific EIDs
            // we'll be back
            if let Some(params) = req.uri().query() {
                if params.chars().all(char::is_alphanumeric) {
                    dbg!(params);
                    let eid = format!("dtn://{}/{}", DTNCORE.lock().unwrap().nodeid, params);
                    DTNCORE.lock().unwrap().unregister_application_agent(
                        ApplicationAgentData::new_with(eid.clone().into()),
                    );
                    *response.body_mut() = Body::from(format!("Unregistered {}", eid));
                }
            }
        }
        (&Method::GET, "/endpoint") => {
            // we'll be back
            if let Some(params) = req.uri().query() {
                if params.chars().all(char::is_alphanumeric) {
                    dbg!(params);
                    let eid = format!("dtn://{}/{}", DTNCORE.lock().unwrap().nodeid, params); // TODO: support non-node-specific EIDs
                    if let Some(aa) = DTNCORE.lock().unwrap().get_endpoint_mut(&eid.into()) {
                        if let Some(mut bundle) = aa.pop() {
                            *response.body_mut() = Body::from(bundle.to_json());
                        } else {
                            *response.body_mut() = Body::from("[]");
                        }
                    } else {
                        *response.status_mut() = StatusCode::NOT_FOUND;
                        *response.body_mut() = Body::from("No such endpoint registered!");
                    }
                }
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Box::new(future::ok(response))
}

pub fn spawn_rest() {
    //let rs = RestService { tx };

    // Construct our SocketAddr to listen on...
    let addr = ([127, 0, 0, 1], 3000).into();

    let fut = move || service_fn(rest_handler);
    // Then bind and serve...
    let server = Server::bind(&addr).serve(fut);

    tokio::spawn(server.map_err(|e| {
        error!("{}", e);
    }));
}