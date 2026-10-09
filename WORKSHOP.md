# Workshop, 9 Oct 2026

Maksim Biriukov published [vprogs: a based rollup on Kaspa](https://biryukovmaxim.github.io/vprogs-workshop/). Michael Sutton pointed at that book the same day. A tweet is not a pin.

The book links kaspanet/vprogs `055ae28a` and biryukovmaxim/vprog-tictactoe `fe6b0e85`. On 9 Oct 2026 those trees were behind the live tips: `release-candidate` was `cc0d54bc` and tictactoe master was `f93e52fe`. The tictactoe root lock names `cc0d54bc`. The guest lock still names `04cfb0ae`.

This crate does not move. `Policy::Master` stays kaspanet/vprogs `f9b84a8`. `Policy::EarlyReturn` stays the parent `fbd677c2`. `Policy::FallThrough` stays `edb9633a`. `cargo test` here is still the settler model. It does not call the guest.

The rules lab for the workshop shape is [STP-KAS/vprog-sovereign](https://github.com/STP-KAS/vprog-sovereign). `cargo test` there is 14 tests. `cargo run --example town` walks one village. [MAP.md](https://github.com/STP-KAS/vprog-sovereign/blob/main/MAP.md) is the use-case map, including the LUMBRIDGE proposition. That map does not change sixpack.wtf. The lab submits nothing.
