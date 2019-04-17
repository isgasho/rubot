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
mod brute;
#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::time::Duration;

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
///     use rubot::{Bot, GameBot};
///     let mut player_a = Bot::new(true);
///     let mut player_b = Bot::new(false);
///     let mut game = Game { flags: 21, active_player: true };
///     loop {
///         game.remove_flags(player_a.select(&game, Duration::from_secs(2)).unwrap());
///         if game.flags == 0 { break }
///
///         game.remove_flags(player_b.select(&game, Duration::from_secs(2)).unwrap());
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
    type Player;
    type Action: PartialEq;
    type Fitness: Ord + Copy;
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
    /// # // this is full of lies, please forgive me senpai-kun
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

/// An interface for dealing with game bots. To learn how the bots currently work, please visit the individual implementations.
pub trait GameBot<T: Game> {
    /// Returns a chosen action based on the given game state.
    ///
    /// In case no `Action` is possible or the bot is currently not the active player, this functions returns `None`.
    /// This method should run at most for a duration which is slightly larger than `duration`.
    fn select(&mut self, state: &T, duration: Duration) -> Option<T::Action>;
}

pub use alpha_beta::Bot;
