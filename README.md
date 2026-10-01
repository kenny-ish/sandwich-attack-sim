# sandwich-attack-sim

Simulates a sandwich attack on a swap in a constant-product pool and finds the largest attack the
victim's slippage setting allows.

```bash
cargo run --release -- --reserve-in 5000000 --reserve-out 1500 --amount 100000 --slippage 1
cargo run --release -- --reserve-in 5000000 --reserve-out 1500 --amount 100000 --slippage 0.1
```

The victim swaps `amount` of token X for token Y with `minOut = quote * (1 - slippage)`. The attacker

1. buys Y with `a` of X right before the victim, which moves the price up,
2. lets the victim's swap execute at the worse price, still above `minOut`,
3. sells the Y back into the pool right after.

The simulator bisects for the largest `a` that doesn't make the victim's swap revert and reports

- the size of the front-run and the attacker's profit in X, after pool fees on both legs but before
  gas, and
- how much less Y the victim got compared with a swap nobody attacked.

With the slippage set to the victim's own price impact plus a small margin, the profitable attack
size drops to almost nothing. That's why a tight slippage setting, or a private order flow, matters.

```bash
cargo test
```
