use std::{collections::HashMap, future::Future};

use futures::{
    stream::{AbortHandle, Abortable},
    FutureExt,
};
use leptos::{prelude::*, spawn::spawn_local};

use crate::{
    cell::{Cell, Property, PropertyGroup, CELLS_COUNT},
    components::in_game_modal::InGameModalState,
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
    current_turn: RwSignal<usize>,
    current_round: RwSignal<usize>,
    rolled_dice: RwSignal<Option<(usize, usize)>>,
    player_token_transition_end: OneShotEventEmitter,
    dice_transition_end: OneShotEventEmitter,
    pub in_game_modal_state: InGameModalState,
    abort_handlers: StoredValue<Vec<AbortHandle>>,
}

impl GameState {
    pub fn new() -> Self {
        let mut players = HashMap::new();
        players.insert(0, Player::new(0, "Ar4ys", PlayerColor::Blue));
        players.insert(1, Player::new(1, "Madeline", PlayerColor::Green));
        players.insert(2, Player::new(2, "Asmodeus", PlayerColor::Red));

        Self {
            self_player: players[&0],
            current_player: RwSignal::new(players[&0]),
            current_turn: RwSignal::new(0),
            current_round: RwSignal::new(0),
            cells: init_cells(),
            players: RwSignal::new(players),
            rolled_dice: RwSignal::new(None),
            player_token_transition_end: OneShotEventEmitter::new(),
            dice_transition_end: OneShotEventEmitter::new(),
            in_game_modal_state: InGameModalState::new(),
            abort_handlers: StoredValue::new(Vec::new()),
        }
    }

    pub fn provide_context(&self) {
        provide_context(*self);
    }

    pub fn use_context() -> Self {
        expect_context::<Self>()
    }

    pub fn current_player(&self) -> Player {
        self.current_player.get()
    }

    pub fn rolled_dice(&self) -> Option<(usize, usize)> {
        self.rolled_dice.get()
    }

    pub fn get_cell(&self, index: usize) -> Cell {
        self.cells[index]
    }

    pub fn current_turn(&self) -> usize {
        self.current_turn.get()
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
        self.players.with_untracked(|players| {
            players
                .values()
                .filter_map(|player| (player.position() == index).then_some(*player))
                .collect()
        })
    }

    // TODO: Better name
    // TODO: Create "leptos_untrack" attribute-macro that will use "SpecialNonReactiveZone" to disable tracking
    pub async fn roll_dice(&self) {
        let dice1 = rand::get_usize(1..=6);
        let dice2 = rand::get_usize(1..=6);
        self.rolled_dice.set(Some((dice1, dice2)));
        self.dice_transition_end.listen_async().await;
        let (prev_position, new_position) = self
            .current_player
            .get_untracked()
            .append_position(dice1 + dice2);
        self.player_token_transition_end.listen_async().await;

        if prev_position + dice1 + dice2 >= CELLS_COUNT {
            self.current_player.get_untracked().deposit(2000.into())
        }

        let current_cell = self.get_cell(new_position);
        current_cell.trigger(self).await;

        self.finish_turn();
    }

    pub fn finish_turn(&self) {
        let is_round_ended = untrack(|| self.next_player());
        self.current_turn.update(|turn| *turn += 1);

        if is_round_ended {
            self.current_round.update(|round| *round += 1);
            for cell in self.cells.iter() {
                if let Cell::Property(property) = cell {
                    property.tick();
                };
            }
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
                .filter(|player| untrack(|| !player.has_lost()))
                .copied()
                .collect()
        });

        let (next_player, is_round_ended) = if players_left.is_empty() {
            (
                self.players.with(|players| {
                    players
                        .values()
                        .find(|player| untrack(|| !player.has_lost()))
                        .copied()
                        .expect("There should be players!")
                }),
                true,
            )
        } else {
            (players_left[0], false)
        };

        self.current_player.set(next_player);
        self.rolled_dice.set(None);

        is_round_ended
    }

    pub fn player_token_transition_end(&self) {
        self.player_token_transition_end.trigger();
    }

    pub fn dice_transition_end(&self) {
        self.dice_transition_end.trigger();
    }

    pub fn get_properties_by_group(&self, property_group: &PropertyGroup) -> Vec<Property> {
        self.cells
            .iter()
            .filter_map(|cell| cell.try_unwrap_property().ok())
            .filter(|prop| prop.data.group == *property_group)
            .collect()
    }

    pub fn has_from_group(
        &self,
        player: &Player,
        property_group: &PropertyGroup,
    ) -> (usize, usize) {
        let prop_groups_iter = self.get_properties_by_group(property_group).into_iter();

        let total = prop_groups_iter.clone().count();
        let owns = prop_groups_iter
            .filter(|prop| prop.owner().is_some_and(|owner| owner == *player))
            .count();

        (owns, total)
    }

    pub fn has_monopoly_on(&self, player: &Player, property_group: &PropertyGroup) -> bool {
        let (owns, total) = self.has_from_group(player, property_group);
        owns == total
    }

    pub fn surrender_player(&self, player: &Player) {
        player.surrender();
        self.cells
            .iter()
            .filter_map(|cell| cell.try_unwrap_property().ok())
            .filter(|p| untrack(|| p.owner()).is_some_and(|owner| owner == *player))
            .for_each(|p| p.remove_owner());

        if self.self_player == *player {
            self.abort_all_tasks();
        }

        if self.current_player.get_untracked() == *player {
            self.finish_turn();
        }
    }

    pub fn spawn_local_abortable(&self, fut: impl Future<Output = ()> + 'static) {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        self.abort_handlers
            .update_value(move |abort_handlers| abort_handlers.push(abort_handle));
        spawn_local(Abortable::new(fut, abort_registration).map(|_| ()));
    }

    fn abort_all_tasks(&self) {
        self.abort_handlers.update_value(|abort_handlers| {
            for handle in abort_handlers.drain(..) {
                handle.abort();
            }
        })
    }
}
