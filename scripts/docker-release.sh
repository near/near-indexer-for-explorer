#!/bin/bash
set -eo pipefail

branch=${BUILDKITE_BRANCH//:/_}
branch=${branch//\//_}
commit=${BUILDKITE_COMMIT}

if [[ ${commit} == "HEAD" ]]; then
    commit=$(git rev-parse HEAD)
fi

image_name="near-indexer-for-explorer"

docker build -t $image_name .

docker tag $image_name nearprotocol/${image_name}:${branch}-${commit}
docker tag $image_name nearprotocol/${image_name}:${branch}

set -x
docker push nearprotocol/${image_name}:${branch}-${commit}
docker push nearprotocol/${image_name}:${branch}
if [[ ${branch} == "master" ]];
then
	docker tag $image_name nearprotocol/${image_name}:latest
	docker push nearprotocol/${image_name}:latest
fi
