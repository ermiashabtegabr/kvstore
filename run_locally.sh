#!/bin/bash

session="kvstore-0"

# replica 1
window=1
tmux send-keys -t $session:$window './target/debug/kvstore --node kvstore-1 --replicas 3' C-m

# replica 2
window=2
tmux send-keys -t $session:$window './target/debug/kvstore --node kvstore-2 --replicas 3' C-m

# replica 3
window=3
tmux send-keys -t $session:$window './target/debug/kvstore --node kvstore-3 --replicas 3' C-m
