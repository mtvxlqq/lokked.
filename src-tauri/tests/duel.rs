//! Tests for the rules of a duel: who may play, what a run is made of, who
//! won, and what the card-by-card breakdown says.

use lokked_lib::core::duel::{
    breakdown, winners, DuelError, DuelSetup, MAX_DUEL_CARDS, MIN_DUEL_CARDS,
};
use lokked_lib::core::review::Grade;

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn setup(players: &[&str]) -> Result<DuelSetup, DuelError> {
    DuelSetup::new(names(players), 20, 20)
}

// --- кто играет ------------------------------------------------------------

#[test]
fn two_players_are_enough_and_four_are_the_most() {
    assert!(setup(&["Ты", "Артём"]).is_ok());
    assert!(setup(&["Ты", "Артём", "Соня", "Илья"]).is_ok());
    assert_eq!(setup(&["Ты"]), Err(DuelError::TooFewPlayers(1)));
    assert_eq!(
        setup(&["Ты", "Артём", "Соня", "Илья", "Гоша"]),
        Err(DuelError::TooManyPlayers(5))
    );
}

#[test]
fn the_first_player_is_the_one_whose_device_it_is() {
    let duel = setup(&["Ты", "Артём", "Соня"]).unwrap();

    assert!(
        duel.players[0].is_owner,
        "первый игрок — владелец устройства"
    );
    assert!(!duel.players[1].is_owner);
    assert!(!duel.players[2].is_owner);
}

#[test]
fn a_nameless_player_is_refused() {
    assert_eq!(
        DuelSetup::new(names(&["Ты", "   "]), 20, 20),
        Err(DuelError::EmptyName)
    );
}

#[test]
fn a_name_is_trimmed_before_it_is_stored() {
    let duel = DuelSetup::new(names(&["  Ты  ", "Артём"]), 20, 20).unwrap();

    assert_eq!(duel.players[0].name, "Ты");
}

#[test]
fn two_players_cannot_share_a_name() {
    // Иначе итоговую таблицу невозможно читать: кто из двух «Артёмов» выиграл.
    assert_eq!(
        DuelSetup::new(names(&["Артём", "артём"]), 20, 20),
        Err(DuelError::DuplicateName("артём".to_string()))
    );
}

// --- из чего состоит заход -------------------------------------------------

#[test]
fn the_number_of_cards_has_to_be_sensible() {
    assert!(DuelSetup::new(names(&["Ты", "Артём"]), MIN_DUEL_CARDS, 20).is_ok());
    assert!(DuelSetup::new(names(&["Ты", "Артём"]), MAX_DUEL_CARDS, 20).is_ok());
    assert_eq!(
        DuelSetup::new(names(&["Ты", "Артём"]), MIN_DUEL_CARDS - 1, 20),
        Err(DuelError::InvalidCards(MIN_DUEL_CARDS - 1))
    );
    assert_eq!(
        DuelSetup::new(names(&["Ты", "Артём"]), MAX_DUEL_CARDS + 1, 20),
        Err(DuelError::InvalidCards(MAX_DUEL_CARDS + 1))
    );
}

#[test]
fn the_time_per_card_follows_the_blitz_rules() {
    assert!(DuelSetup::new(names(&["Ты", "Артём"]), 20, 4).is_err());
    assert!(DuelSetup::new(names(&["Ты", "Артём"]), 20, 121).is_err());
    assert!(DuelSetup::new(names(&["Ты", "Артём"]), 20, 5).is_ok());
}

// --- кто выиграл -----------------------------------------------------------

#[test]
fn the_highest_score_wins() {
    assert_eq!(winners(&[1240, 980]), vec![0]);
    assert_eq!(winners(&[980, 1240]), vec![1]);
    assert_eq!(winners(&[100, 1240, 980]), vec![1]);
}

#[test]
fn an_equal_score_is_a_draw_between_everyone_who_scored_it() {
    assert_eq!(winners(&[1000, 1000]), vec![0, 1]);
    assert_eq!(winners(&[1000, 500, 1000]), vec![0, 2]);
}

#[test]
fn a_duel_nobody_scored_in_has_no_winner() {
    assert!(winners(&[0, 0]).is_empty());
    assert!(winners(&[]).is_empty());
}

// --- разбор по карточкам ---------------------------------------------------

#[test]
fn the_breakdown_puts_every_player_against_every_card() {
    let cards = names(&["c-1", "c-2"]);
    let answers = [
        (0, 0, Grade::Good),
        (1, 0, Grade::Again),
        (0, 1, Grade::Hard),
        (1, 1, Grade::Easy),
    ];

    let rows = breakdown(&cards, 2, &answers);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].card_id, "c-1");
    assert_eq!(rows[0].answers, vec![Some(Grade::Good), Some(Grade::Again)]);
    assert_eq!(rows[1].answers, vec![Some(Grade::Hard), Some(Grade::Easy)]);
}

#[test]
fn a_card_a_player_never_got_to_is_left_blank() {
    // Дуэль бросили на середине хода второго игрока: у него ответов меньше.
    let cards = names(&["c-1", "c-2"]);
    let answers = [
        (0, 0, Grade::Good),
        (0, 1, Grade::Good),
        (1, 0, Grade::Hard),
    ];

    let rows = breakdown(&cards, 2, &answers);

    assert_eq!(rows[1].answers, vec![Some(Grade::Good), None]);
}

#[test]
fn the_breakdown_of_a_duel_with_no_answers_is_still_the_whole_deck() {
    let rows = breakdown(&names(&["c-1", "c-2"]), 3, &[]);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].answers, vec![None, None, None]);
}
