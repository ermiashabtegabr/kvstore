#!/bin/bash

echo "**** starting kvstore node ****"

export CLUSTER_DIR_ROOT=./cluster-data

if [[ -z "${NODE_ID}" ]]; then
    echo 'NODE_ID environment variable is not specified'
    exit 1
fi

./kvstore --node $NODE_ID --replicas 3
