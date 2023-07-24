use simplesearch::common::server::WebServer;

const PORT: u16 = 8080;
const HOST: [u8;4] = [127, 0, 0, 1];
const POOL_SIZE: usize = 50;

fn main() {
    let server = WebServer::new(PORT, HOST, POOL_SIZE);
    eprintln!("created server: {:#?}", server);

    server.connect()

}
