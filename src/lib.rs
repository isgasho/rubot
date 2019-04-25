#![forbid(unsafe_code)]
//! An easily reusable game bot for deterministic games.
//!
//! It is required to implement the trait [`Game`][game] to use this crate.
//! For more details, look at the [trait documentation][game] or visit the [examples directory][ex].
//!
//! While this crate will probably have many different kind of bots in the future, there is currently only one: [`alpha_beta`][ab].
//!
//! This bot uses an optimized version of [alpha beta pruning][ab_wiki] with [iterative deepening][id].
//!
//! [id]:https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search
//! [ab_wiki]:https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
//! [ab]:alpha_beta/struct.Bot.html
//! [ex]:https://github.com/lcnr/rubot/tree/master/examples
//! [game]:trait.Game.html
pub mod alpha_beta;

#[allow(unused)]
#[doc(hidden)]
pub mod brute;
#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::time::{Duration, Instant};

/// An interface required to interact with [`GameBot`s][bot].
///
/// # Examples
///
/// Implementing this trait for `21 flags`. The game has the following rules:
///
/// - at the beginning there are 21 flags.
/// - 2 players draw 1, 2 or 3 flags in alternating turns
/// - the player who removes the last flag wins
///
/// This is example is really simplified and should be viewed as such.
/// For a more realistic example visit the `/examples` folder of this project.
/// ```
/// use std::{
///     ops::RangeInclusive,         
///     time::Duration
/// };
///
/// #[derive(Clone)]
/// struct Game {
///     flags: u32,
///     active_player: Player
/// }
///
/// type Player = bool;
///
/// impl Game {
///     fn remove_flags(&mut self, flags: u32) {
///         self.flags -= flags;
///         self.active_player = !self.active_player;
///     }
///
///     fn winner(&self) -> Player {
///         assert_eq!(self.flags, 0);
///         !self.active_player
///     }
/// }
///
/// impl rubot::Game for Game {
///     type Player = Player;
///     type Action = u32;
///     /// `true` if the player wins the game, `false` otherwise.
///     type Fitness = bool;
///     type Actions = RangeInclusive<u32>;
///     
///     fn actions(&self, player: &Self::Player) -> (bool, Self::Actions) {
///         (*player == self.active_player, 1..=std::cmp::min(self.flags, 3))
///     }
///     
///     fn execute(&mut self, action: &Self::Action, player: &Self::Player) -> Self::Fitness {
///         (action, player, &self);
///         self.remove_flags(*action);
///         self.flags == 0 && *player == self.winner()
///     }
/// }
///
/// fn main() {
///     use rubot::{Bot, ToCompletion};
///     let mut player_a = Bot::new(true);
///     let mut player_b = Bot::new(false);
///     let mut game = Game { flags: 21, active_player: true };
///     loop {
///         game.remove_flags(player_a.select(&game, ToCompletion).unwrap());
///         if game.flags == 0 { break }
///
///         game.remove_flags(player_b.select(&game, ToCompletion).unwrap());
///         if game.flags == 0 { break }
///     }
///     // in case both players play perfectly, the player who begins should always win
///     assert_eq!(game.winner(), true, "players are not playing optimally");
/// }
/// ```
/// [bot]: trait.GameBot.html
/// [act]: trait.Game.html#associatedtype.Action
/// [player]: trait.Game.html#associatedtype.player
pub trait Game: Clone {
    /// the player type
    type Player;
    /// a executable action
    type Action: PartialEq;
    /// the fitness of a state, higher is better
    type Fitness: Ord + Copy;
    /// the collection returned by [`fn actions`][ac]
    ///
    /// [ac]:trait.Game.html#tymethod.actions
    type Actions: IntoIterator<Item = Self::Action>;

    /// Returns all currently possible actions and if they are executed by the given `player`.
    fn actions(&self, player: &Self::Player) -> (bool, Self::Actions);

    /// Execute a given `action`, returning the new `fitness` for the given `player`.
    /// The returned fitness is always from the perspective of `player`,
    /// even if the `player` is not active and another player is doing this.
    ///
    /// A correctly implemented `GameBot` will only call this function with
    /// actions generated by [`fn actions`][actions].
    ///
    /// [undefined behavior]:https://doc.rust-lang.org/beta/reference/behavior-considered-undefined.html
    /// [actions]: trait.Game.html#tymethod.actions
    fn execute(&mut self, action: &Self::Action, player: &Self::Player) -> Self::Fitness;

