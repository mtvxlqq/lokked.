//! Tests for the adaptive picker: what a card's weight is made of, and how
//! the next card is drawn from those weights.
//!
//! The rules being checked are the ones the whole idea rests on — no card
//! ever falls out of rotation, a card just missed comes back soon, and the
//! same seed replays the same run.

use chrono::{DateTime, TimeDelta, Utc};

use lokked_lib::core::review::Grade;
use lokked_lib::core::scheduler::pick::{weighted_order, Picker, Weighted, REPEAT_WINDOW};
use lokked_lib::core::scheduler::weights::{
    weight, Answer, CardHistory, MIN_WEIGHT, NEW_WEIGHT, RECENT_ANSWERS,
};

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// A card answered `grade` `count` times, the last one `minutes_ago`.
fn history(card_id: &str, grade: Grade, count: u32, minutes_ago: i64) -> CardHistory {
    let mut card = CardHistory::new(card_id);

    for step in (0..count).rev() {
        card.answered(Answer {
            at: now() - TimeDelta::minutes(minutes_ago + step as i64 * 60),
            grade,
        });
    }

    card
}

fn weighted(cards: &[(&str, f64)]) -> Vec<Weighted> {
    cards
        .iter()
        .map(|(card_id, weight)| Weighted {
            card_id: (*card_id).to_string(),
            weight: *weight,
        })
        .collect()
}

/// Draws `count` cards, one after another.
fn draw(picker: &mut Picker, count: usize) -> Vec<String> {
    (0..count).filter_map(|_| picker.pick()).collect()
}

// --- вес карточки ----------------------------------------------------------

#[test]
fn a_card_nobody_has_answered_gets_the_middle_weight() {
    // Новая карточка не должна ни вытеснять слабые, ни ждать своей очереди
    // до последнего: её вес — середина, а не максимум.
    let fresh = CardHistory::new("новая");

    assert_eq!(weight(&fresh, now(), 1.0), NEW_WEIGHT);
}

#[test]
fn a_missed_card_outweighs_a_known_one() {
    let missed = history("промах", Grade::Again, 5, 10);
    let known = history("знаю", Grade::Good, 5, 10);

    assert!(weight(&missed, now(), 1.0) > weight(&known, now(), 1.0));
}

#[test]
fn a_card_answered_easily_weighs_less_than_one_answered_with_effort() {
    let easy = history("легко", Grade::Easy, 5, 10);
    let hard = history("с трудом", Grade::Hard, 5, 10);

    assert!(weight(&easy, now(), 1.0) < weight(&hard, now(), 1.0));
}

#[test]
fn a_perfectly_known_card_keeps_a_positive_weight() {
    // Главное требование: пятьдесят безошибочных показов делают вес малым,
    // но не нулевым — карточка не теряется из виду никогда.
    let mastered = history("выучена", Grade::Easy, 50, 10);
    let value = weight(&mastered, now(), 1.0);

    assert!(value > 0.0, "вес выученной карточки упал до нуля");
    assert!(
        value < NEW_WEIGHT / 4.0,
        "выученная карточка весит слишком много: {value}"
    );
}

#[test]
fn even_at_the_highest_aggressiveness_no_weight_reaches_zero() {
    let mastered = history("выучена", Grade::Easy, 50, 10);

    assert!(weight(&mastered, now(), 2.0) >= MIN_WEIGHT);
}

#[test]
fn recent_answers_count_for_more_than_old_ones() {
    // Одна и та же точность за окно, но у первой карточки промахи свежие,
    // а у второй — давно исправленные.
    let mut slipping = CardHistory::new("поехала");
    let mut improving = CardHistory::new("выправилась");

    for step in 0..6 {
        let at = now() - TimeDelta::hours(6 - step);
        let (early, late) = (Grade::Good, Grade::Again);
        slipping.answered(Answer {
            at,
            grade: if step < 3 { early } else { late },
        });
        improving.answered(Answer {
            at,
            grade: if step < 3 { late } else { early },
        });
    }

    assert!(weight(&slipping, now(), 1.0) > weight(&improving, now(), 1.0));
}

#[test]
fn only_the_last_answers_are_looked_at() {
    // Карточка, которую год назад не помнили, а с тех пор отвечали верно,
    // не должна тянуть старые промахи за собой вечно.
    let mut card = CardHistory::new("исправленная");
    for _ in 0..20 {
        card.answered(Answer {
            at: now() - TimeDelta::days(300),
            grade: Grade::Again,
        });
    }
    for step in (0..RECENT_ANSWERS).rev() {
        card.answered(Answer {
            at: now() - TimeDelta::hours(step as i64),
            grade: Grade::Good,
        });
    }

    let clean = history("чистая", Grade::Good, 30, 0);

    assert!((weight(&card, now(), 1.0) - weight(&clean, now(), 1.0)).abs() < 1e-9);
}

