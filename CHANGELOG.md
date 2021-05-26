# Changelog

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
