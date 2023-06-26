use std::{thread, time::Duration};
use trust_dns_proto::op::response_code::ResponseCode;
use trust_dns_resolver::{error::ResolveErrorKind, TokioAsyncResolver};

static NXDOMAIN_RETRY_WAIT: u64 = 5000;

pub async fn srv_lookup(_domain: String) {
    let port_name = "grpc";
    let protocol = "tcp";
    let service_name = "kvstore-hs";
    let namespace = "kvstore-k8s";

    let dns_name = format!("_{port_name}._{protocol}.{service_name}.{namespace}",);

    let resolver =
        TokioAsyncResolver::tokio_from_system_conf().expect("Failed to create async resolver");

    let mut name_lookup_attempts = 10;

    loop {
        match resolver.srv_lookup(dns_name.clone()).await {
            Ok(response) => {
                response
                    .iter()
                    .for_each(|srv| println!("Found service {srv:?}"));
                break;
            }
            Err(err) => {
                if let ResolveErrorKind::NoRecordsFound { response_code, .. } = err.kind() {
                    if response_code == &ResponseCode::NXDomain {
                        if name_lookup_attempts == 0 {
                            break;
                        }

                        println!("Failed to lookup srv for peer discovery {name_lookup_attempts} attempts left");
                        name_lookup_attempts -= 1;
                        thread::sleep(Duration::from_millis(NXDOMAIN_RETRY_WAIT));
                    }
                } else {
                    println!("Failed to lookup srv: {err:?}");
                    break;
                }
            }
        }
    }
}
