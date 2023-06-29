#![allow(dead_code)]
use std::{thread, time::Duration};
use trust_dns_resolver::{error::ResolveError, Name, TokioAsyncResolver};

const NXDOMAIN_RETRY_WAIT: usize = 5000;
const MAX_ATTEMPTS: usize = 5;

pub async fn find_peers(dns_name: String) -> Result<Vec<Name>, ResolveError> {
    let resolver =
        TokioAsyncResolver::tokio_from_system_conf().expect("Failed to create async resolver");

    let mut lookup_attempts = 0;

    loop {
        match resolver.srv_lookup(&dns_name).await {
            Ok(response) => {
                return Ok(response
                    .into_iter()
                    .map(|srv| srv.target().clone())
                    .collect::<Vec<_>>());
            }
            Err(err) => {
                if lookup_attempts == MAX_ATTEMPTS {
                    return Err(err);
                }

                lookup_attempts += 1;
                thread::sleep(Duration::from_millis(NXDOMAIN_RETRY_WAIT as u64));
            }
        }
    }
}
