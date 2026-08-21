//! Tests for what a study run is made of: the order the cards come in, and
//! what an answer means.

use lokked_lib::core::review::Grade;
use lokked_lib::core::scheduler::{shuffle, Rng};

fn ids(count: usize) -> Vec<String> {
    (0..count).map(|n| format!("card-{n:03}")).collect()
}

// --- генератор -------------------------------------------------------------

#[test]
fn the_same_seed_gives_the_same_numbers() {
    let mut left = Rng::new(42);
    let mut right = Rng::new(42);

    let from_left: Vec<u64> = (0..8).map(|_| left.next_u64()).collect();
    let from_right: Vec<u64> = (0..8).map(|_| right.next_u64()).collect();

    assert_eq!(from_left, from_right);
}

#[test]
fn different_seeds_give_different_numbers() {
    let mut one = Rng::new(1);
    let mut two = Rng::new(2);

    assert_ne!(one.next_u64(), two.next_u64());
}

#[test]
fn a_bound_is_never_reached_and_zero_is_reachable() {
    let mut rng = Rng::new(7);
    let mut seen_zero = false;

    for _ in 0..500 {
        let value = rng.below(3);
        assert!(value < 3);
        seen_zero |= value == 0;
    }

    assert!(seen_zero);
}

#[test]
fn a_bound_of_one_leaves_only_zero() {
    let mut rng = Rng::new(9);

    assert_eq!(rng.below(1), 0);
    assert_eq!(rng.below(0), 0);
}

#[test]
fn the_numbers_spread_over_the_whole_range() {
    // Не проверка качества генератора, а защита от очевидно сломанного:
    // такого, который всегда возвращает одно и то же.
    let mut rng = Rng::new(2026);
    let mut buckets = [0_u32; 4];

    for _ in 0..4000 {
        buckets[rng.below(4) as usize] += 1;
    }

    assert!(
        buckets.iter().all(|count| *count > 800),
        "распределение перекошено: {buckets:?}"
    );
}

// --- перемешивание ---------------------------------------------------------

#[test]
fn shuffling_keeps_every_card_exactly_once() {
    let mut cards = ids(50);
    let before = cards.clone();

    shuffle(&mut cards, 123);

    let mut sorted = cards.clone();
    sorted.sort();
    assert_eq!(sorted, before);
}

#[test]
fn the_same_seed_gives_the_same_order() {
    let mut one = ids(30);
    let mut two = ids(30);

    shuffle(&mut one, 2026);
    shuffle(&mut two, 2026);

    assert_eq!(one, two);
}

#[test]
fn a_different_seed_gives_a_different_order() {
    let mut one = ids(30);
    let mut two = ids(30);

    shuffle(&mut one, 1);
    shuffle(&mut two, 2);

    assert_ne!(one, two);
}

#[test]
fn shuffling_actually_moves_things() {
    let mut cards = ids(30);
    let before = cards.clone();

    shuffle(&mut cards, 5);

    assert_ne!(cards, before);
}

#[test]
fn shuffling_nothing_or_one_card_is_not_an_error() {
    let mut empty: Vec<String> = Vec::new();
    let mut single = ids(1);

    shuffle(&mut empty, 3);
    shuffle(&mut single, 3);

    assert!(empty.is_empty());
    assert_eq!(single, ids(1));
}

// --- оценки ----------------------------------------------------------------

#[test]
fn a_grade_survives_a_round_trip_through_its_slug() {
    for grade in [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy] {
        assert_eq!(Grade::parse(grade.as_str()), Ok(grade));
    }
}

#[test]
fn an_unknown_grade_is_refused() {
    assert!(Grade::parse("наверное").is_err());
}

#[test]
fn only_again_counts_as_not_knowing() {
    // «С трудом» — это всё-таки вспомнил: карточка пойдёт чаще, но ошибкой
    // прогона не считается.
    assert!(!Grade::Again.is_correct());
    assert!(Grade::Hard.is_correct());
    assert!(Grade::Good.is_correct());
    assert!(Grade::Easy.is_correct());
}
