# Quiet lane

Experimental only. Not a product.

A lab model for Kaspa testnet-10. Two seats seal a face. The prize is one exit leaf. A settlement that matches nothing still publishes that leaf, and the claim spends the smallest large-first slice of the delegate pool.

A hosted tic-tac-toe match on testnet-10, 1 Oct 2026, is written up in [TN10.md](TN10.md). This crate did not submit that match.

This crate is not a vprog. It does not load an ELF, call `runtime::run`, or enter the node VM. On kaspanet/vprogs master `f9b84a8`, `node/vm` `process_transaction` is `todo!`. The guest body that does run is `zk/backend/risc0/runtime-processor`, and the host check of that body is [STP-KAS/vprog-runtime-test](https://github.com/STP-KAS/vprog-runtime-test). This crate does not submit a transaction, load a wallet, or prove a receipt. The seal is a lab mix, not a Kaspa hash. [DISCLAIMER.md](DISCLAIMER.md).

## Test the guest

The check to run is [STP-KAS/vprog-runtime-test](https://github.com/STP-KAS/vprog-runtime-test).

```
git clone https://github.com/STP-KAS/vprog-runtime-test.git
cd vprog-runtime-test
cargo test -p vprog-runtime-test --manifest-path vprogs/Cargo.toml
```

Three tests. Balances 50 and 7, amount 20, land at 30 and 27. Amount 51 is a guest error. Two indexes that are both 0 are a guest error. The first run needs a network so cargo can fetch the locked rusty-kaspa commit. The command submits nothing.

`cargo test` in this repository is the settler model, 28 tests. It does not call that guest.

## What

`Table` is the modeled board. Each seat commits a digest, then reveals a face and a salt. Stone breaks Blade, Blade cuts Cloth, and Cloth covers Stone. The same face returns both stakes. A seat that never reveals, after the window, forfeits the pot to the seat that did. `note_l1` records a carrier. `execute` applies the reveal. The seats stay empty until `execute`.

`Lane::wake` is the settler path. `Policy::Master` is kaspanet/vprogs `f9b84a8`: the retained window only, so an unmatched boundary leaves the leaf in the suffix. `Policy::EarlyReturn` is the parent `fbd677c2`: both windows count, and a queued hit is enough to re-form. `Policy::FallThrough` is `edb9633a`: an unmatched boundary re-forms anyway. A second wake of the same front is guarded. A deferred proof on that fall-through still sets the guard. Master has no deferred-proof return. `latest: None` moves nothing. `rollback` clears the guard.

`select_delegates` is the claim loop from the tic-tac-toe driver at `5c37c146`. Largest coin first. A coin is taken while the sum already taken is still short of the leaf. The coin that crosses the leaf is included. Unselected coins stay. Excess returns as one change coin. The pool sum plus the paid leaf equals the pool sum from before the claim.

The table opens only on testnet-10. Mainnet and simnet are refused. A stake of 0 is refused. A published leaf can be claimed as soon as the wake emits it.

## Why

On 1 Oct 2026, [biryukovmaxim/vprog-tictactoe](https://github.com/biryukovmaxim/vprog-tictactoe) master is `cb91e862`, five commits after `098be674`. The guest board did not change in those five. What changed is the host around the guest.

A quiet lane was stranding exit leaves. On master `f9b84a8`, `reaggregate_superseded` returns when the settlement boundary matches no retained batch. The queued window is not read. The parent `fbd677c2` reads both windows and returns only when neither matches, so a queued hit drains that prefix and re-forms the suffix. After the watch had already drained a boundary, every later wake matched nothing, and the suffix waited for some unrelated batch. `edb9633a` falls through into the re-form. The guard is the front batch's checkpoint index, so a republish of the same front does not emit twice. `latest: None` returns on master and on `edb9633a`, and the guard stays until rollback.

A client that treats an L1 accept as a finished move is early. The driver report at `cb91e862` is the list of transaction ids L1 accepted. The guest can still reject the action.

A claim that attaches the whole delegate pool pays for mass it does not need. The driver now funds the leaf from a minimal largest-first subset. The coins are conserved. Change goes back to the pool.

The throw is the small game that makes those rules visible: the seals can sit on L1 before the guest runs, the winner's leaf can sit stranded on a quiet lane, a queued-only match publishes that leaf on the parent and not on master, and the payout does not sweep every delegate coin. The claim does not wait. The driver spends the leaf when the exit feed serves it.

## How

The match is ordinary state. `lab_seal(seat, face, salt)` is checked for equality at reveal. A bad salt consumes that step and leaves the commit in place, so a new step can reveal. Closing the match pushes one batch onto the retained suffix. Nothing is published until a fall-through wake. The claim then runs the delegate loop.

The DAA score is an argument. On 1 Oct 2026 the example was run with the live testnet-10 virtual DAA score. The score is a clock. The process does not open a socket.

```
cargo test
cargo run --example throw -- <virtualDaaScore>
```

`virtualDaaScore` is the field on `https://api-tn10.kaspa.org/info/blockdag`. Refuse the run if `networkName` is anything but `kaspa-testnet-10`.

## Sources

Read on 1 Oct 2026.

- [biryukovmaxim/vprog-tictactoe](https://github.com/biryukovmaxim/vprog-tictactoe) master `cb91e862` (29 Sep 2026). Five commits after `098be674`: `520fc7c` dev-server host headers, `5c37c146` minimal delegate subset, `4c73c3d9` pin `fbd677c2`, `9fdc9ebc` pin `edb9633a`, `cb91e862` "accepted by L1". The guest sources are absent from that diff.
- `driver/examples/claim.rs` at `5c37c146`. `driver/src/scenario.rs` at `cb91e862`.
- [kaspanet/vprogs](https://github.com/kaspanet/vprogs) master `f9b84a863a7c7c20586a9cf947550475e894f72e` (28 Jul 2026, no release). Branch `fix/exits-stranded-suffix` at `edb9633aafd9a282adb7cbdfe364cdda99715ed6` (29 Sep 2026), parent `fbd677c2a383e8e04ca8cde5f38c98e0128eb54f`. `zk/aggregate-prover/src/worker.rs`, `reaggregate_superseded` and `apply_rollback`. The tic-tac-toe `Cargo.lock` pins `edb9633a`, not master. Read again on 1 Oct 2026: master matches retained blocks only and returns when `settled_prefix` is `None`. The parent returns only when both windows miss. `edb9633a` does not return. On master, `prove_bundle` has no deferred outcome, and the guard is stored after `prove_bundle` returns. On `edb9633a` a deferred proof returns `BundleOutcome::Deferred` and the caller still stores the front index.
- The commit message on `edb9633a` also shortens the settler confirm tick from 30s to 5s. This crate does not run that clock.
- rusty-kaspa in the tic-tac-toe `Cargo.toml` remains `eb0a856d4e1ef9d884c5997bc43408091f5a5632`. The published node release v2.1.0 is `01b532e8`.
- [STP-KAS/declared-ply](https://github.com/STP-KAS/declared-ply) models the guest board at `098be674`. Its 25-step gap is a sequence gap between plies. An attempt closer than `last_seq + gap` is immature. That gap is not a claim clock, and the five commits after `098be674` do not touch the guest.

## What the run printed

`cargo test` is 28 tests, in debug and in release. Inside those tests: all 9 face pairs, 1,344 delegate selections, 2,000 random pools, the quiet-lane guard, and the master window against the parent.

`cargo run --example throw -- 585426931` printed:

```
network kaspa-testnet-10
daa 585426931
stake 100000000 sompi each
L1 accepted the two seals. The seats were still empty.
Guest executed both seals.
Stone breaks Blade.
Creator takes 200000000 sompi.
The leaf sits in the retained suffix. The settlement matches nothing.
Early return leaves the leaf in the suffix. The claim is stranded.
Fall through publishes the leaf. A second wake of the same front is guarded.
Master f9b84a8 strands that unmatched leaf. The queue is not its window.
A queued-only match still strands the leaf on master. The parent fbd677c2 publishes it.
An absent settlement moves nothing.
Claim spends delegate coins 2 and 3. Change 20000000 sompi returns. Coin 1 stays.
Paid 200000000. Pool before 260000000. Pool after 60000000.
```

`585426931` is the `virtualDaaScore` this desk read from testnet-10 immediately before that run, on 1 Oct 2026 about 17:12 UTC. `networkName` was `kaspa-testnet-10`. `/info/health` on that fetch said the node and the database were synced, and the accepted-tx lag was 1 second. An earlier fetch the same hour reported kaspad `serverVersion` 2.1.0. The fetch beside the run reported 2.0.1. That field moved, so it is not a pin. The published node pin stays rusty-kaspa v2.1.0 `01b532e8`.

The hosted demo at `https://vprogs-tt.izio.fr/api/state` answered on both fetches. Settled DAA stayed `584192938`. The L2 tip moved, from `3537096` to `3538729`. The virtual DAA on the second fetch was `1,233,993` above that settled score. This crate did not submit a game to that host. The host runs the tic-tac-toe guest, and this throw is a different program.

## Inconsistencies

| Where | What the files say |
| --- | --- |
| This crate | It is a model of the settler wake and the claim loop. It is not a guest program. Master `node/vm` does not execute transactions. |
| Which vprogs commit | kaspanet/vprogs master is `f9b84a8`. Tic-tac-toe master `cb91e862` pins branch `fix/exits-stranded-suffix` at `edb9633a`. |
| Master versus the parent | `f9b84a8` matches retained blocks only. A boundary that hits the queued window and misses retained returns, and the queued batch stays. `EarlyReturn` is `fbd677c2`, which drains that queued batch and re-forms the suffix. |
| Absent settlement | `latest: None` returns before the drain on master and on `edb9633a`. The guard stays. Rollback is what clears it. |
| Claim clock | `claim.rs` at `5c37c146` spends the leaf when `/api/exits` serves it. It does not add 25 to the birth DAA. The 25 in declared-ply is the sequence gap between plies. |
| README versus Cargo.toml | The tic-tac-toe README still names `fix/settlement-watch-wedge` in parentheses. `Cargo.toml` names `fix/exits-stranded-suffix`. |
| L1 versus guest | `scenario.rs` logs "accepted by L1" and says the guest may still reject the action. |
| Zero leaf | The driver selector returns no coins when the leaf amount is 0. This crate refuses a zero stake and refuses to pay a zero leaf. |
| Guest board | The five new commits do not touch the guest. The pending ring at `098be674` is unchanged. |
| Node version field | `/info/health` `serverVersion` was 2.1.0 and then 2.0.1 within the same hour. Neither string is the release commit. |
| Hash | `lab_seal` is a lab mix so the tests can check a reveal. A real guest would use the runtime hasher. |
| Proof | This crate does not prove. On `edb9633a`, `Prove::Deferred` sets the front guard and publishes nothing, because the caller stores the index after `prove_bundle` returns `BundleOutcome::Deferred`. Master has no such outcome: the same call is `WakeError::NoDeferral` and the lane stays put. |

Intentions are good; thought process is questionable. STP remains delusional. Si vis pacem, para bellum.
