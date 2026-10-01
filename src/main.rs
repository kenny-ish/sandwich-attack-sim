use sandwich_attack_sim::{best_attack, Pool};

fn get(args: &[String], name: &str, default: f64) -> f64 {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pool = Pool {
        x: get(&args, "--reserve-in", 5_000_000.0),
        y: get(&args, "--reserve-out", 1_500.0),
        fee: get(&args, "--fee", 0.3) / 100.0,
    };
    let amount = get(&args, "--amount", 100_000.0);
    let slippage = get(&args, "--slippage", 1.0) / 100.0;

    let o = best_attack(pool, amount, slippage);
    let impact = 1.0 - (o.victim_out_clean / amount) / (pool.y / pool.x);
    println!("victim swaps {amount} X, slippage tolerance {:.2}%", slippage * 100.0);
    println!("  price impact of the swap itself  {:.2}%", impact * 100.0);
    println!("  output without attack            {:.6} Y", o.victim_out_clean);
    println!("attacker");
    println!("  front-run size                   {:.2} X", o.front_run);
    println!("  profit before gas                {:.2} X", o.profit);
    println!("victim");
    println!("  output with attack               {:.6} Y", o.victim_out);
    println!("  loss                             {:.6} Y ({:.2}%)",
        o.victim_out_clean - o.victim_out, (1.0 - o.victim_out / o.victim_out_clean) * 100.0);
}
