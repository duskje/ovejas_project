#!/bin/bash
REMOTE_ADDRESS=$(tofu output -raw server_address)
ssh -o "StrictHostKeyChecking no" root@$REMOTE_ADDRESS

