# Changelog

## 0.10.3

* Upgrade `nearcore` to 1.22.0
* Add [NFT events](https://nomicon.io/Standards/NonFungibleToken/Event.html) support: `assets__non_fungible_token_events` table stores the information about NFT `mint`, `transfer`, `burn` events

## 0.10.2

- Change the retry logic. Make indexer fail with error if is has retried for 5 min
- Upgrade `nearcore` to 1.22.0

## 0.10.1

* Upgrade `nearcore` to 1.21.1

## 0.10.0

* Drop `--allow-missing-relations-in-first-blocks` flag
* Introduce `--non-strict-mode` which does the same as `--allow-missing-relations-in-first-blocks` flag did but endlessly
* Add `--stop-after-number-of-blocks <count>` flag to plan Indexer for Explorer to stop once it indexed the provided `<count>` of blocks. May be useful for debug or maintenance purposes.

## Breaking changes

* The flag `--allow-missing-relations-in-first-blocks` is not available anymore in favor of `--non-strict-mode` flag

## 0.9.3

* Escape `args_json` on the fly to avoid null-byte issues
* Upgrade to NEAR Indexer Framework `0.10.0`
* Refactor the storing Accounts and AccessKeys from genesis to optimize memory usage
* Improve logging to better understand what Indexer for Explorer is doing on the start

## 0.9.2 (hotfix)

* Change `receiver_id` field type to `String` to be compatible with `nearcore` `AccessKeyPermissionView` struct (it caused problems during AccessKey serialization)

## 0.9.1

* Upgrade `nearcore` to 1.21.0 (rc1)

## 0.9.0 (nearcore dependency contains bug)

* Upgrade `nearcore` to 1.21.0

## Breaking changes

* `init` command has changed according to changes in `nearcore`:
  - `download` argument has been replaced with `download_config` and `download_genesis`
  - `boot_nodes` argument was added
  - `download_config_url` was added
* `AccountId` from `near-primitives` was replaced with separate crate `near-account-id` and it is no longer an alias for `String`
  - All the fields related to an account id have type `near_account_id::AccountId`

## 0.8.0

* Background calculation of circulating supply and storing it to DB
* Improvements on some tables (add indexes, simplify sorting etc.)

## 0.7.1

* Update nearcore version to 1.20.0-rc.2

## 0.7.0

* Handle null-bytes in `AddKey` actions
* Update nearcore version to 1.20.0

## Breaking change

`init_configs` function from nearcore has been extended with additional optional parameter `max_gas_burnt_view`. We've extended NEAR Indexer for Explorer `InitConfigArgs`

## 0.6.9

* Add `--concurrency` parameter to adjust the number of simultaneously running asynchronous adapters

## 0.6.8

* Update NEAR Indexer Framework version to 0.9.2 (with optimized delayed receipts tracking system)

## 0.6.7

* Remove duplicates from `account_changes` table by fixing unique index ([see issue #74](https://github.com/near/near-indexer-for-explorer/issues/74))

## 0.6.6 (hotfix)

* Upgrade `nearcore` to 1.19.1 (hotfix)

## 0.6.5

* Update NEAR Indexer Framework version to 0.9.1 (previous contained a bug with processing delayed local receipts)

## 0.6.4 (contains bug)

* Fix the overwriting of `created_by_receipt_id` for implicit accounts that may confuse users ([see issue #68 for ref](https://github.com/near/near-indexer-for-explorer/issues/68))

## 0.6.3

* Denormalize table `action_receipt_actions` in order to speed up some queries by avoid
  additional joins
* Extend `extract_action_type_and_value_from_action_view` function to try to parse base64 encoded args
  as a JSON object to put them decoded in the function call args `action_receipt_actions.args` additionally

## 0.6.2

* Upgrade `nearcore` dependency to exclude recent updates to runtime which caused a bug ([see for ref](https://github.com/near/nearcore/releases/tag/1.19.0-rc.2))

## 0.6.1

* Upgrade `nearcore` to support newer protocol version (45)

## 0.6.0

* Upgrade `nearcore` to get NEAR Indexer Framework 0.9.0
* Corresponding changes into adapters according to changes in `StreamerMessage` structure
* NEAR Indexer for Explorer now uses stable Rust (`rust-toolchain` has been updated accordingly) 

## 0.5.0

* Tweak `sync-from-interruption` mode to start syncing from N blocks earlier that actual interruption

## 0.4.0

* Update `nearcore` dependency
* Update underlying dependencies to correspond `nearcore`

**The way of starting `actix` runtime has changes**

## 0.3.0

* Migrate from `tokio-diesel` to `actix-diesel` (patched by @frol)

## 0.2.3

* Upgrade `nearcore` dependency
* Upgrade some external dependencies (`actix`, `tokio`)

## 0.2.2

* Fill `deleted_by_receipt_id` if `access_key` on owner account deletion

## 0.2.1

* Add `access_key` on transfer to implicit account action
* Upgrade `nearcore` dependency