#[test]
fn a_card_not_seen_for_a_long_time_gains_weight() {
    let yesterday = history("вчерашняя", Grade::Good, 5, 24 * 60);
    let long_ago = history("забытая", Grade::Good, 5, 60 * 24 * 60);

    assert!(weight(&long_ago, now(), 1.0) > weight(&yesterday, now(), 1.0));
}

#[test]
fn the_last_answer_decides_more_than_the_ones_before_it() {
    // После «не помню» карточка должна вернуться скоро, даже если до этого
    // с ней всё было в порядке.
    let mut lapsed = history("сорвалась", Grade::Good, 8, 60);
    lapsed.answered(Answer {
        at: now() - TimeDelta::minutes(1),
        grade: Grade::Again,
    });
    let steady = history("ровная", Grade::Good, 9, 60);

    assert!(weight(&lapsed, now(), 1.0) > weight(&steady, now(), 1.0) * 3.0);
}

#[test]
fn zero_aggressiveness_makes_every_card_weigh_the_same() {
    let mastered = history("выучена", Grade::Easy, 50, 10);
    let missed = history("промах", Grade::Again, 5, 10);
    let fresh = CardHistory::new("новая");

    for card in [&mastered, &missed, &fresh] {
        assert!((weight(card, now(), 0.0) - 1.0).abs() < 1e-9);
    }
}

#[test]
fn more_aggressiveness_widens_the_gap() {
    let missed = history("промах", Grade::Again, 5, 10);
    let mastered = history("выучена", Grade::Easy, 50, 10);

    let calm = weight(&missed, now(), 0.5) / weight(&mastered, now(), 0.5);
    let sharp = weight(&missed, now(), 2.0) / weight(&mastered, now(), 2.0);

    assert!(
        sharp > calm * 2.0,
        "перекос почти не изменился: {calm} → {sharp}"
    );
}

// --- выбор следующей карточки ----------------------------------------------

#[test]
fn the_same_seed_gives_the_same_run() {
    let cards = weighted(&[("a", 1.0), ("b", 0.2), ("c", 3.0), ("d", 0.7)]);
    let mut one = Picker::new(cards.clone(), REPEAT_WINDOW, 2026);
    let mut two = Picker::new(cards, REPEAT_WINDOW, 2026);

    assert_eq!(draw(&mut one, 40), draw(&mut two, 40));
}

#[test]
fn a_different_seed_gives_a_different_run() {
    let cards: Vec<Weighted> = (0..12)
        .map(|n| Weighted {
            card_id: format!("card-{n:02}"),
            weight: 1.0,
        })
        .collect();
    let mut one = Picker::new(cards.clone(), REPEAT_WINDOW, 1);
    let mut two = Picker::new(cards, REPEAT_WINDOW, 2);

    assert_ne!(draw(&mut one, 40), draw(&mut two, 40));
}

#[test]
fn a_card_never_comes_up_twice_in_a_row() {
    // Даже с весом в сто раз выше остальных.
    let mut picker = Picker::new(
        weighted(&[("тяжёлая", 100.0), ("лёгкая", 1.0), ("ещё", 1.0)]),
        REPEAT_WINDOW,
        7,
    );

    let drawn = draw(&mut picker, 200);

    assert!(
        drawn.windows(2).all(|pair| pair[0] != pair[1]),
        "карточка выпала дважды подряд"
    );
}

#[test]
fn the_last_few_cards_are_kept_out_of_the_draw() {
    // Окно во всю ширину: карточек вдвое больше, чем оно держит.
    let cards: Vec<Weighted> = (0..8)
        .map(|n| Weighted {
            card_id: format!("card-{n}"),
            weight: 1.0,
        })
        .collect();
    let mut picker = Picker::new(cards, REPEAT_WINDOW, 11);

    let drawn = draw(&mut picker, 100);

    for step in REPEAT_WINDOW..drawn.len() {
        let window = &drawn[step - REPEAT_WINDOW..step];
        assert!(
            !window.contains(&drawn[step]),
            "карточка {} повторилась внутри окна",
            drawn[step]
        );
    }
}

#[test]
fn a_deck_of_one_card_keeps_dealing_it() {
    // Окно не может быть шире колоды: иначе выбирать станет не из чего.
    let mut picker = Picker::new(weighted(&[("одна", 1.0)]), REPEAT_WINDOW, 3);

    assert_eq!(draw(&mut picker, 5), vec!["одна"; 5]);
}

#[test]
fn a_deck_of_two_cards_alternates() {
    let mut picker = Picker::new(weighted(&[("a", 5.0), ("b", 0.1)]), REPEAT_WINDOW, 4);

    let drawn = draw(&mut picker, 6);

    assert!(drawn.windows(2).all(|pair| pair[0] != pair[1]), "{drawn:?}");
}

