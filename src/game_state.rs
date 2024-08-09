use std::collections::HashMap;

use leptos::prelude::*;

use crate::{
    cell::{Cell, CELLS_COUNT},
    game_data::init_cells,
    player::{Player, PlayerColor, PlayerId},
    utils::{oneshot_event_emitter::OneShotEventEmitter, rand},
};

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    cells: [Cell; CELLS_COUNT],
    players: RwSignal<HashMap<PlayerId, Player>>,
    pub self_player: Player,
    current_player: RwSignal<Player>,
    current_step: RwSignal<usize>,
    current_round: RwSignal<usize>,
    render_dice: RwSignal<Option<(usize, usize)>>,
    player_token_transition_end: OneShotEventEmitter,
    dice_transition_end: OneShotEventEmitter,
}

impl GameState {
    pub fn new() -> Self {
        let mut players = HashMap::new();
        players.insert(0, Player::new(0, "Ar4ys", PlayerColor::Blue));
        players.insert(1, Player::new(1, "Madeline", PlayerColor::Green));

        Self {
            self_player: players[&0],
            current_player: RwSignal::new(players[&0]),
            current_step: RwSignal::new(0),
            current_round: RwSignal::new(0),
            cells: init_cells(),
            players: RwSignal::new(players),
            render_dice: RwSignal::new(None),
            player_token_transition_end: OneShotEventEmitter::new(),
            dice_transition_end: OneShotEventEmitter::new(),
        }
    }

    pub fn use_context() -> Self {
        expect_context::<Self>()
    }

    pub fn current_player(&self) -> Player {
        self.current_player.get()
    }

    pub fn render_dice(&self) -> Option<(usize, usize)> {
        self.render_dice.get()
    }

    pub fn get_cell(&self, index: usize) -> Cell {
        self.cells[index]
    }

    pub fn get_players(&self) -> HashMap<PlayerId, Player> {
        self.players.get()
    }

    pub fn get_player_by_id(&self, id: PlayerId) -> Player {
        self.players.with_untracked(|players| {
            *players
                .get(&id)
                .unwrap_or_else(|| panic!("Player with id \"{id}\" should to exists"))
        })
    }

    pub fn get_players_by_cell(&self, index: usize) -> Vec<Player> {
        self.players.with(|players| {
            players
                .values()
                .filter_map(|player| (player.position() == index).then_some(*player))
                .collect()
        })
    }

    pub async fn roll_dice(&self) {
        let dice1 = rand::get_usize(1..=6);
        let dice2 = rand::get_usize(1..=6);
        self.render_dice.set(Some((dice1, dice2)));
        self.dice_transition_end.listen_async().await;
        self.current_player
            .with_untracked(|player| player.append_position(dice1 + dice2));

        self.player_token_transition_end.listen_async().await;
        self.render_dice.set(None);
        self.finish_step();
    }

    pub fn finish_step(&self) {
        let is_round_ended = untrack(|| self.next_player());
        self.current_step.update(|step| *step += 1);
        if is_round_ended {
            self.current_round.update(|round| *round += 1);
        }
    }

    pub fn next_player(&self) -> bool {
        let players_left: Vec<_> = self.players.with(|players| {
            players
                .values()
                // Skip all players before current one
                .skip_while(|player| **player != self.current_player())
                // Skip current player
                .skip(1)
                .copied()
                .collect()
        });

        let (next_player, is_round_ended) = if players_left.is_empty() {
            (
                self.players.with(|players| {
                    players
                        .values()
                        .next()
                        .copied()
                        .expect("There should be players!")
                }),
                true,
            )
        } else {
            (players_left[0], false)
        };

        self.current_player.set(next_player);

        is_round_ended
    }

    pub fn player_token_transition_end(&self) {
        self.player_token_transition_end.trigger();
    }
    pub fn dice_transition_end(&self) {
        self.dice_transition_end.trigger();
    }
}
