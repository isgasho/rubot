//! This module contains a bot which simply brute forces every possible action, this bot should only be used for testing.
use crate::Game;

/// A bot which uses brute force to calculate the optimal move
pub struct Bot<T: Game> {
    player: T::Player,
}

impl<T: Game> Bot<T> {
    pub fn new(player: T::Player) -> Self {
        Self {
            player
        }
    }

    pub fn select(&mut self, state: &T, depth: u32) -> Option<T::Action> {
        let (active, actions) = state.actions(&self.player);
        if !active { return None }

        let mut actions = actions.into_iter();

        let mut best = {
            let action = actions.next()?;
            let value = self.minimax(state, &action, depth);
            (action, value)
        };

        for action in actions {
            let new = self.minimax(state, &action, depth);
            if new > best.1 {
                best = (action, new);
            }
        }
        
        Some(best.0)
    }

    fn minimax(&mut self, state: &T, action: &T::Action, depth: u32) -> T::Fitness {
        if depth == 0 {
            state.look_ahead(&action, &self.player)
        }
        else {
            let mut state = state.clone();
            let fitness = state.execute(&action, &self.player);
            let (active, actions) = state.actions(&self.player);
            
            let iter = actions.into_iter().map(|action| {
                self.minimax(&state, &action, depth - 1)
            });

            if active { 
                iter.max()
            } else { 
                iter.min()
            }.unwrap_or(fitness)
        }
    }
}