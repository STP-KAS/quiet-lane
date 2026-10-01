//! A two-seat sealed throw, and the quiet-lane wake that publishes its prize.
//!
//! The throw is the lab guest. Each seat seals a face, then reveals it. Stone
//! breaks Blade, Blade cuts Cloth, Cloth covers Stone. The prize is an exit
//! leaf sitting in the retained suffix.
//!
//! [`Lane::wake`] follows `reaggregate_superseded` in
//! `zk/aggregate-prover/src/worker.rs`.
//! [`Policy::Master`] is kaspanet/vprogs `f9b84a8`: the retained window only,
//! and an unmatched boundary returns before the re-form.
//! [`Policy::EarlyReturn`] is the parent `fbd677c2`: both windows, and an
//! unmatched pair returns.
//! [`Policy::FallThrough`] is `edb9633a`: both windows, and an unmatched pair
//! re-forms. The front-index guard makes a second wake of the same front a no-op.
//!
//! [`select_delegates`] is the claim loop in biryukovmaxim/vprog-tictactoe
//! `driver/examples/claim.rs` at `5c37c146`: largest first, stop once the coins
//! already taken cover the leaf. A published leaf is claimable immediately.
//! The 25-step gap in the declared-ply note is a sequence gap between plies,
//! not a wait on this claim.
//!
//! [`Table::note_l1`] records a carrier. [`Table::execute`] is the guest.
//! A carrier can be on L1 while the seats are still empty, which is the split
//! named in `driver/src/scenario.rs` at `cb91e862`.
//!
//! This crate submits nothing and proves nothing. [`lab_seal`] is a lab mix,
//! not a Kaspa hash.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Sompi in one tKAS.
pub const SOMPI_PER_TKAS: u64 = 100_000_000;

/// Lab seal domain byte.
const SEAL_DOM: u8 = b'S';

