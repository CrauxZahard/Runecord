//! User actions, grouped by feature.

use super::*;

mod account;
mod admin;
mod guilds;
mod media;
mod messages;
mod moderation;
mod navigation;
mod picks;
mod social;

fn timeout_seconds(value: u64, unit: TimeoutUnit) -> u64 {
    value.min(unit.max_value()) * unit.seconds()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_timeout_converts_and_clamps() {
        assert_eq!(timeout_seconds(10, TimeoutUnit::Minutes), 600);
        assert_eq!(timeout_seconds(3, TimeoutUnit::Hours), 10_800);
        assert_eq!(timeout_seconds(3, TimeoutUnit::Days), 259_200);
        assert_eq!(timeout_seconds(28, TimeoutUnit::Days), 2_419_200);
        assert_eq!(timeout_seconds(29, TimeoutUnit::Days), 2_419_200);
        assert_eq!(timeout_seconds(99_999, TimeoutUnit::Minutes), 2_419_200);
    }
}
