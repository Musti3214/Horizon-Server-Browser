pub fn addresses(host: &str, port: u16) -> crate::Result<Vec<std::net::SocketAddr>> {
    crate::resolver::resolve(host, port).map(super::ipv4_first)
}
