This is a simple and experimental strongly consistent key-value store using [OmniPaxos](https://github.com/haraldng/omnipaxos) that can be deployed in kubernetes. 

### Get started using minikube
Make sure that you have [kubernetes](https://kubernetes.io/docs/setup/), [kubectl](https://kubernetes.io/docs/tasks/tools/), and [minikube](https://minikube.sigs.k8s.io/docs/start/) installed.

Start minikube with sufficient resources 
```bash
$ minikube start --memory 2048 --cpus=2
```

Clone the repo
```bash
$ git clone git@github.com:ermiashabtegabr/kvstore.git
```

Inside the `kvstore` run:
```bash
$ kubectl create -f kvstore-k8s-deployed.yaml
```

This should output the following: 
```bash
service/kvstore-hs created
service/kvstore-cs created
statefulset.apps/kvstore created
```

This created a cluster of three pods that run the `kvstore` and send messages to eachother. 

To insert key-values you have open a proxy that sends the requests to the `kvstore-cs` service that serves as a loadbalancer and sends your requests to one of the pods. To do so, open a separate terminal and run:
```bash
$ kubectl port-forward services/kvstore-cs -n kvstore-k8s 8080:8080
```

Go back to the earlier terminal and run:
```bash
$ curl -X POST http://127.0.0.1:8080/set -H 'Content-Type: application/json' -d '{"key": "eight", "value": 8}'
$ curl -X POST http://127.0.0.1:8080/set -H 'Content-Type: application/json' -d '{"key": "nine", "value": 9}'
$ curl -X POST http://127.0.0.1:8080/set -H 'Content-Type: application/json' -d '{"key": "ten", "value": 10}'
```

This will insert the keys `eight`, `nine`, and `ten` with their associated types `8`, `9`, and `10`. 

To get the inserted values run:
```bash
$ curl http://127.0.0.1:8080/get\?key\=eight # -> [eight] -> [Some(8)]
$ curl http://127.0.0.1:8080/get\?key\=nine # -> [nine] -> [Some(9)]
$ curl http://127.0.0.1:8080/get\?key\=ten # -> [ten] -> [Some(10)]
```

To delete the cluster run:
```bash
$ kubectl delete -f kvstore-k8s-deployed.yaml
```

You also have to delete the `persistentVolumes`:
```bash
$ kubectl delete pvc --all -n kvstore-k8s 
```

Finally to stop minikube run:
```bash
$ minikube stop
```