/// Lab seal mix constant.
const SEAL_MIX: u8 = 0xA5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    /// Kaspa testnet-10. The only network this table opens.
    Testnet10,
    Mainnet,
    Simnet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    Creator = 1,
    Joiner = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Stone = 1,
    Cloth = 2,
    Blade = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StepId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Commit {
        id: StepId,
        seat: Seat,
        digest: [u8; 32],
        daa: u64,
    },
    Reveal {
        id: StepId,
        seat: Seat,
        face: Face,
        salt: [u8; 16],
        daa: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    /// One seat takes the whole pot.
    Win(Seat),
    /// Each seat receives its stake back.
    Draw,
    /// The seat that revealed takes the pot after the window.
    Forfeit(Seat),
    /// Neither seat revealed. Each stake returns.
    Return,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// kaspanet/vprogs master `f9b84a8`. Retained window only.
    Master,
    /// Parent `fbd677c2`. Both windows. An unmatched pair returns.
    EarlyReturn,
    /// `edb9633a`. Both windows. An unmatched pair re-forms.
    FallThrough,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prove {
    /// The suffix proof emitted.
    Emitted,
    /// The suffix proof deferred. The guard still advances.
    Deferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// Leading queued batches this settlement covers. `None` and `Some(0)` match nothing.
    pub queued_prefix: Option<usize>,
    /// Leading retained batches this settlement covers. `None` and `Some(0)` match nothing.
    pub retained_prefix: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reform {
    /// `latest` was `None`. The lane was not changed.
    Absent,
    /// The unmatched boundary returned before any re-form.
    EarlyReturn,
    /// The retained suffix was empty, and the guard was cleared.
    EmptySuffix,
    /// This front was already re-formed.
    Guarded,
    /// The proof deferred. No leaf was published. The guard advanced.
    Deferred,
    /// An all-empty suffix emitted a no-op. The guard advanced.
    Noop,
    /// `n` leaves were appended to the published list. The guard advanced.
    Published(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WakeReport {
    pub dropped_queued: usize,
    pub dropped_retained: usize,
    pub reform: Reform,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub index: u64,
    pub has_txs: bool,
    pub leaves: Vec<Leaf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leaf {
    pub id: u64,
    pub seat: Seat,
    pub sompi: u64,
    pub born_daa: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coin {
    pub id: u64,
    pub sompi: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenSpec {
    pub network: Network,
    pub stake: u64,
    pub reveal_window: u64,
    pub daa: u64,
    pub bundle_cap: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Claim {
    pub seat: Seat,
    pub leaf: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payout {
    pub leaf: u64,
    pub seat: Seat,
    pub paid: u64,
    pub spent: Vec<u64>,
    pub change: Option<Coin>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptReport {
    pub lines: Vec<String>,
    pub paid: u64,
    pub pool_before: u64,
    pub pool_after: u64,
    pub published: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenError {
    WrongNetwork,
    ZeroStake,
    ZeroWindow,
    ZeroCap,
    StakeOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateError {
    AlreadyOnL1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecError {
    NotOnL1,
    AlreadyExecuted,
    DaaRewind,
    SeatTaken,
    SealClosed,
    TooEarly,
    TooLate,
    DaaOverflow,
    BadSeal,
    SecondReveal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseError {
    AlreadyClosed,
    SealsOpen,
    DaaRewind,
    WindowOpen,
    DaaOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeError {
    BadPrefix,
    /// Master has no deferred-proof return. The lane was not changed.
    NoDeferral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimError {
    Stranded,
    WrongSeat,
    AlreadyClaimed,
    ZeroLeaf,
    ShortPool,
    Overflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptError {
    Open(OpenError),
    Gate(GateError),
    Exec(ExecError),
    Close(CloseError),
    Wake(WakeError),
    Claim(ClaimError),
    Shape(&'static str),
}

/// Who wins when both faces are on the table. `None` is a draw.
pub fn judge(creator: Face, joiner: Face) -> Option<Seat> {
    use Face::*;
    match (creator, joiner) {
        (Stone, Blade) | (Blade, Cloth) | (Cloth, Stone) => Some(Seat::Creator),
        (a, b) if a == b => None,
        _ => Some(Seat::Joiner),
    }
}

/// One-line name of a deciding pair. The winner's face is first.
pub fn blow(win: Face, lose: Face) -> &'static str {
    match (win, lose) {
        (Face::Stone, Face::Blade) => "Stone breaks Blade.",
        (Face::Blade, Face::Cloth) => "Blade cuts Cloth.",
        (Face::Cloth, Face::Stone) => "Cloth covers Stone.",
        _ => "The faces do not decide.",
    }
}

/// Lab seal of a seat, a face, and a 16-byte salt.
///
/// Equal inputs return equal bytes. This is not blake2b and not a sighash.
pub fn lab_seal(seat: Seat, face: Face, salt: &[u8; 16]) -> [u8; 32] {
    let mut state = [0u8; 32];
    state[0] = SEAL_DOM;
    state[1] = seat as u8;
    state[2] = face as u8;
    state[3] = SEAL_MIX;
    state[4..20].copy_from_slice(salt);
    for round in 0..8u8 {
        for i in 0..32 {
            let a = state[i];
            let b = state[(i + 7) % 32];
            let c = state[(i + 13) % 32];
            state[i] = a.wrapping_mul(31).wrapping_add(b).rotate_left(3) ^ c ^ round;
        }
    }
    state
}

/// Largest-first subset that covers `leaf`.
///
/// A coin is taken while the sum already taken is still short. The coin that
/// crosses `leaf` is included. A `leaf` of 0 selects nothing. Equal amounts
/// keep their input order. Addition saturates, matching the driver loop.
pub fn select_delegates(coins: &[Coin], leaf: u64) -> Vec<Coin> {
    let mut ranked = coins.to_vec();
    ranked.sort_by_key(|coin| std::cmp::Reverse(coin.sompi));
    let mut funded: u64 = 0;
    ranked
        .into_iter()
        .take_while(|coin| {
            let enough = funded >= leaf;
            funded = funded.saturating_add(coin.sompi);
            !enough
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lane {
    bundle_cap: usize,
    queued: VecDeque<Batch>,
    retained: VecDeque<Batch>,
    last_reformed_from: Option<u64>,
    published: Vec<Leaf>,
}

impl Lane {
    /// An empty lane. `bundle_cap` is the maximum suffix length of one re-form.
    pub fn new(bundle_cap: usize) -> Result<Self, OpenError> {
        if bundle_cap == 0 {
            return Err(OpenError::ZeroCap);
        }
        Ok(Self {
            bundle_cap,
            queued: VecDeque::new(),
            retained: VecDeque::new(),
            last_reformed_from: None,
            published: Vec::new(),
        })
    }

    /// Appends one batch that has not been proved yet.
    pub fn push_queued(&mut self, batch: Batch) {
        self.queued.push_back(batch);
    }

    /// Appends one proved batch that a settlement has not covered yet.
    pub fn push_retained(&mut self, batch: Batch) {
        self.retained.push_back(batch);
    }

    /// First-batch index of the most recent re-form, if the guard is set.
    pub fn last_reformed_from(&self) -> Option<u64> {
        self.last_reformed_from
    }

    /// Leaves emitted by re-forms, in emission order. A rollback can repeat an id.
    pub fn published(&self) -> &[Leaf] {
        &self.published
    }

    pub fn queued_len(&self) -> usize {
        self.queued.len()
    }

    pub fn retained_len(&self) -> usize {
        self.retained.len()
    }

    /// `latest: None` on master `f9b84a8` and on `edb9633a`.
    ///
    /// The lane does not drain, re-form, or clear the guard. Rollback is the
    /// path that truncates.
    pub fn settlement_absent(&self) -> WakeReport {
        WakeReport {
            dropped_queued: 0,
            dropped_retained: 0,
            reform: Reform::Absent,
        }
    }

    /// Applies one settlement wake.
    ///
    /// [`Policy::Master`] ignores the queued prefix, including a count past
    /// the queue. An unmatched retained boundary returns with the lane
    /// unchanged. A deferred proof is [`WakeError::NoDeferral`] and changes
    /// nothing.
    ///
    /// On the other two policies a prefix longer than its window is
    /// [`WakeError::BadPrefix`] and changes nothing. An unmatched boundary
    /// under [`Policy::EarlyReturn`] returns before the drain. Every other
    /// wake drains the matched prefixes, then re-forms up to `bundle_cap`
    /// retained batches from the front. Those batches stay retained.
    pub fn wake(
        &mut self,
        policy: Policy,
        boundary: Boundary,
        prove: Prove,
    ) -> Result<WakeReport, WakeError> {
        if policy == Policy::Master {
            return self.wake_master(boundary, prove);
        }
        let queued_n = prefix(boundary.queued_prefix, self.queued.len())?;
        let retained_n = prefix(boundary.retained_prefix, self.retained.len())?;
        if queued_n.is_none() && retained_n.is_none() && policy == Policy::EarlyReturn {
            return Ok(WakeReport {
                dropped_queued: 0,
                dropped_retained: 0,
                reform: Reform::EarlyReturn,
            });
        }
        if let Some(n) = queued_n {
            self.queued.drain(0..n);
        }
        if let Some(n) = retained_n {
            self.retained.drain(0..n);
        }
        let reform = self.reform(prove);
        Ok(WakeReport {
            dropped_queued: queued_n.unwrap_or(0),
            dropped_retained: retained_n.unwrap_or(0),
            reform,
        })
    }

    /// Drops batches whose index is above `target_index`, and clears the guard.
    pub fn rollback(&mut self, target_index: u64) {
        self.queued.retain(|batch| batch.index <= target_index);
        self.retained.retain(|batch| batch.index <= target_index);
        self.last_reformed_from = None;
    }

    /// Master `f9b84a8`: retained match, then the same re-form.
    ///
    /// The queued prefix is not read. A deferred proof returns
    /// [`WakeError::NoDeferral`] before any drain.
    fn wake_master(
        &mut self,
        boundary: Boundary,
        prove: Prove,
    ) -> Result<WakeReport, WakeError> {
        if prove == Prove::Deferred {
            return Err(WakeError::NoDeferral);
        }
        let Some(n) = prefix(boundary.retained_prefix, self.retained.len())? else {
            return Ok(WakeReport {
                dropped_queued: 0,
                dropped_retained: 0,
                reform: Reform::EarlyReturn,
            });
        };
        self.retained.drain(0..n);
        let reform = self.reform(prove);
        Ok(WakeReport {
            dropped_queued: 0,
            dropped_retained: n,
            reform,
        })
    }

    /// Re-forms the retained front. See [`Reform`] for each result.
    fn reform(&mut self, prove: Prove) -> Reform {
        let suffix_len = self.retained.len().min(self.bundle_cap);
        if suffix_len == 0 {
            self.last_reformed_from = None;
            return Reform::EmptySuffix;
        }
        let suffix_from = self.retained[0].index;
        if self.last_reformed_from == Some(suffix_from) {
            return Reform::Guarded;
        }
        let has_txs = self
            .retained
            .iter()
            .take(suffix_len)
            .any(|batch| batch.has_txs);
        if has_txs && prove == Prove::Deferred {
            self.last_reformed_from = Some(suffix_from);
            return Reform::Deferred;
        }
        if !has_txs {
            self.last_reformed_from = Some(suffix_from);
            return Reform::Noop;
        }
        let mut n = 0;
        for batch in self.retained.iter().take(suffix_len) {
            for leaf in &batch.leaves {
                self.published.push(*leaf);
                n += 1;
            }
        }
        self.last_reformed_from = Some(suffix_from);
        Reform::Published(n)
    }
}

/// `None` and `Some(0)` match nothing. A count past `len` is [`WakeError::BadPrefix`].
fn prefix(raw: Option<usize>, len: usize) -> Result<Option<usize>, WakeError> {
    match raw {
        None | Some(0) => Ok(None),
        Some(n) if n > len => Err(WakeError::BadPrefix),
        Some(n) => Ok(Some(n)),
    }
}

#[derive(Clone, Debug)]
struct SeatState {
    digest: Option<[u8; 32]>,
    face: Option<Face>,
}

#[derive(Clone, Debug)]
struct Noted {
    step: Step,
    done: bool,
}

#[derive(Clone, Debug)]
pub struct Table {
    stake: u64,
    pot: u64,
    reveal_window: u64,
    daa: u64,
    opened_daa: Option<u64>,
    seats: [SeatState; 2],
    noted: BTreeMap<StepId, Noted>,
    finish: Option<Finish>,
    next_leaf: u64,
    next_coin: u64,
    pool: Vec<Coin>,
    claimed: BTreeSet<u64>,
    lane: Lane,
}

impl Table {
    /// Opens a two-seat table on testnet-10.
    pub fn open(spec: OpenSpec) -> Result<Self, OpenError> {
        if spec.network != Network::Testnet10 {
            return Err(OpenError::WrongNetwork);
        }
        if spec.stake == 0 {
            return Err(OpenError::ZeroStake);
        }
        if spec.reveal_window == 0 {
            return Err(OpenError::ZeroWindow);
        }
        let pot = spec.stake.checked_mul(2).ok_or(OpenError::StakeOverflow)?;
        let lane = Lane::new(spec.bundle_cap)?;
        Ok(Self {
            stake: spec.stake,
            pot,
            reveal_window: spec.reveal_window,
            daa: spec.daa,
            opened_daa: None,
            seats: [
                SeatState {
                    digest: None,
                    face: None,
                },
                SeatState {
                    digest: None,
                    face: None,
                },
            ],
            noted: BTreeMap::new(),
            finish: None,
            next_leaf: 1,
            next_coin: 1,
            pool: Vec::new(),
            claimed: BTreeSet::new(),
            lane,
        })
    }

    /// Records one carrier as accepted by L1. The seats do not move.
    pub fn note_l1(&mut self, step: Step) -> Result<(), GateError> {
        let id = step_id(step);
        if self.noted.contains_key(&id) {
            return Err(GateError::AlreadyOnL1);
        }
        self.noted.insert(id, Noted { step, done: false });
        Ok(())
    }

    /// Runs one noted carrier in the guest.
    ///
    /// A rejected write still consumes the id. A second call is
    /// [`ExecError::AlreadyExecuted`].
    pub fn execute(&mut self, id: StepId) -> Result<(), ExecError> {
        let noted = self.noted.get_mut(&id).ok_or(ExecError::NotOnL1)?;
        if noted.done {
            return Err(ExecError::AlreadyExecuted);
        }
        let step = noted.step;
        noted.done = true;
        let daa = step_daa(step);
        if daa < self.daa {
            return Err(ExecError::DaaRewind);
        }
        self.daa = daa;
        match step {
            Step::Commit { seat, digest, .. } => self.seal(seat, digest),
            Step::Reveal {
                seat,
                face,
                salt,
                daa,
                ..
            } => self.reveal(seat, face, salt, daa),
        }
    }

    /// True after a commit has been executed for `seat`.
    pub fn sealed(&self, seat: Seat) -> bool {
        self.seats[seat_index(seat)].digest.is_some()
    }

    /// The revealed face, after a seal that matched.
    pub fn revealed(&self, seat: Seat) -> Option<Face> {
        self.seats[seat_index(seat)].face
    }

    pub fn finish(&self) -> Option<Finish> {
        self.finish
    }

    pub fn pool(&self) -> &[Coin] {
        &self.pool
    }

    /// Sum of the delegate pool. Overflow saturates at `u64::MAX`.
    pub fn pool_total(&self) -> u64 {
        self.pool
            .iter()
            .fold(0u64, |sum, coin| sum.saturating_add(coin.sompi))
    }

    pub fn published(&self) -> &[Leaf] {
        self.lane.published()
    }

    pub fn lane(&self) -> &Lane {
        &self.lane
    }

    /// Adds one delegate coin and returns its id.
    pub fn deposit_delegate(&mut self, sompi: u64) -> u64 {
        let id = self.next_coin;
        self.next_coin = self.next_coin.saturating_add(1);
        self.pool.push(Coin { id, sompi });
        id
    }

    /// Freezes the match and pushes the prize into the retained suffix.
    pub fn close(&mut self, daa: u64) -> Result<Finish, CloseError> {
        if self.finish.is_some() {
            return Err(CloseError::AlreadyClosed);
        }
        let opened = self.opened_daa.ok_or(CloseError::SealsOpen)?;
        if daa < self.daa {
            return Err(CloseError::DaaRewind);
        }
        let end = opened
            .checked_add(self.reveal_window)
            .ok_or(CloseError::DaaOverflow)?;
        let creator = self.seats[0].face;
        let joiner = self.seats[1].face;
        let finish = match (creator, joiner) {
            (Some(a), Some(b)) => match judge(a, b) {
                Some(seat) => Finish::Win(seat),
                None => Finish::Draw,
            },
            (Some(_), None) => finish_after_window(daa, end, Finish::Forfeit(Seat::Creator))?,
            (None, Some(_)) => finish_after_window(daa, end, Finish::Forfeit(Seat::Joiner))?,
            (None, None) => finish_after_window(daa, end, Finish::Return)?,
        };
        self.daa = daa;
        self.finish = Some(finish);
        let leaves = self.prize_leaves(finish, daa);
        let index = self.next_batch_index();
        self.lane.push_retained(Batch {
            index,
            has_txs: true,
            leaves,
        });
        Ok(finish)
    }

    pub fn wake(
        &mut self,
        policy: Policy,
        boundary: Boundary,
        prove: Prove,
    ) -> Result<WakeReport, WakeError> {
        self.lane.wake(policy, boundary, prove)
    }

    pub fn rollback(&mut self, target_index: u64) {
        self.lane.rollback(target_index);
    }

    /// Pays one published leaf from the smallest large-first delegate slice.
    ///
    /// The selected coins leave the pool. Excess comes back as one change coin.
    /// Coins that were not selected stay, with the same ids. The pool sum plus
    /// the paid leaf equals the pool sum from before the claim. Publication is
    /// the gate. There is no DAA wait.
    pub fn claim(&mut self, claim: Claim) -> Result<Payout, ClaimError> {
        if self.claimed.contains(&claim.leaf) {
            return Err(ClaimError::AlreadyClaimed);
        }
        let leaf = self
            .lane
            .published()
            .iter()
            .find(|leaf| leaf.id == claim.leaf)
            .copied()
            .ok_or(ClaimError::Stranded)?;
        if leaf.seat != claim.seat {
            return Err(ClaimError::WrongSeat);
        }
        if leaf.sompi == 0 {
            return Err(ClaimError::ZeroLeaf);
        }
        let selected = select_delegates(&self.pool, leaf.sompi);
        let mut sum = 0u64;
        for coin in &selected {
            sum = sum.checked_add(coin.sompi).ok_or(ClaimError::Overflow)?;
        }
        if sum < leaf.sompi {
            return Err(ClaimError::ShortPool);
        }
        let spent: Vec<u64> = selected.iter().map(|coin| coin.id).collect();
        self.pool.retain(|coin| !spent.contains(&coin.id));
        let excess = sum - leaf.sompi;
        let change = if excess == 0 {
            None
        } else {
            let id = self.next_coin;
            self.next_coin = self.next_coin.saturating_add(1);
            let coin = Coin { id, sompi: excess };
            self.pool.push(coin);
            Some(coin)
        };
        self.claimed.insert(claim.leaf);
        Ok(Payout {
            leaf: leaf.id,
            seat: leaf.seat,
            paid: leaf.sompi,
            spent,
            change,
        })
    }

    /// Stores a commit. The second commit opens the reveal window at the current DAA.
    fn seal(&mut self, seat: Seat, digest: [u8; 32]) -> Result<(), ExecError> {
        let slot = &mut self.seats[seat_index(seat)];
        if slot.digest.is_some() {
            return Err(ExecError::SeatTaken);
        }
        slot.digest = Some(digest);
        if self.seats.iter().all(|seat| seat.digest.is_some()) {
            self.opened_daa = Some(self.daa);
        }
        Ok(())
    }

    /// Checks the window and the lab seal, then stores the face.
    fn reveal(
        &mut self,
        seat: Seat,
        face: Face,
        salt: [u8; 16],
        daa: u64,
    ) -> Result<(), ExecError> {
        let opened = self.opened_daa.ok_or(ExecError::SealClosed)?;
        if daa < opened {
            return Err(ExecError::TooEarly);
        }
        let end = opened
            .checked_add(self.reveal_window)
            .ok_or(ExecError::DaaOverflow)?;
        if daa > end {
            return Err(ExecError::TooLate);
        }
        let slot = &mut self.seats[seat_index(seat)];
        let digest = slot.digest.ok_or(ExecError::SealClosed)?;
        if lab_seal(seat, face, &salt) != digest {
            return Err(ExecError::BadSeal);
        }
        if slot.face.is_some() {
            return Err(ExecError::SecondReveal);
        }
        slot.face = Some(face);
        Ok(())
    }

    fn prize_leaves(&mut self, finish: Finish, born_daa: u64) -> Vec<Leaf> {
        let mut one = |seat: Seat, sompi: u64| {
            let id = self.next_leaf;
            self.next_leaf = self.next_leaf.saturating_add(1);
            Leaf {
                id,
                seat,
                sompi,
                born_daa,
            }
        };
        match finish {
            Finish::Win(seat) | Finish::Forfeit(seat) => vec![one(seat, self.pot)],
            Finish::Draw | Finish::Return => {
                vec![
                    one(Seat::Creator, self.stake),
                    one(Seat::Joiner, self.stake),
                ]
            }
        }
    }

    fn next_batch_index(&self) -> u64 {
        self.lane
            .retained
            .back()
            .map(|batch| batch.index.saturating_add(1))
            .unwrap_or(1)
    }
}

fn finish_after_window(daa: u64, end: u64, finish: Finish) -> Result<Finish, CloseError> {
    let after = end.checked_add(1).ok_or(CloseError::DaaOverflow)?;
    if daa < after {
        return Err(CloseError::WindowOpen);
    }
    Ok(finish)
}

fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::Creator => 0,
        Seat::Joiner => 1,
    }
}

fn step_id(step: Step) -> StepId {
    match step {
        Step::Commit { id, .. } | Step::Reveal { id, .. } => id,
    }
}

fn step_daa(step: Step) -> u64 {
    match step {
        Step::Commit { daa, .. } | Step::Reveal { daa, .. } => daa,
    }
}

/// Plays Stone against Blade at `daa` and claims the pot on the fall-through wake.
///
/// The early-return wake is run on a clone, so the paid table is the fall-through.
pub fn script(daa: u64) -> Result<ScriptReport, ScriptError> {
    let stake = SOMPI_PER_TKAS;
    let mut table = Table::open(OpenSpec {
        network: Network::Testnet10,
        stake,
        reveal_window: 100,
        daa,
        bundle_cap: 4,
    })?;
    let salt_creator = [0x11u8; 16];
    let salt_joiner = [0x22u8; 16];
    let creator = lab_seal(Seat::Creator, Face::Stone, &salt_creator);
    let joiner = lab_seal(Seat::Joiner, Face::Blade, &salt_joiner);
    table.note_l1(Step::Commit {
        id: StepId(1),
        seat: Seat::Creator,
        digest: creator,
        daa,
    })?;
    table.note_l1(Step::Commit {
        id: StepId(2),
        seat: Seat::Joiner,
        digest: joiner,
        daa,
    })?;
    if table.sealed(Seat::Creator) || table.sealed(Seat::Joiner) {
        return Err(ScriptError::Shape("seats moved on L1 accept"));
    }
    table.execute(StepId(1))?;
    table.execute(StepId(2))?;
    table.note_l1(Step::Reveal {
        id: StepId(3),
        seat: Seat::Creator,
        face: Face::Stone,
        salt: salt_creator,
        daa,
    })?;
    table.note_l1(Step::Reveal {
        id: StepId(4),
        seat: Seat::Joiner,
        face: Face::Blade,
        salt: salt_joiner,
        daa,
    })?;
    table.execute(StepId(3))?;
    table.execute(StepId(4))?;
    let finish = table.close(daa)?;
    if finish != Finish::Win(Seat::Creator) {
        return Err(ScriptError::Shape("stone should break blade"));
    }

    let quiet = Boundary {
        queued_prefix: None,
        retained_prefix: None,
    };
    let mut early = table.clone();
    let early_wake = early.wake(Policy::EarlyReturn, quiet, Prove::Emitted)?;
    if early_wake.reform != Reform::EarlyReturn {
        return Err(ScriptError::Shape("early wake should return"));
    }
    let stranded = early.claim(Claim {
        seat: Seat::Creator,
        leaf: 1,
    });
    if stranded != Err(ClaimError::Stranded) {
        return Err(ScriptError::Shape("early claim should be stranded"));
    }
    let mut master_table = table.clone();
    let master_quiet = master_table.wake(Policy::Master, quiet, Prove::Emitted)?;
    if master_quiet.reform != Reform::EarlyReturn || !master_table.published().is_empty() {
        return Err(ScriptError::Shape("master should strand an unmatched leaf"));
    }
    let absent = master_table.lane().settlement_absent();
    if absent.reform != Reform::Absent || master_table.lane().retained_len() != 1 {
        return Err(ScriptError::Shape("an absent settlement should move nothing"));
    }

    let mut queued_lane = Lane::new(4)?;
    queued_lane.push_queued(Batch {
        index: 1,
        has_txs: true,
        leaves: vec![],
    });
    queued_lane.push_retained(Batch {
        index: 2,
        has_txs: true,
        leaves: vec![Leaf {
            id: 1,
            seat: Seat::Creator,
            sompi: stake * 2,
            born_daa: daa,
        }],
    });
    let queued_only = Boundary {
        queued_prefix: Some(1),
        retained_prefix: None,
    };
    let master_queued = queued_lane
        .wake(Policy::Master, queued_only, Prove::Emitted)?;
    if master_queued.reform != Reform::EarlyReturn
        || queued_lane.queued_len() != 1
        || !queued_lane.published().is_empty()
    {
        return Err(ScriptError::Shape(
            "master should ignore a queued-only match",
        ));
    }
    let mut parent = queued_lane.clone();
    let parent_wake = parent.wake(Policy::EarlyReturn, queued_only, Prove::Emitted)?;
    if parent_wake.dropped_queued != 1
        || parent_wake.reform != Reform::Published(1)
        || parent.queued_len() != 0
    {
        return Err(ScriptError::Shape(
            "parent should publish on a queued-only match",
        ));
    }

    let published = table.wake(Policy::FallThrough, quiet, Prove::Emitted)?;
    if published.reform != Reform::Published(1) {
        return Err(ScriptError::Shape("fall through should publish one leaf"));
    }
    let again = table.wake(Policy::FallThrough, quiet, Prove::Emitted)?;
    if again.reform != Reform::Guarded {
        return Err(ScriptError::Shape("republish should be guarded"));
    }

    table.deposit_delegate(40_000_000);
    table.deposit_delegate(150_000_000);
    table.deposit_delegate(70_000_000);
    let pool_before = table.pool_total();
    let payout = table.claim(Claim {
        seat: Seat::Creator,
        leaf: 1,
    })?;
    let pool_after = table.pool_total();
    if payout.paid != stake * 2 || pool_before != pool_after + payout.paid {
        return Err(ScriptError::Shape("pool was not conserved"));
    }
    if payout.spent != vec![2, 3] {
        return Err(ScriptError::Shape(
            "claim did not take the minimal large coins",
        ));
    }
    if payout.change
        != Some(Coin {
            id: 4,
            sompi: 20_000_000,
        })
    {
        return Err(ScriptError::Shape("change was wrong"));
    }
    if table.pool()
        != [
            Coin {
                id: 1,
                sompi: 40_000_000,
            },
            Coin {
                id: 4,
                sompi: 20_000_000,
            },
        ]
    {
        return Err(ScriptError::Shape("pool leftovers were wrong"));
    }

    let lines = vec![
        format!("network kaspa-testnet-10"),
        format!("daa {daa}"),
        format!("stake {stake} sompi each"),
        "L1 accepted the two seals. The seats were still empty.".to_string(),
        "Guest executed both seals.".to_string(),
        blow(Face::Stone, Face::Blade).to_string(),
        format!("Creator takes {pot} sompi.", pot = stake * 2),
        "The leaf sits in the retained suffix. The settlement matches nothing.".to_string(),
        "Early return leaves the leaf in the suffix. The claim is stranded.".to_string(),
        "Fall through publishes the leaf. A second wake of the same front is guarded.".to_string(),
        "Master f9b84a8 strands that unmatched leaf. The queue is not its window.".to_string(),
        "A queued-only match still strands the leaf on master. The parent fbd677c2 publishes it."
            .to_string(),
        "An absent settlement moves nothing.".to_string(),
        format!(
            "Claim spends delegate coins 2 and 3. Change {} sompi returns. Coin 1 stays.",
            20_000_000
        ),
        format!(
            "Paid {paid}. Pool before {pool_before}. Pool after {pool_after}.",
            paid = payout.paid
        ),
    ];
    Ok(ScriptReport {
        lines,
        paid: payout.paid,
        pool_before,
        pool_after,
        published: table.published().len(),
    })
}

impl From<OpenError> for ScriptError {
    fn from(err: OpenError) -> Self {
        Self::Open(err)
    }
}

impl From<GateError> for ScriptError {
    fn from(err: GateError) -> Self {
        Self::Gate(err)
    }
}

impl From<ExecError> for ScriptError {
    fn from(err: ExecError) -> Self {
        Self::Exec(err)
    }
}

impl From<CloseError> for ScriptError {
    fn from(err: CloseError) -> Self {
        Self::Close(err)
    }
}

impl From<WakeError> for ScriptError {
    fn from(err: WakeError) -> Self {
        Self::Wake(err)
    }
}

impl From<ClaimError> for ScriptError {
    fn from(err: ClaimError) -> Self {
        Self::Claim(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn salt(byte: u8) -> [u8; 16] {
        [byte; 16]
    }

    fn open_at(daa: u64) -> Table {
        Table::open(OpenSpec {
            network: Network::Testnet10,
            stake: 50,
            reveal_window: 10,
            daa,
            bundle_cap: 4,
        })
        .unwrap()
    }

    fn commit(id: u64, seat: Seat, face: Face, salt: [u8; 16], daa: u64) -> Step {
        Step::Commit {
            id: StepId(id),
            seat,
            digest: lab_seal(seat, face, &salt),
            daa,
        }
    }

    fn reveal(id: u64, seat: Seat, face: Face, salt: [u8; 16], daa: u64) -> Step {
        Step::Reveal {
            id: StepId(id),
            seat,
            face,
            salt,
            daa,
        }
    }

    fn note_exec(table: &mut Table, step: Step) {
        let id = step_id(step);
        table.note_l1(step).unwrap();
        table.execute(id).unwrap();
    }

    fn both_sealed(table: &mut Table, a: Face, b: Face, daa: u64) {
        note_exec(table, commit(1, Seat::Creator, a, salt(1), daa));
        note_exec(table, commit(2, Seat::Joiner, b, salt(2), daa));
    }

    fn both_revealed(table: &mut Table, a: Face, b: Face, daa: u64) {
        note_exec(table, reveal(3, Seat::Creator, a, salt(1), daa));
        note_exec(table, reveal(4, Seat::Joiner, b, salt(2), daa));
    }

    #[test]
    fn lab_seal_is_stable_and_separates_inputs() {
        let salt = [0x11u8; 16];
        let stone = lab_seal(Seat::Creator, Face::Stone, &salt);
        assert_eq!(
            stone,
            [
                0x50, 0x7e, 0x41, 0x2e, 0x5b, 0x51, 0x98, 0x9a, 0x43, 0xfe, 0x7d, 0x1e, 0x3d, 0x6b,
                0x06, 0xb3, 0x77, 0x06, 0x7c, 0x3c, 0xe9, 0x76, 0x4d, 0xea, 0x30, 0x76, 0xfe, 0x5c,
                0x1d, 0x79, 0xb8, 0x9e,
            ]
        );
        assert_eq!(stone, lab_seal(Seat::Creator, Face::Stone, &salt));
        assert_ne!(stone, lab_seal(Seat::Joiner, Face::Stone, &salt));
        assert_ne!(stone, lab_seal(Seat::Creator, Face::Cloth, &salt));
        let mut other = salt;
        other[0] = 0x12;
        assert_ne!(stone, lab_seal(Seat::Creator, Face::Stone, &other));
    }

    #[test]
    fn lab_seal_has_no_collision_in_a_small_cube() {
        let mut seen = BTreeSet::new();
        for seat in [Seat::Creator, Seat::Joiner] {
            for face in [Face::Stone, Face::Cloth, Face::Blade] {
                for byte in 0..32u8 {
                    assert!(seen.insert(lab_seal(seat, face, &salt(byte))));
                }
            }
        }
        assert_eq!(seen.len(), 2 * 3 * 32);
    }

    #[test]
    fn open_refuses_the_other_networks_and_the_zero_holes() {
        let spec = OpenSpec {
            network: Network::Testnet10,
            stake: 1,
            reveal_window: 1,
            daa: 0,
            bundle_cap: 1,
        };
        assert!(Table::open(spec).is_ok());
        assert_eq!(
            Table::open(OpenSpec {
                network: Network::Mainnet,
                ..spec
            })
            .unwrap_err(),
            OpenError::WrongNetwork
        );
        assert_eq!(
            Table::open(OpenSpec {
                network: Network::Simnet,
                ..spec
            })
            .unwrap_err(),
            OpenError::WrongNetwork
        );
        assert_eq!(
            Table::open(OpenSpec { stake: 0, ..spec }).unwrap_err(),
            OpenError::ZeroStake
        );
        assert_eq!(
            Table::open(OpenSpec {
                reveal_window: 0,
                ..spec
            })
            .unwrap_err(),
            OpenError::ZeroWindow
        );
        assert_eq!(
            Table::open(OpenSpec {
                bundle_cap: 0,
                ..spec
            })
            .unwrap_err(),
            OpenError::ZeroCap
        );
        assert_eq!(
            Table::open(OpenSpec {
                stake: u64::MAX,
                ..spec
            })
            .unwrap_err(),
            OpenError::StakeOverflow
        );
    }

    #[test]
    fn l1_accept_does_not_seal_and_a_rejected_guest_write_is_consumed() {
        let mut table = open_at(100);
        let step = commit(1, Seat::Creator, Face::Stone, salt(1), 100);
        table.note_l1(step).unwrap();
        assert!(!table.sealed(Seat::Creator));
        assert_eq!(table.note_l1(step).unwrap_err(), GateError::AlreadyOnL1);
        table.execute(StepId(1)).unwrap();
        assert!(table.sealed(Seat::Creator));
        assert_eq!(
            table.execute(StepId(1)).unwrap_err(),
            ExecError::AlreadyExecuted
        );
        assert_eq!(table.execute(StepId(9)).unwrap_err(), ExecError::NotOnL1);

        let bad = reveal(2, Seat::Creator, Face::Cloth, salt(1), 100);
        table.note_l1(bad).unwrap();
        assert_eq!(table.execute(StepId(2)).unwrap_err(), ExecError::SealClosed);
        assert_eq!(
            table.execute(StepId(2)).unwrap_err(),
            ExecError::AlreadyExecuted
        );
        assert!(table.revealed(Seat::Creator).is_none());
    }

    #[test]
    fn bad_seal_keeps_the_commit_and_a_new_step_can_reveal() {
        let mut table = open_at(10);
        both_sealed(&mut table, Face::Stone, Face::Blade, 10);
        let bad = reveal(3, Seat::Creator, Face::Stone, salt(9), 10);
        table.note_l1(bad).unwrap();
        assert_eq!(table.execute(StepId(3)).unwrap_err(), ExecError::BadSeal);
        assert!(table.revealed(Seat::Creator).is_none());
        note_exec(
            &mut table,
            reveal(4, Seat::Creator, Face::Stone, salt(1), 11),
        );
        assert_eq!(table.revealed(Seat::Creator), Some(Face::Stone));
        let again = reveal(5, Seat::Creator, Face::Stone, salt(1), 11);
        table.note_l1(again).unwrap();
        assert_eq!(
            table.execute(StepId(5)).unwrap_err(),
            ExecError::SecondReveal
        );
    }

    #[test]
    fn second_commit_does_not_replace_the_first() {
        let mut table = open_at(1);
        note_exec(
            &mut table,
            commit(1, Seat::Creator, Face::Stone, salt(1), 1),
        );
        let second = commit(2, Seat::Creator, Face::Blade, salt(3), 1);
        table.note_l1(second).unwrap();
        assert_eq!(table.execute(StepId(2)).unwrap_err(), ExecError::SeatTaken);
        note_exec(&mut table, commit(3, Seat::Joiner, Face::Blade, salt(2), 1));
        note_exec(
            &mut table,
            reveal(4, Seat::Creator, Face::Stone, salt(1), 1),
        );
        assert_eq!(table.revealed(Seat::Creator), Some(Face::Stone));
    }

    #[test]
    fn reveal_window_bounds_are_inclusive() {
        let mut table = open_at(50);
        both_sealed(&mut table, Face::Stone, Face::Cloth, 50);
        let early = reveal(3, Seat::Creator, Face::Stone, salt(1), 49);
        table.note_l1(early).unwrap();
        assert_eq!(table.execute(StepId(3)).unwrap_err(), ExecError::DaaRewind);

        // The floor sits on the second seal, so a reveal dated earlier is a rewind.
        // TooEarly is the guest check used when the window opens ahead of the floor.
        let mut table = open_at(50);
        note_exec(
            &mut table,
            commit(1, Seat::Creator, Face::Stone, salt(1), 50),
        );
        note_exec(
            &mut table,
            commit(2, Seat::Joiner, Face::Cloth, salt(2), 60),
        );
        table.daa = 60;
        table.opened_daa = Some(62);
        let too_soon = reveal(3, Seat::Creator, Face::Stone, salt(1), 61);
        table.note_l1(too_soon).unwrap();
        assert_eq!(table.execute(StepId(3)).unwrap_err(), ExecError::TooEarly);

        let mut table = open_at(50);
        note_exec(
            &mut table,
            commit(1, Seat::Creator, Face::Stone, salt(1), 50),
        );
        note_exec(
            &mut table,
            commit(2, Seat::Joiner, Face::Cloth, salt(2), 60),
        );
        note_exec(
            &mut table,
            reveal(3, Seat::Joiner, Face::Cloth, salt(2), 70),
        );
        note_exec(
            &mut table,
            reveal(4, Seat::Creator, Face::Stone, salt(1), 70),
        );
        assert_eq!(table.revealed(Seat::Joiner), Some(Face::Cloth));
        assert_eq!(table.revealed(Seat::Creator), Some(Face::Stone));
        let late = reveal(5, Seat::Joiner, Face::Cloth, salt(2), 71);
        table.note_l1(late).unwrap();
        assert_eq!(table.execute(StepId(5)).unwrap_err(), ExecError::TooLate);
    }

    #[test]
    fn every_face_pair_matches_the_cycle() {
        let faces = [Face::Stone, Face::Cloth, Face::Blade];
        for creator in faces {
            for joiner in faces {
                let mut table = open_at(1);
                both_sealed(&mut table, creator, joiner, 1);
                both_revealed(&mut table, creator, joiner, 1);
                let finish = table.close(1).unwrap();
                match judge(creator, joiner) {
                    Some(seat) => {
                        assert_eq!(finish, Finish::Win(seat));
                        let leaves = prize(&table);
                        assert_eq!(leaves, vec![(seat, 100)]);
                    }
                    None => {
                        assert_eq!(finish, Finish::Draw);
                        assert_eq!(prize(&table), vec![(Seat::Creator, 50), (Seat::Joiner, 50)]);
                    }
                }
            }
        }
    }

    fn prize(table: &Table) -> Vec<(Seat, u64)> {
        table.lane.retained[0]
            .leaves
            .iter()
            .map(|leaf| (leaf.seat, leaf.sompi))
            .collect()
    }

    #[test]
    fn forfeit_and_return_wait_until_the_window_shuts() {
        let mut table = open_at(1);
        both_sealed(&mut table, Face::Stone, Face::Blade, 1);
        note_exec(
            &mut table,
            reveal(3, Seat::Creator, Face::Stone, salt(1), 1),
        );
        assert_eq!(table.close(11).unwrap_err(), CloseError::WindowOpen);
        assert_eq!(table.close(12).unwrap(), Finish::Forfeit(Seat::Creator));
        assert_eq!(prize(&table), vec![(Seat::Creator, 100)]);

        let mut table = open_at(1);
        both_sealed(&mut table, Face::Stone, Face::Blade, 1);
        note_exec(&mut table, reveal(3, Seat::Joiner, Face::Blade, salt(2), 5));
        assert_eq!(table.close(12).unwrap(), Finish::Forfeit(Seat::Joiner));

        let mut table = open_at(1);
        both_sealed(&mut table, Face::Stone, Face::Blade, 1);
        assert_eq!(table.close(11).unwrap_err(), CloseError::WindowOpen);
        assert_eq!(table.close(12).unwrap(), Finish::Return);
        assert_eq!(prize(&table), vec![(Seat::Creator, 50), (Seat::Joiner, 50)]);
        assert_eq!(table.close(13).unwrap_err(), CloseError::AlreadyClosed);
    }

    #[test]
    fn close_before_both_seals_stays_open() {
        let mut table = open_at(1);
        note_exec(
            &mut table,
            commit(1, Seat::Creator, Face::Stone, salt(1), 1),
        );
        assert_eq!(table.close(100).unwrap_err(), CloseError::SealsOpen);
        assert!(table.lane.retained.is_empty());
    }

    #[test]
    fn daa_rewind_does_not_move_the_floor() {
        let mut table = open_at(20);
        let old = commit(1, Seat::Creator, Face::Stone, salt(1), 19);
        table.note_l1(old).unwrap();
        assert_eq!(table.execute(StepId(1)).unwrap_err(), ExecError::DaaRewind);
        assert!(!table.sealed(Seat::Creator));
        note_exec(
            &mut table,
            commit(2, Seat::Creator, Face::Stone, salt(1), 20),
        );
        assert!(table.sealed(Seat::Creator));
    }

    #[test]
    fn quiet_lane_early_return_strands_and_fall_through_pays_once() {
        let report = script(1_000).unwrap();
        assert_eq!(report.paid, 200_000_000);
        assert_eq!(report.pool_before, 260_000_000);
        assert_eq!(report.pool_after, 60_000_000);
        assert_eq!(report.published, 1);
        assert_eq!(report.pool_before, report.pool_after + report.paid);
        let text = report.lines.join("\n");
        assert!(text.contains("kaspa-testnet-10"));
        assert!(text.contains("The seats were still empty."));
        assert!(text.contains("Stone breaks Blade."));
        assert!(text.contains("The claim is stranded."));
        assert!(text.contains("same front is guarded"));
        assert!(text.contains("Master f9b84a8 strands that unmatched leaf."));
        assert!(text.contains("parent fbd677c2 publishes it."));
        assert!(text.contains("An absent settlement moves nothing."));
    }

    #[test]
    fn script_accepts_a_high_testnet_daa() {
        let report = script(585_423_349).unwrap();
        assert_eq!(report.paid, 200_000_000);
        assert!(report.lines.iter().any(|line| line == "daa 585423349"));
    }

    #[test]
    fn select_delegates_matches_the_driver_stop() {
        let coins = [
            Coin { id: 1, sompi: 5 },
            Coin { id: 2, sompi: 5 },
            Coin { id: 3, sompi: 5 },
        ];
        assert_eq!(
            select_delegates(&coins, 6)
                .into_iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            select_delegates(&coins, 5)
                .into_iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert!(select_delegates(&coins, 0).is_empty());
        let mixed = [
            Coin { id: 7, sompi: 3 },
            Coin { id: 8, sompi: 0 },
            Coin { id: 9, sompi: 2 },
        ];
        assert_eq!(
            select_delegates(&mixed, 5)
                .into_iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![7, 9]
        );
        assert_eq!(
            select_delegates(&mixed, 4)
                .into_iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![7, 9]
        );
    }

    #[test]
    fn claim_conserves_and_refuses_the_short_and_zero_and_overflow_pools() {
        let mut lane = Lane::new(4).unwrap();
        lane.push_retained(Batch {
            index: 1,
            has_txs: true,
            leaves: vec![
                Leaf {
                    id: 1,
                    seat: Seat::Creator,
                    sompi: 10,
                    born_daa: 0,
                },
                Leaf {
                    id: 2,
                    seat: Seat::Joiner,
                    sompi: 0,
                    born_daa: 0,
                },
                Leaf {
                    id: 3,
                    seat: Seat::Creator,
                    sompi: u64::MAX,
                    born_daa: 0,
                },
            ],
        });
        assert_eq!(
            lane.wake(
                Policy::FallThrough,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: None
                },
                Prove::Emitted
            )
            .unwrap()
            .reform,
            Reform::Published(3)
        );

        let mut table = table_with_lane(lane);
        table.deposit_delegate(4);
        table.deposit_delegate(7);
        table.deposit_delegate(3);
        let before = table.pool().to_vec();
        assert_eq!(
            table
                .claim(Claim {
                    seat: Seat::Joiner,
                    leaf: 1,
                })
                .unwrap_err(),
            ClaimError::WrongSeat
        );
        assert_eq!(table.pool(), before.as_slice());
        let paid = table
            .claim(Claim {
                seat: Seat::Creator,
                leaf: 1,
            })
            .unwrap();
        assert_eq!(paid.spent, vec![2, 1]);
        assert_eq!(paid.change, Some(Coin { id: 4, sompi: 1 }));
        assert_eq!(table.pool_total() + paid.paid, 14);
        assert_eq!(
            table
                .claim(Claim {
                    seat: Seat::Creator,
                    leaf: 1,
                })
                .unwrap_err(),
            ClaimError::AlreadyClaimed
        );
        assert_eq!(
            table
                .claim(Claim {
                    seat: Seat::Joiner,
                    leaf: 2,
                })
                .unwrap_err(),
            ClaimError::ZeroLeaf
        );
        assert_eq!(table.pool_total(), 4);

        let mut table = table_with_lane(published_leaf(1, 20));
        table.deposit_delegate(5);
        table.deposit_delegate(5);
        table.deposit_delegate(5);
        let pool = table.pool().to_vec();
        assert_eq!(
            table
                .claim(Claim {
                    seat: Seat::Creator,
                    leaf: 1,
                })
                .unwrap_err(),
            ClaimError::ShortPool
        );
        assert_eq!(table.pool(), pool.as_slice());

        let mut table = table_with_lane(published_leaf(1, u64::MAX));
        table.deposit_delegate(u64::MAX - 1);
        table.deposit_delegate(2);
        let pool = table.pool().to_vec();
        assert_eq!(
            table
                .claim(Claim {
                    seat: Seat::Creator,
                    leaf: 1,
                })
                .unwrap_err(),
            ClaimError::Overflow
        );
        assert_eq!(table.pool(), pool.as_slice());
        assert!(table.claimed.is_empty());
    }

    fn published_leaf(id: u64, sompi: u64) -> Lane {
        let mut lane = Lane::new(2).unwrap();
        lane.push_retained(Batch {
            index: 1,
            has_txs: true,
            leaves: vec![Leaf {
                id,
                seat: Seat::Creator,
                sompi,
                born_daa: 0,
            }],
        });
        lane.wake(
            Policy::FallThrough,
            Boundary {
                queued_prefix: None,
                retained_prefix: None,
            },
            Prove::Emitted,
        )
        .unwrap();
        lane
    }

    fn table_with_lane(lane: Lane) -> Table {
        Table {
            stake: 1,
            pot: 2,
            reveal_window: 1,
            daa: 0,
            opened_daa: None,
            seats: [
                SeatState {
                    digest: None,
                    face: None,
                },
                SeatState {
                    digest: None,
                    face: None,
                },
            ],
            noted: BTreeMap::new(),
            finish: None,
            next_leaf: 10,
            next_coin: 1,
            pool: Vec::new(),
            claimed: BTreeSet::new(),
            lane,
        }
    }

    #[test]
    fn fund_sweep_conserves_every_small_pool() {
        let amounts = [0u64, 1, 5, 9];
        let mut checks = 0u32;
        for a in amounts {
            for b in amounts {
                for c in amounts {
                    for leaf in 0..21u64 {
                        checks += 1;
                        let coins = [
                            Coin { id: 1, sompi: a },
                            Coin { id: 2, sompi: b },
                            Coin { id: 3, sompi: c },
                        ];
                        let selected = select_delegates(&coins, leaf);
                        if leaf == 0 {
                            assert!(selected.is_empty());
                            continue;
                        }
                        let mut funded = 0u64;
                        for coin in &selected {
                            assert!(funded < leaf);
                            funded = funded.saturating_add(coin.sompi);
                        }
                        if selected.is_empty() {
                            continue;
                        }
                        let covered_without_last = selected[..selected.len() - 1]
                            .iter()
                            .fold(0u64, |sum, coin| sum.saturating_add(coin.sompi));
                        assert!(covered_without_last < leaf);
                    }
                }
            }
        }
        assert_eq!(checks, 4 * 4 * 4 * 21);
    }

    #[test]
    fn fund_sweep_pays_or_restores_two_thousand_pools() {
        let mut state = 0x51f1u64;
        for _ in 0..2_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let n = (state % 5) + 1;
            let mut coins = Vec::new();
            for i in 0..n {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                coins.push(Coin {
                    id: i + 1,
                    sompi: state % 40,
                });
            }
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let leaf_sompi = (state % 80) + 1;
            let mut lane = Lane::new(1).unwrap();
            lane.push_retained(Batch {
                index: 1,
                has_txs: true,
                leaves: vec![Leaf {
                    id: 1,
                    seat: Seat::Creator,
                    sompi: leaf_sompi,
                    born_daa: 0,
                }],
            });
            lane.wake(
                Policy::FallThrough,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: None,
                },
                Prove::Emitted,
            )
            .unwrap();
            let mut table = table_with_lane(lane);
            let mut before = 0u64;
            for coin in &coins {
                table.deposit_delegate(coin.sompi);
                before = before.saturating_add(coin.sompi);
            }
            let snapshot = table.pool().to_vec();
            match table.claim(Claim {
                seat: Seat::Creator,
                leaf: 1,
            }) {
                Ok(payout) => {
                    assert_eq!(payout.paid, leaf_sompi);
                    assert_eq!(table.pool_total() + payout.paid, before);
                    let spent: u64 = payout
                        .spent
                        .iter()
                        .map(|id| snapshot.iter().find(|coin| coin.id == *id).unwrap().sompi)
                        .fold(0u64, |sum, sompi| sum.checked_add(sompi).unwrap());
                    assert!(spent >= leaf_sompi);
                    assert_eq!(
                        spent - leaf_sompi,
                        payout.change.map(|coin| coin.sompi).unwrap_or(0)
                    );
                }
                Err(ClaimError::ShortPool) | Err(ClaimError::Overflow) => {
                    assert_eq!(table.pool(), snapshot.as_slice());
                }
                Err(other) => panic!("unexpected {other:?}"),
            }
        }
    }

    #[test]
    fn unmatched_early_return_keeps_the_suffix_and_fall_through_publishes_it() {
        let mut lane = sample_lane(1);
        let quiet = Boundary {
            queued_prefix: None,
            retained_prefix: None,
        };
        let early = lane
            .wake(Policy::EarlyReturn, quiet, Prove::Emitted)
            .unwrap();
        assert_eq!(early.reform, Reform::EarlyReturn);
        assert!(lane.published().is_empty());
        assert_eq!(lane.retained_len(), 1);
        assert_eq!(lane.last_reformed_from(), None);
        let woke = lane
            .wake(Policy::FallThrough, quiet, Prove::Emitted)
            .unwrap();
        assert_eq!(woke.reform, Reform::Published(1));
        assert_eq!(lane.last_reformed_from(), Some(7));
        let again = lane
            .wake(Policy::FallThrough, quiet, Prove::Emitted)
            .unwrap();
        assert_eq!(again.reform, Reform::Guarded);
        assert_eq!(lane.published().len(), 1);
        assert_eq!(lane.retained_len(), 1);
    }

    #[test]
    fn a_matched_boundary_still_reforms_under_the_early_policy() {
        let mut lane = Lane::new(4).unwrap();
        lane.push_queued(Batch {
            index: 1,
            has_txs: true,
            leaves: vec![],
        });
        lane.push_queued(Batch {
            index: 2,
            has_txs: true,
            leaves: vec![],
        });
        lane.push_retained(Batch {
            index: 3,
            has_txs: true,
            leaves: vec![Leaf {
                id: 1,
                seat: Seat::Creator,
                sompi: 5,
                born_daa: 1,
            }],
        });
        let report = lane
            .wake(
                Policy::EarlyReturn,
                Boundary {
                    queued_prefix: Some(1),
                    retained_prefix: None,
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(report.dropped_queued, 1);
        assert_eq!(report.reform, Reform::Published(1));
        assert_eq!(lane.queued_len(), 1);
        assert_eq!(lane.retained_len(), 1);
    }

    #[test]
    fn bundle_cap_stops_at_the_front_until_that_front_is_drained() {
        let mut lane = Lane::new(1).unwrap();
        for index in 1..=3 {
            lane.push_retained(Batch {
                index,
                has_txs: true,
                leaves: vec![Leaf {
                    id: index,
                    seat: Seat::Creator,
                    sompi: index,
                    born_daa: 0,
                }],
            });
        }
        let quiet = Boundary {
            queued_prefix: None,
            retained_prefix: None,
        };
        assert_eq!(
            lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
                .unwrap()
                .reform,
            Reform::Published(1)
        );
        assert_eq!(lane.published()[0].id, 1);
        assert_eq!(
            lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
                .unwrap()
                .reform,
            Reform::Guarded
        );
        assert_eq!(lane.published().len(), 1);
        let report = lane
            .wake(
                Policy::FallThrough,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: Some(1),
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(report.dropped_retained, 1);
        assert_eq!(report.reform, Reform::Published(1));
        assert_eq!(lane.published().len(), 2);
        assert_eq!(lane.published()[1].id, 2);
        assert_eq!(lane.retained_len(), 2);
        assert_eq!(lane.last_reformed_from(), Some(2));
    }

    #[test]
    fn empty_suffix_clears_the_guard_and_a_later_batch_can_form() {
        let mut lane = sample_lane(4);
        let quiet = Boundary {
            queued_prefix: None,
            retained_prefix: None,
        };
        lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
            .unwrap();
        let cleared = lane
            .wake(
                Policy::FallThrough,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: Some(1),
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(cleared.reform, Reform::EmptySuffix);
        assert_eq!(lane.last_reformed_from(), None);
        lane.push_retained(Batch {
            index: 9,
            has_txs: true,
            leaves: vec![Leaf {
                id: 9,
                seat: Seat::Joiner,
                sompi: 1,
                born_daa: 0,
            }],
        });
        assert_eq!(
            lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
                .unwrap()
                .reform,
            Reform::Published(1)
        );
        assert_eq!(lane.last_reformed_from(), Some(9));
    }

    #[test]
    fn deferred_proof_sets_the_guard_and_rollback_lets_the_next_wake_emit() {
        let mut lane = sample_lane(4);
        let quiet = Boundary {
            queued_prefix: None,
            retained_prefix: None,
        };
        let deferred = lane
            .wake(Policy::FallThrough, quiet, Prove::Deferred)
            .unwrap();
        assert_eq!(deferred.reform, Reform::Deferred);
        assert!(lane.published().is_empty());
        assert_eq!(lane.last_reformed_from(), Some(7));
        assert_eq!(
            lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
                .unwrap()
                .reform,
            Reform::Guarded
        );
        lane.rollback(7);
        assert_eq!(lane.last_reformed_from(), None);
        assert_eq!(lane.retained_len(), 1);
        assert_eq!(
            lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
                .unwrap()
                .reform,
            Reform::Published(1)
        );
        assert_eq!(lane.published().len(), 1);
    }

    #[test]
    fn rollback_drops_indexes_above_the_target_and_a_republish_pays_once() {
        let mut lane = Lane::new(4).unwrap();
        lane.push_queued(Batch {
            index: 8,
            has_txs: true,
            leaves: vec![],
        });
        lane.push_retained(Batch {
            index: 3,
            has_txs: true,
            leaves: vec![Leaf {
                id: 1,
                seat: Seat::Creator,
                sompi: 8,
                born_daa: 0,
            }],
        });
        lane.push_retained(Batch {
            index: 8,
            has_txs: true,
            leaves: vec![Leaf {
                id: 2,
                seat: Seat::Joiner,
                sompi: 8,
                born_daa: 0,
            }],
        });
        lane.rollback(3);
        assert_eq!(lane.queued_len(), 0);
        assert_eq!(lane.retained_len(), 1);
        assert_eq!(lane.retained.front().unwrap().index, 3);
        let quiet = Boundary {
            queued_prefix: None,
            retained_prefix: None,
        };
        lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
            .unwrap();
        lane.rollback(3);
        lane.wake(Policy::FallThrough, quiet, Prove::Emitted)
            .unwrap();
        assert_eq!(lane.published().len(), 2);
        assert!(lane.published().iter().all(|leaf| leaf.id == 1));

        let mut table = table_with_lane(lane);
        table.deposit_delegate(8);
        let payout = table
            .claim(Claim {
                seat: Seat::Creator,
                leaf: 1,
            })
            .unwrap();
        assert_eq!(payout.paid, 8);
        assert_eq!(
            table
                .claim(Claim {
                    seat: Seat::Creator,
                    leaf: 1,
                })
                .unwrap_err(),
            ClaimError::AlreadyClaimed
        );
        assert_eq!(table.pool_total(), 0);
    }

    #[test]
    fn an_empty_batch_reforms_as_a_noop() {
        let mut lane = Lane::new(2).unwrap();
        lane.push_retained(Batch {
            index: 1,
            has_txs: false,
            leaves: vec![],
        });
        let report = lane
            .wake(
                Policy::FallThrough,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: None,
                },
                Prove::Deferred,
            )
            .unwrap();
        assert_eq!(report.reform, Reform::Noop);
        assert_eq!(lane.last_reformed_from(), Some(1));
        assert!(lane.published().is_empty());
    }

    #[test]
    fn a_prefix_past_the_window_changes_nothing() {
        let mut lane = sample_lane(4);
        let err = lane
            .wake(
                Policy::FallThrough,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: Some(2),
                },
                Prove::Emitted,
            )
            .unwrap_err();
        assert_eq!(err, WakeError::BadPrefix);
        assert_eq!(lane.retained_len(), 1);
        assert!(lane.published().is_empty());
        assert_eq!(lane.last_reformed_from(), None);
    }

    #[test]
    fn master_ignores_the_queued_window_and_a_deferred_proof() {
        let mut lane = Lane::new(4).unwrap();
        lane.push_queued(Batch {
            index: 1,
            has_txs: true,
            leaves: vec![],
        });
        lane.push_retained(Batch {
            index: 4,
            has_txs: true,
            leaves: vec![Leaf {
                id: 1,
                seat: Seat::Creator,
                sompi: 9,
                born_daa: 0,
            }],
        });
        lane.push_retained(Batch {
            index: 5,
            has_txs: false,
            leaves: vec![],
        });
        let queued_only = Boundary {
            queued_prefix: Some(99),
            retained_prefix: None,
        };
        let missed = lane
            .wake(Policy::Master, queued_only, Prove::Emitted)
            .unwrap();
        assert_eq!(missed.reform, Reform::EarlyReturn);
        assert_eq!(lane.queued_len(), 1);
        assert_eq!(lane.retained_len(), 2);
        assert!(lane.published().is_empty());
        assert_eq!(lane.last_reformed_from(), None);

        let mut parent = lane.clone();
        let published = parent
            .wake(
                Policy::EarlyReturn,
                Boundary {
                    queued_prefix: Some(1),
                    retained_prefix: None,
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(published.dropped_queued, 1);
        assert_eq!(published.reform, Reform::Published(1));
        assert_eq!(parent.queued_len(), 0);
        assert_eq!(parent.published().len(), 1);

        let before = lane.clone();
        assert_eq!(
            lane.wake(
                Policy::Master,
                Boundary {
                    queued_prefix: Some(1),
                    retained_prefix: Some(1),
                },
                Prove::Deferred,
            )
            .unwrap_err(),
            WakeError::NoDeferral
        );
        assert_eq!(lane, before);

        let formed = lane
            .wake(
                Policy::Master,
                Boundary {
                    queued_prefix: Some(99),
                    retained_prefix: Some(1),
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(formed.dropped_queued, 0);
        assert_eq!(formed.dropped_retained, 1);
        assert_eq!(formed.reform, Reform::Noop);
        assert_eq!(lane.queued_len(), 1);
        assert_eq!(lane.retained_len(), 1);
        assert_eq!(lane.last_reformed_from(), Some(5));
        assert!(lane.published().is_empty());

        let again = lane
            .wake(Policy::Master, queued_only, Prove::Emitted)
            .unwrap();
        assert_eq!(again.reform, Reform::EarlyReturn);
        assert_eq!(lane.last_reformed_from(), Some(5));
        assert_eq!(lane.queued_len(), 1);

        let absent = lane.settlement_absent();
        assert_eq!(absent.reform, Reform::Absent);
        assert_eq!(lane.last_reformed_from(), Some(5));
        assert_eq!(lane.retained_len(), 1);

        let cleared = lane
            .wake(
                Policy::Master,
                Boundary {
                    queued_prefix: None,
                    retained_prefix: Some(1),
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(cleared.dropped_queued, 0);
        assert_eq!(cleared.reform, Reform::EmptySuffix);
        assert_eq!(lane.last_reformed_from(), None);
        assert_eq!(lane.queued_len(), 1);
        assert_eq!(lane.retained_len(), 0);
    }

    #[test]
    fn master_rejects_a_retained_prefix_past_the_window() {
        let mut lane = sample_lane(4);
        let before = lane.clone();
        assert_eq!(
            lane.wake(
                Policy::Master,
                Boundary {
                    queued_prefix: Some(3),
                    retained_prefix: Some(2),
                },
                Prove::Emitted,
            )
            .unwrap_err(),
            WakeError::BadPrefix
        );
        assert_eq!(lane, before);
    }

    #[test]
    fn some_zero_is_an_unmatched_boundary() {
        let mut lane = sample_lane(4);
        let report = lane
            .wake(
                Policy::EarlyReturn,
                Boundary {
                    queued_prefix: Some(0),
                    retained_prefix: Some(0),
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(report.reform, Reform::EarlyReturn);
        assert!(lane.published().is_empty());
        let master = lane
            .wake(
                Policy::Master,
                Boundary {
                    queued_prefix: Some(0),
                    retained_prefix: Some(0),
                },
                Prove::Emitted,
            )
            .unwrap();
        assert_eq!(master.reform, Reform::EarlyReturn);
        assert_eq!(lane.retained_len(), 1);
        assert!(lane.published().is_empty());
    }

    fn sample_lane(cap: usize) -> Lane {
        let mut lane = Lane::new(cap).unwrap();
        lane.push_retained(Batch {
            index: 7,
            has_txs: true,
            leaves: vec![Leaf {
                id: 1,
                seat: Seat::Creator,
                sompi: 4,
                born_daa: 3,
            }],
        });
        lane
    }
}