#[test]
fn an_empty_deck_deals_nothing() {
    let mut picker = Picker::new(Vec::new(), REPEAT_WINDOW, 5);

    assert_eq!(picker.pick(), None);
    assert!(picker.is_empty());
}

#[test]
fn heavier_cards_come_up_more_often() {
    // Двести карточек, тысяча показов: у десяти из них вес вчетверо выше,
    // и на них должна прийтись заметно большая доля показов.
    let mut cards: Vec<Weighted> = (0..200)
        .map(|n| Weighted {
            card_id: format!("card-{n:03}"),
            weight: if n < 10 { 4.0 } else { 1.0 },
        })
        .collect();
    cards.shrink_to_fit();
    let mut picker = Picker::new(cards, REPEAT_WINDOW, 2026);

    let drawn = draw(&mut picker, 1000);
    let heavy = drawn
        .iter()
        .filter(|card| card.as_str() < "card-010")
        .count();

    // Ожидание — 4×10 / (4×10 + 190) ≈ 17,4% от тысячи, то есть около 174.
    assert!(
        (120..=230).contains(&heavy),
        "доля тяжёлых карточек не похожа на ожидаемую: {heavy}"
    );
}

#[test]
fn a_mastered_card_still_turns_up_over_a_long_run() {
    // Требование «карточки не теряются из виду»: даже вес в двадцать раз
    // ниже остальных обязан выпасть на длинной дистанции.
    let mut cards: Vec<Weighted> = (0..19)
        .map(|n| Weighted {
            card_id: format!("card-{n:02}"),
            weight: 1.0,
        })
        .collect();
    cards.push(Weighted {
        card_id: "выучена".to_string(),
        weight: 0.08,
    });
    let mut picker = Picker::new(cards, REPEAT_WINDOW, 99);

    let drawn = draw(&mut picker, 1000);

    assert!(
        drawn.iter().any(|card| card == "выучена"),
        "выученная карточка не выпала ни разу за тысячу показов"
    );
}

#[test]
fn a_card_answered_again_comes_back_within_ten() {
    // То, ради чего вес пересчитывается по ходу прогона: «не помню» должно
    // возвращать карточку скоро, а не в следующий заход.
    let cards: Vec<Weighted> = (0..20)
        .map(|n| Weighted {
            card_id: format!("card-{n:02}"),
            weight: 0.4,
        })
        .collect();

    let returned = (0..50)
        .filter(|seed| {
            let mut picker = Picker::new(cards.clone(), REPEAT_WINDOW, *seed);
            let missed = picker.pick().expect("колода не пуста");
            picker.reweigh(&missed, 4.0);

            draw(&mut picker, 10).contains(&missed)
        })
        .count();

    // Не «всегда»: выбор случайный, и гарантировать возврат к сроку можно
    // было бы только очередью, которой здесь нет. Но почти всегда.
    assert!(
        returned >= 40,
        "карточка вернулась за десять показов лишь в {returned} прогонах из пятидесяти"
    );
}

#[test]
fn reweighing_a_card_that_is_not_in_the_deck_changes_nothing() {
    let mut picker = Picker::new(weighted(&[("a", 1.0), ("b", 1.0)]), REPEAT_WINDOW, 8);
    picker.reweigh("чужая", 100.0);

    assert!(draw(&mut picker, 10).iter().all(|card| card != "чужая"));
}

// --- порядок для марафона --------------------------------------------------

#[test]
fn a_weighted_order_keeps_every_card_exactly_once() {
    let cards = weighted(&[("a", 1.0), ("b", 5.0), ("c", 0.1), ("d", 2.0)]);

    let mut order = weighted_order(cards, 42);
    order.sort();

    assert_eq!(order, vec!["a", "b", "c", "d"]);
}

#[test]
fn a_weighted_order_puts_the_heavy_cards_earlier() {
    // Марафон проходит всю колоду, поэтому вес решает не «попадёт ли»,
    // а «когда»: слабые карточки должны идти ближе к началу.
    let cards: Vec<Weighted> = (0..100)
        .map(|n| Weighted {
            card_id: format!("card-{n:03}"),
            weight: if n < 10 { 8.0 } else { 1.0 },
        })
        .collect();

    let order = weighted_order(cards, 2026);
    let first_half = order[..50]
        .iter()
        .filter(|card| card.as_str() < "card-010")
        .count();

    assert!(
        first_half >= 7,
        "тяжёлые карточки не ушли вперёд: {first_half}"
    );
}

#[test]
fn a_weighted_order_of_nothing_is_nothing() {
    assert!(weighted_order(Vec::new(), 1).is_empty());
    assert_eq!(weighted_order(weighted(&[("одна", 1.0)]), 1), vec!["одна"]);
}
