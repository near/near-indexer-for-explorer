Thank you for considering contributing to the NEAR Indexer for Explorer!

We welcome all external contributions. This document outlines the process of contributing to NEAR Indexer for Explorer.
For contributing to other repositories, see `CONTRIBUTING.md` in the corresponding repository.
For non-technical contributions, such as e.g. content or events, see [this document](https://docs.nearprotocol.com/docs/contribution/contribution-overview).

# Pull Requests and Issues

All the contributions to NEAR Indexer for Explorer happen via Pull Requests. To create a Pull Request, fork NEAR Indexer for Explorer repository on GitHub, create a new branch, do the work there, and then send the PR via GitHub interface.

The PRs should always be against the `master` branch.

The exact process depends on the particular contribution you are making.

## Typos or small fixes

If you see an obvious typo, or an obvious bug that can be fixed with a small change, in the code or documentation, feel free to submit the pull request that fixes it without opening an issue.

### Submitting the PR

Once your change is ready, prepare the PR. The PR can contain any number of commits, but when it is merged, they will all get squashed. The commit names and descriptions can be arbitrary, but the name and the description of the PR must follow the following template:

```
<type>: <name>

<description>

Test plan
---------
<test plan>
```

Where `type` is `fix` for fixes, `feat` for features, `refactor` for changes that primarily reorganize code, `doc` for changes that primarily change documentation or comments, and `test` for changes that primarily introduce new tests. The type is case sensitive.

The `test plan` should describe in detail what tests are presented, and what cases they cover.

### After the PR is submitted

1. We have a CI process configured to run all the sanity tests on each PR. If the CI fails on your PR, you need to fix it before it will be reviewed.
2. Once the CI passes, you should expect the first feedback to appear within 48 hours. The reviewers will first review your tests, and make sure that they can convince themselves the test coverage is adequate before they even look into the change, so make sure you tested all the corner cases.
3. Once you address all the comments, and your PR is accepted, we will take care of merging it.

## Proposing new ideas and features

If you want to propose an idea or a feature and work on it, create a new issue in the `near-indexer-for-Explorer` repository.

You should expect someone to comment on the issue within 48 hours after it is created.

## Code Style

We enforce formatting with rustfmt, so your code should be formatted with `cargo fmt` and checked with `cargo clippy` to pass CI. Additionally, we extend those default rules with some extras.

### Imports Ordering and Grouping

We use the following order to group our imports (use statements):

1. standard Rust library imports (e.g. `std::sync::Arc`)
2. external crates (e.g. `tokio::time`)
3. near-* crates (e.g. `near_indexer::near_primitives::types`)
4. local crate modules (e.g. `crate::db`)
5. local modules (e.g. `self::access_keys`)
6. `mod` statements

Separate each group with an empty line and maintain alphabetical order inside of each group (it is done automatically by rustfmt).

Here is an artificial example:

```rust
use std::path::PathBuf;
use std::sync::Arc;

use tokio::time;

use near_indexer::near_primitives;

pub use crate::db::AccessKey;

use self::db;

mod access_keys;
mod utils;
```

### Use Statements

#### Wildcards

Try to avoid wildcard imports on a module level as they are hard to reason about. Note, it is fine to use them in a limited scope, e.g. inside a function, macro, etc.

#### Module Imports vs Leaf Item Imports

To improve readability, try to avoid importing individual structs, functions, etc unless they are nonambiguous.

We prefer this:

```rust
use near_indexer::near_primitives;

fn my_func() {
    let my_account = near_primitives::types::Account::from("test.near");
}
```

over:

```rust
use near_indexer::near_primitives::types::Account;

fn my_func() {
    let my_account = Account::from("test.near");
}
```

The rationale behind this is that there are plenty of different `Account` types in various contexts (e.g. DB schema, NEAR account, local crate struct).

# Setting up the environment

`nearcore` uses nightly Rust features, so you will need nightly rust installed. See [this document](https://doc.rust-lang.org/1.2.0/book/nightly-rust.html) for details. NEAR Indexer for Explorer follows `rust-toolchain` specified by `nearcore`.

Majority of NEAR developers use CLion with Rust plugin as their primary IDE.

We also had success with VSCode with rust-analyzer, see the steps for installation [here](https://commonwealth.im/near/proposal/discussion/338-remote-development-with-vscode-and-rustanalyzer).

Some of us use VIM with [rust.vim](https://github.com/rust-lang/rust.vim) and [rusty-tags](https://github.com/dan-t/rusty-tags). It has fewer features than CLion or VSCode, but overall provides a usable setting.

Refer to [this document](https://docs.nearprotocol.com/docs/contribution/nearcore) for details on setting up your environment.
