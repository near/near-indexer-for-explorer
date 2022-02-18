#!/bin/bash

echo "Working directory is `pwd`"
sleep 5

if [ "$env" = "development" ]; then

    echo "Running localnet..."
    ./diesel migration run && \
    ./indexer-explorer --home-dir /root/.near/localnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id localnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /root/.near/localnet/config.json && \
    sed -i 's/"archive": false/"archive": true/' /root/.near/localnet/config.json && \
    ./indexer-explorer --home-dir /root/.near/localnet run --store-genesis --stream-while-syncing --non-strict-mode --concurrency 100 sync-from-latest

elif [ "$env" = "staging" ]; then

    echo "Running testnet..."
    ./diesel migration run && \
    ./indexer-explorer --home-dir /root/.near/testnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id testnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /root/.near/testnet/config.json && \
    sed -i 's/"archive": false/"archive": true/' /root/.near/testnet/config.json && \
    ./indexer-explorer --home-dir /root/.near/testnet run --store-genesis --stream-while-syncing --non-strict-mode --concurrency 100 sync-from-latest

elif [ "$env" = "production" ]; then

    echo "Running mainnnet..."
    ./diesel migration run && \
    ./indexer-explorer --home-dir /root/.near/mainnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id mainnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /root/.near/mainnet/config.json && \
    sed -i 's/"archive": false/"archive": true/' /root/.near/mainnet/config.json && \
    ./indexer-explorer --home-dir /root/.near/mainnet run --store-genesis --stream-while-syncing --non-strict-mode --concurrency 100 sync-from-latest

else
    exit 1
fi