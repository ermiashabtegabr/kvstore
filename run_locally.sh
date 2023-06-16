#!/bin/bash

session="kvstore-0"

# replica 1
window=1
tmux send-keys -t $session:$window 'cargo run -- --id 1 --peers 2 3' C-m

# replica 2
window=2
tmux send-keys -t $session:$window 'cargo run -- --id 2 --peers 1 3' C-m

# replica 3
window=3
tmux send-keys -t $session:$window 'cargo run -- --id 3 --peers 1 2' C-m
