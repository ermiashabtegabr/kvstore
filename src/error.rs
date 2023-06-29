use tonic::transport::Error as TonicErr;
use trust_dns_resolver::error::ResolveError;

#[derive(Debug)]
pub enum Error {
    Connection(TonicErr),
    SrvLookup(ResolveError),
}
