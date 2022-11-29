# Changelog

## 0.11.0

* `indexer-explorer-lake` now uses the default AWS credentials provider. Credentials can no longer be set via command line arguments and environment variables need to be updated as follows:
```diff
- "LAKE_AWS_ACCESS_KEY=AKIAIOSFODNN7EXAMPLE"
- "LAKE_AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
+ "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE"
+ "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

## 0.10.3

* Extract database logic to library crate

## 0.10.2

* Avoid recreating access key on transfer to implicit account

## 0.10.1

* Upgrade `near-lake-framework` to `0.5.2`

## 0.10.0

* Upgrade `near-lake-framework` to `0.5.1`
