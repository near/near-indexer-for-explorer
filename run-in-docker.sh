#!/usr/bin/env bash
set -euo pipefail   # Bash "strict mode"
script_dirpath="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"


# ==================================================================================================
#                                             Constants
# ==================================================================================================
# The path where the localnet NEAR config dir will be initialized
LOCALNET_NEAR_DIRPATH="/root/.near/localnet"
CONFIG_JSON_FILEPATH="${LOCALNET_NEAR_DIRPATH}/config.json"

# Config properties that will be set as part of startup
TRACKED_SHARD_CONFIG_PROPERTY="tracked_shards"
ARCHIVE_CONFIG_PROPERTY="archive"


# ==================================================================================================
#                                       Arg Parsing & Validation
# ==================================================================================================
show_helptext_and_exit() {
    echo "Usage: $(basename "${0}") diesel_binary_filepath database_url indexer_binary_filepath [extra_indexer_param]..."
    echo ""
    echo "  diesel_binary_filepath  The filepath to the Diesel binary that will be used to run the database migration"
    echo "  database_url            The URL of the database against which the Diesel migration should be run, and the "
    echo "                          indexer should connect to (e.g. \"postgres://near:near@contract-helper-db:5432/indexer\")"
    echo "  indexer_binary_filepath The filepath to the binary that will run the indexer node"
    echo "  extra_indexer_param...  An arbitrary number of extra parameters that will be passed as-is to the indexer"
    echo ""
    exit 1  # Exit with an error so that if this is accidentally called by CI, the script will fail
}

diesel_binary_filepath="${1:-}"
database_url="${2:-}"
indexer_binary_filepath="${3:-}"

if [ -z "${diesel_binary_filepath}" ]; then
    echo "Error: no Diesel binary filepath provided" >&2
    show_helptext_and_exit
fi
if ! [ -f "${diesel_binary_filepath}" ]; then
    echo "Error: provided Diesel binary filepath '${some_filepath_arg}' isn't a valid file" >&2
    show_helptext_and_exit
fi
if [ -z "${database_url}" ]; then
    echo "Error: no database URL provided" >&2
    show_helptext_and_exit
fi
if [ -z "${indexer_binary_filepath}" ]; then
    echo "Error: no indexer binary filepath provided" >&2
    show_helptext_and_exit
fi
if ! [ -f "${indexer_binary_filepath}" ]; then
    echo "Error: provided indexer binary filepath '${some_filepath_arg}' isn't a valid file" >&2
    show_helptext_and_exit
fi

shift 3   # Prep for consuming the extra indexer params below


# ==================================================================================================
#                                             Main Logic
# ==================================================================================================
# We add this check to see if the localnet directory already exists so that we can restart the 
# indexer-for-explorer container: if the directory doesn't exist, the container is starting for the 
# first time; if it already exists, the container is restarting so there's no need to do the migration 
# or genesis setup
if ! [ -d "${LOCALNET_NEAR_DIRPATH}" ]; then
    if ! DATABASE_URL="${database_url}" "${diesel_binary_filepath}" migration run; then
        echo "Error: The Diesel migration failed" >&2
        exit 1
    fi

    if ! DATABASE_URL="${database_url}" "${indexer_binary_filepath}" --home-dir "${LOCALNET_NEAR_DIRPATH}" init ${BOOT_NODES:+--boot-nodes=${BOOT_NODES}} --chain-id localnet; then
        echo "Error: An error occurred generating the genesis information" >&2
        exit 1
    fi

    # Required due to https://github.com/near/near-indexer-for-explorer#configure-near-indexer-for-explorer
    if ! num_tracked_shard_instances="$(grep -c "\"${TRACKED_SHARD_CONFIG_PROPERTY}\":" "${CONFIG_JSON_FILEPATH}" || true)"; then
        echo "Error: An error occurred getting the number of instances of the '${TRACKED_SHARD_CONFIG_PROPERTY}' config property to verify there's only one" >&2
        exit 1
    fi
    if [ "${num_tracked_shard_instances}" -ne 1 ]; then
        echo "Error: Expected exactly one line to match property '${TRACKED_SHARD_CONFIG_PROPERTY}' in config file '${CONFIG_JSON_FILEPATH}' but got ${num_tracked_shard_instances}" >&2
        exit 1
    fi
    if ! sed -i 's/"'${TRACKED_SHARD_CONFIG_PROPERTY}'": \[\]/"'${TRACKED_SHARD_CONFIG_PROPERTY}'": \[0\]/' "${CONFIG_JSON_FILEPATH}"; then
        echo "Error: An error occurred setting the tracked shards in the config" >&2
        exit 1
    fi

    # Required to keep more than 5 blocks in memory
    if ! num_archive_instances="$(grep -c "\"${ARCHIVE_CONFIG_PROPERTY}\":" "${CONFIG_JSON_FILEPATH}" || true)"; then
        echo "Error: An error occurred getting the number of instances of the '${ARCHIVE_CONFIG_PROPERTY}' config property to verify there's only one" >&2
        exit 1
    fi
    if [ "${num_archive_instances}" -ne 1 ]; then
        echo "Error: Expected exactly one line to match property '${ARCHIVE_CONFIG_PROPERTY}' in config file '${CONFIG_JSON_FILEPATH}' but got ${num_archive_instances}" >&2
        exit 1
    fi
    if ! sed -i 's/"'${ARCHIVE_CONFIG_PROPERTY}'": false/"'${ARCHIVE_CONFIG_PROPERTY}'": true/' "${CONFIG_JSON_FILEPATH}"; then
        echo "Error: An error occurred setting the archive mode to true" >&2
        exit 1
    fi
fi

# NOTE1: If the --store-genesis flag isn't set, the accounts in genesis won't get created in the DB which will lead to foreign key constraint violations
#  See https://github.com/near/near-indexer-for-explorer/issues/167
# NOTE2: The funky ${1+"${@}"} incantation is how you you feed arguments exactly as-is to a child script in Bash
#  ${*} loses quoting and ${@} trips set -e if no arguments are passed, so this incantation says, "if and only if
#  ${1} exists, evaluate ${@}"
DATABASE_URL="${database_url}" "${indexer_binary_filepath}" --home-dir "${LOCALNET_NEAR_DIRPATH}" run --store-genesis sync-from-latest ${1+"${@}"}