    /// Returns the fitness after `action` is executed.
    /// The returned fitness is always from the perspective of `player`,
    /// even if the `player` is not active and another player is doing this.
    ///
    /// This function should always return the same [`Fitness`][fit] as calling [`fn execute`][exe].
    ///
    /// ```rust
    /// # // Why am I lying to you :O
    /// # struct Game;
    /// # impl Game {
    /// #   fn look_ahead(&self, action: &(), player: &()) -> u32 { 42 }
    /// #   fn execute(&mut self, action: &(), player: &()) -> u32 { 42 }
    /// # }
    /// # let mut state = Game;
    /// # let (action, player) = ((), &());
    /// let look_ahead = state.look_ahead(&action, player);
    /// let execute = state.execute(&action, player);
    ///
    /// assert_eq!(look_ahead, execute);
    /// ```
    /// [fit]: trait.Game.html#associatedtype.Fitness
    /// [exe]: trait.Game.html#tymethod.execute
    fn look_ahead(&self, action: &Self::Action, player: &Self::Player) -> Self::Fitness {
        self.clone().execute(action, player)
    }
}

/// Converts a type into a [`RunCondition`][rc].
///
/// # Examples
///
/// [`Duration`][dur]
/// ```rust
/// # struct Game;
/// # struct Bot;
/// # impl Bot {
/// #   fn select<U: rubot::IntoRunCondition>(&mut self, state: &Game, condition: U) -> Option<()> {
/// #       Some(())
/// #   }
/// # }
/// use std::time::Duration;
///
/// let available_time = Duration::from_secs(2);
///
/// let game: Game = // ...
/// # Game;
/// let mut bot: Bot = // ...
/// # Bot;
/// assert!(bot.select(&game, available_time).is_some())
/// ```
///
///
/// [rc]: trait.RunCondition.html
/// [dur]: https://doc.rust-lang.org/std/time/struct.Duration.html
pub trait IntoRunCondition {
    type RunCondition: RunCondition;

    /// consumes `self` and returns a [`RunCondition`][rc].
    ///
    /// [rc]: trait.RunCondition.html
    fn into_run_condition(self) -> Self::RunCondition;
}

impl<T> IntoRunCondition for T
where
    T: RunCondition,
{
    type RunCondition = Self;

    fn into_run_condition(self) -> Self {
        self
    }
}

/// Can be converted into [`RunCondition`][rc] which returns `true` for the first `self.0` steps.
///
/// [rc]: trait.RunCondition.html
#[derive(Clone, Copy, Debug)]
pub struct Steps(pub u32);

/// The [`RunCondition`][rc] created by `fn `[`Steps`][steps]`::into_run_condition`
///
/// [rc]: trait.RunCondition.html
/// [steps]: struct.Steps.html
#[doc(hidden)]
pub struct InnerSteps(u32, u32);

impl IntoRunCondition for Steps {
    type RunCondition = InnerSteps;

    fn into_run_condition(self) -> InnerSteps {
        InnerSteps(0, self.0)
    }
}

impl RunCondition for InnerSteps {
    #[inline]
    fn step(&mut self) -> bool {
        self.0 += 1;
        self.0 < self.1
    }

    #[inline]
    fn depth(&mut self, _: u32) -> bool {
        true
    }
}

/// Creates a [`RunCondition`][rc] which returns `true` until this `Duration` has passed.
///
/// [rc]: trait.RunCondition.html
impl IntoRunCondition for Duration {
    type RunCondition = Instant;

    fn into_run_condition(self) -> Instant {
        Instant::now() + self
    }
}

/// A condition which indicates if the [`Bot`][bot] should keep on running.
///
/// # Examples
///
/// ```rust
/// use std::time::Duration;
///
///
/// ```
///
/// [bot]: alpha_beta/struct.Bot.html
pub trait RunCondition {
    fn step(&mut self) -> bool;
    fn depth(&mut self, depth: u32) -> bool;
}

/// Returns `true` while the `Instant` is still in the future
impl RunCondition for Instant {
    #[inline]
    fn step(&mut self) -> bool {
        Instant::now() < *self
    }

    #[inline]
    fn depth(&mut self, _: u32) -> bool {
        Instant::now() < *self
    }
}

/// A struct implementing [`RunCondition`][rc] which always returns `true`.
///
/// This means that the bot will run until the best action was found with certainty.
///
/// [rc]: trait.RunCondition.html
#[derive(Clone, Copy, Debug)]
pub struct ToCompletion;

impl RunCondition for ToCompletion {
    #[inline]
    fn step(&mut self) -> bool {
        true
    }

    #[inline]
    fn depth(&mut self, _: u32) -> bool {
        true
    }
}

/// A struct implementing [`RunCondition`][rc] returning `false` once the current depth is bigger than `self.0`.
///
/// [rc]: trait.RunCondition.html
#[derive(Clone, Copy, Debug)]
pub struct Depth(pub u32);

impl RunCondition for Depth {
    #[inline]
    fn step(&mut self) -> bool {
        true
    }

    #[inline]
    fn depth(&mut self, depth: u32) -> bool {
        self.0 > depth
    }
}

pub use alpha_beta::Bot;
