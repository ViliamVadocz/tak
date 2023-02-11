const EXPLORATION_BASE: f32 = 20_000.0;
const EXPLORATION_INIT: f32 = 2.5;

/// ```latex
/// c_{puct}(s) = \log \frac{\sum_a N(s, a) + c_{puct\_base} + 1}{c_{puct\_base}} + c_{puct\_init}
/// ```
fn exploration_rate(sum_of_action_visits: f32) -> f32 {
    ((sum_of_action_visits + EXPLORATION_BASE + 1.0) / EXPLORATION_BASE).log2() + EXPLORATION_INIT
}

/// ```latex
/// U(s_t, a) = c_{puct} P(s_t, a) \frac{\sqrt{\sum_b N(s_t, b)}}{1 + N(s_t, a)}
/// ```
fn exploration(policy: f32, sum_of_action_visits: f32, action_visits: f32) -> f32 {
    exploration_rate(sum_of_action_visits) * policy * sum_of_action_visits.sqrt()
        / (1.0 + action_visits)
}

/// Q(s_t, a) + U(s_t, a)
pub fn upper_confidence_bound(
    q_value: f32,
    policy: f32,
    sum_of_action_visits: f32,
    action_visits: f32,
) -> f32 {
    q_value + exploration(policy, sum_of_action_visits, action_visits)
}
