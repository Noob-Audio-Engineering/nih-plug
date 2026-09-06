# This fork

This is Noob Audio Engineering's fork of [nih-plug]. It exists to carry changes
upstream has not taken, and it is meant to be kept close to upstream rather
than allowed to drift.

[nih-plug]: https://github.com/robbert-vdh/nih-plug

## Branches

| branch | what it is |
|---|---|
| `master` | a clean mirror of upstream. **Never commit to it.** |
| `noob` | our maintained line: upstream plus everything below. This is what the plug-ins depend on. |
| `host-resize` | the original single-patch branch, kept until every plug-in has moved to `noob`. |

## What we carry, and why

**Host-driven editor resizing** — `Editor::can_resize`, `check_size_constraint`
and `set_size`, with the matching work in the CLAP wrapper and the VST3 view.
Upstream has no way for a host to resize an editor, and our editors are web
views that a person drags the corner of. Without this the window and the page
disagree about how big they are.

**An Audio Unit backend** — see `src/wrapper/auv2`. nih-plug exports CLAP and
VST3; there is no AU, and on macOS that shuts out Logic entirely. The
alternative was to wrap our CLAP with `clap-wrapper`, which works and is what
the plug-ins ship today, but a wrapper is another layer between the host and
the audio and we would rather own the format than adapt to it.

Each of these is a separate commit on `noob`, and each keeps to nih-plug's own
shape and naming so that a future upstream merge is a merge and not a rewrite.

## Taking a new upstream

Do this in order. It is deliberately dull.

```sh
git remote add upstream https://github.com/robbert-vdh/nih-plug.git   # once
git fetch upstream

# 1. Move the mirror. Fast-forward only: if this refuses, someone has
#    committed to `master`, and the fix is to move that commit to `noob`
#    rather than to force anything.
git checkout master
git merge --ff-only upstream/master
git push origin master

# 2. Rebase our line on to it. Rebase and not merge, so `noob` stays
#    readable as "upstream, then our patches" and each patch can be dropped
#    the day upstream grows its own version of it.
git checkout noob
git rebase master

# 3. Prove it still builds, for every format we claim.
cargo check --workspace --all-features
cargo test --workspace
cargo check --features auv2 --target aarch64-apple-darwin    # macOS only

# 4. Push the rebased line. `--force-with-lease`, never `--force`: the lease
#    is what refuses when somebody else has pushed in the meantime.
git push --force-with-lease origin noob
```

Then, in each plug-in repository:

```sh
cargo update -p nih_plug
cargo test && cargo clippy --all-targets --features plugin
```

and let its own pipeline build the bundles. **Update one plug-in first and let
it go green before doing the other five**, so a bad upstream is found on one
repository rather than on six.

## Keeping a patch small enough to survive

The reason this fork is cheap to maintain is that nothing in it is clever:

- **Touch as few files as possible.** The resize patch is five files. A patch
  spread over thirty conflicts on every upstream release.
- **Follow the surrounding code**, including its naming and its comment style,
  so a conflict is resolved by reading rather than by guessing what we meant.
- **Never reformat.** A stray `cargo fmt` over an untouched file turns a clean
  rebase into a hand merge.
- **Prefer adding a module to editing one.** The AU backend is almost entirely
  new files under `src/wrapper/auv2` for exactly this reason: new files never
  conflict.

## If upstream takes one of ours

Drop the commit from `noob` at the next rebase and delete its section above.
The point of the fork is to get smaller.
