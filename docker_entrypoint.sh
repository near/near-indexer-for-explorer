#!/bin/bash

echo "Working directory is `pwd`"
sleep 5

if [ "$environment" = "development" ]; then

    echo "Running localnet..."
    ./diesel migration run && \
    ./indexer-explorer --home-dir /indexer/near/localnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id localnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /indexer/near/localnet/config.json && \
    sed -i 's/"archive": false/"archive": true/' /indexer/near/localnet/config.json && \
    ./indexer-explorer --home-dir /indexer/near/localnet run --store-genesis --stream-while-syncing --non-strict-mode --concurrency 100 sync-from-latest

elif [ "$environment" = "staging" ]; then

    echo "Running testnet..."
    ./diesel migration run && \
    ./indexer-explorer --home-dir /indexer/near/testnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id testnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /indexer/near/testnet/config.json && \
    sed -i 's/"archive": false/"archive": true/' /indexer/near/testnet/config.json && \
    ./indexer-explorer --home-dir /indexer/near/testnet run --store-genesis --stream-while-syncing --non-strict-mode --concurrency 100 sync-from-latest

elif [ "$environment" = "production" ]; then

    echo "Running mainnnet..."
    ./diesel migration run && \
    ./indexer-explorer --home-dir /indexer/near/mainnet init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id mainnet && \
    sed -i 's/"tracked_shards": \[\]/"tracked_shards": \[0\]/' /indexer/near/mainnet/config.json && \
    sed -i 's/"archive": false/"archive": true/' /indexer/near/mainnet/config.json && \
    ./indexer-explorer --home-dir /indexer/near/mainnet run --store-genesis --stream-while-syncing --non-strict-mode --concurrency 100 sync-from-latest

else
    exit 1
fi
