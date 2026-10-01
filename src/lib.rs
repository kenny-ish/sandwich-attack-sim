#[derive(Clone, Copy, Debug)]
pub struct Pool {
    pub x: f64,
    pub y: f64,
    pub fee: f64,
}

impl Pool {
    /// Swap `dx` of X for Y; returns output and updates reserves.
    pub fn buy_y(&mut self, dx: f64) -> f64 {
        let dx_eff = dx * (1.0 - self.fee);
        let out = self.y * dx_eff / (self.x + dx_eff);
        self.x += dx;
        self.y -= out;
        out
    }

    /// Swap `dy` of Y back for X.
    pub fn sell_y(&mut self, dy: f64) -> f64 {
        let dy_eff = dy * (1.0 - self.fee);
        let out = self.x * dy_eff / (self.y + dy_eff);
        self.y += dy;
        self.x -= out;
        out
    }

    pub fn quote_y(&self, dx: f64) -> f64 {
        let mut p = *self;
        p.buy_y(dx)
    }
}

#[derive(Debug)]
pub struct Outcome {
    pub front_run: f64,
    pub profit: f64,
    pub victim_out: f64,
    pub victim_out_clean: f64,
}

/// Run a sandwich with front-run size `a`. Returns None if the victim's trade would revert.
pub fn sandwich(pool: Pool, victim_in: f64, min_out: f64, a: f64) -> Option<Outcome> {
    let clean = pool.quote_y(victim_in);
    let mut p = pool;
    let bought = p.buy_y(a);
    let victim_out = p.buy_y(victim_in);
    if victim_out < min_out {
        return None;
    }
    let back = p.sell_y(bought);
    Some(Outcome { front_run: a, profit: back - a, victim_out, victim_out_clean: clean })
}

/// Largest front-run that keeps the victim above `min_out` (victim output falls monotonically in `a`).
pub fn max_front_run(pool: Pool, victim_in: f64, min_out: f64) -> f64 {
    if sandwich(pool, victim_in, min_out, 0.0).is_none() {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0, pool.x * 10.0);
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if sandwich(pool, victim_in, min_out, mid).is_some() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

pub fn best_attack(pool: Pool, victim_in: f64, slippage: f64) -> Outcome {
    let min_out = pool.quote_y(victim_in) * (1.0 - slippage);
    let a = max_front_run(pool, victim_in, min_out);
    sandwich(pool, victim_in, min_out, a).expect("a is feasible by construction")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> Pool {
        Pool { x: 5_000_000.0, y: 1_500.0, fee: 0.003 }
    }

    #[test]
    fn round_trip_without_victim_loses_fees() {
        let mut p = pool();
        let y = p.buy_y(10_000.0);
        assert!(p.sell_y(y) < 10_000.0);
    }

    #[test]
    fn zero_slippage_leaves_nothing() {
        // bisection can still find a front-run of ~1e-10 that float rounding tolerates,
        // so "nothing" means below any meaningful amount rather than exactly zero
        let o = best_attack(pool(), 100_000.0, 0.0);
        assert!(o.front_run < 1e-6, "{o:?}");
        assert!(o.profit < 1e-6, "{o:?}");
    }

    #[test]
    fn loose_slippage_is_profitable_and_respects_min_out() {
        let o = best_attack(pool(), 100_000.0, 0.01);
        assert!(o.profit > 0.0, "{o:?}");
        assert!(o.victim_out >= o.victim_out_clean * 0.99 - 1e-9);
        assert!(o.victim_out < o.victim_out_clean);
    }

    #[test]
    fn more_slippage_more_profit() {
        let tight = best_attack(pool(), 100_000.0, 0.005).profit;
        let loose = best_attack(pool(), 100_000.0, 0.03).profit;
        assert!(loose > tight);
    }
}
