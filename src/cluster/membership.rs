use trust_dns_resolver::Resolver;

async fn get_pod_labels(domain: String) {
    let mut resolver = Resolver::default().unwrap();

    let dns_name = "kvstore-hs.kvstore-k8s.svc.cluster.local";

    let response = resolver.lookup_ip(dns_name).unwrap();

    for ip in response.iter() {
        println!("Resolved IP: {}", ip);
    }
}
