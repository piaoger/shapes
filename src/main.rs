#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate router;
extern crate serde;

use iron::prelude::*;
use iron::status;
use router::Router;
use serde::json;
use std::io::Read;
use std::sync::{Arc, Mutex};
//use std::fs::File;
use std::path::Path;

// static files
extern crate mount;
extern crate staticfile;

use mount::Mount;
use staticfile::Static;


// Websocket
extern crate websocket;

use std::thread;
 use websocket::{Server, Message, Sender, Receiver};

use websocket::header::WebSocketProtocol;


#[derive(Serialize, Deserialize)]
struct Greeting {
    msg: String
}


fn setup_httpserv() {
    let greeting = Arc::new(Mutex::new(Greeting { msg: "Hello, World".to_string() }));
    let greeting_clone = greeting.clone();
    
    let mut router = Router::new();

    router.get("/", move |r: &mut Request| hello_world(r, &greeting.lock().unwrap()));
    router.post("/set", move |r: &mut Request| set_greeting(r, &mut greeting_clone.lock().unwrap()));
    
    fn hello_world(_: &mut Request, greeting: &Greeting) -> IronResult<Response> {
        let payload = json::to_string(greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    // Receive a message by POST and play it back.
    fn set_greeting(request: &mut Request, greeting: &mut Greeting) -> IronResult<Response> {
        let mut payload = String::new();
        request.body.read_to_string(&mut payload).unwrap();
        println!("{}", payload);
        *greeting = json::from_str(&payload).unwrap();
        Ok(Response::with(status::Ok))
    }


    let mut mount = Mount::new();
    mount.mount("/api", router);
    mount.mount("/", Static::new(Path::new("www")));

        // Listen
    Iron::new(mount).http("127.0.0.1:8080").unwrap();

    //


}

fn setup_wsserv() {

    let server = Server::bind("127.0.0.1:8021").unwrap();

    for connection in server {
        // Spawn a new thread for each connection.
        thread::spawn(move || {

            let request = connection.unwrap().read_request().unwrap(); // Get the request
            request.validate().unwrap();

            // Checking websocket protocol, here is "rust-websocket"
            //   html: var socket = new WebSocket("ws://localhost:8080", "rust-websocket");
            // If no websocket protocol check here, there will have error in Chromimum browsers: chrome, opera....
            // Let's also check the protocol - if it's not what we want, then fail the connection
            // if request.protocol().is_none() || !request.protocol().unwrap().contains(&"rust-websocket".to_string()) {
            //     let response = request.fail();
            //     let _ = response.send_into_inner();
            //     return;
            // }

            // let mut response = request.accept(); // Generate a response
            // response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()])); // Send a Sec-WebSocket-Protocol header
            // let mut client = response.send().unwrap(); // Send the response

            let response = request.accept(); // Form a response
            let mut client = response.send().unwrap(); // Send the response
            
            let ip = client.get_mut_sender()
                .get_mut()
                .peer_addr()
                .unwrap();
            
            println!("Connection from {}", ip);
            
            let message = Message::Text("Hello".to_string());
            client.send_message(message).unwrap();
            
            let (mut sender, mut receiver) = client.split();
            
            for message in receiver.incoming_messages() {
                let message = message.unwrap();
                
                match message {
                    Message::Close(_) => {
                        let message = Message::Close(None);
                        sender.send_message(message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    Message::Ping(data) => {
                        let message = Message::Pong(data);
                        sender.send_message(message).unwrap();
                    }
                    _ => sender.send_message(message).unwrap(),
                }
            }
        });
    }
}


 

use std::net::Ipv4Addr;

#[derive(Debug)]
enum HostAddress {
    DomainName(String),
    Ip(Ipv4Addr),
}

#[derive(Debug)]
struct Port(u32);

#[derive(Debug)]
struct ConnectionLimit(u32);

 

fn main() {
    // Start listening for http connections
    thread::spawn(move || {
        setup_httpserv() ;
    });
    
    setup_wsserv();
    //mainanymap();
}
