use std::convert::TryInto;

use crate::{
    common::{Coin, GenericCoinAmount, Liquidity, LokiAmount},
    constants::LOKI_SWAP_PROCESS_FEE,
    utils,
};

use num_bigint::BigInt;

mod search;

fn calc_autoswap_from_loki(
    loki_amount: LokiAmount,
    other_amount: GenericCoinAmount,
    liquidity: Liquidity,
) -> Result<(LokiAmount, GenericCoinAmount), &'static str> {
    if loki_amount.to_atomic() <= LOKI_SWAP_PROCESS_FEE {
        warn!("Fee exceeds staked amount");
        let stake = calc_symmetric_from_other(other_amount, liquidity);
        return Ok((stake.loki, stake.other));
    }

    let x = search::find_loki_x(loki_amount, other_amount, liquidity, LOKI_SWAP_PROCESS_FEE)
        .unwrap_or(0);

    let y = utils::price::calculate_output_amount(
        Coin::LOKI,
        x,
        liquidity.loki_depth,
        LOKI_SWAP_PROCESS_FEE,
        other_amount.coin_type(),
        liquidity.depth,
        0,
    )
    .unwrap_or(0);

    let loki_effective = LokiAmount::from_atomic(loki_amount.to_atomic().saturating_sub(x));
    let other_effective = GenericCoinAmount::from_atomic(
        other_amount.coin_type(),
        other_amount.to_atomic().saturating_add(y),
    );

    if y == 0 {
        debug!("Auto-swapped amount is negligible");
        let stake = calc_symmetric_from_other(other_amount, liquidity);
        Ok((stake.loki, stake.other))
    } else {
        // Liquidity "changed" due to autoswap
        let loki_depth = liquidity.loki_depth + x - LOKI_SWAP_PROCESS_FEE;
        let depth = liquidity.depth - y;

        validate_autoswap(
            loki_effective,
            other_effective,
            Liquidity { loki_depth, depth },
        )
        .map_err(|_| "Autoswap didn't pass validity check")?;
        Ok((loki_effective, other_effective))
    }
}

fn small_other_stake(other_amount: GenericCoinAmount, liquidity: Liquidity) -> bool {
    let e: BigInt = other_amount.to_atomic().into();
    let dl: BigInt = liquidity.loki_depth.into();
    let de: BigInt = liquidity.depth.into();

    // The amount of loki that we would receive after swapping all of the other coin
    let max_loki = &e * &dl * &de / ((&e + &de) * (&e + &de)) - BigInt::from(LOKI_SWAP_PROCESS_FEE);

    max_loki < BigInt::from(0)
}

fn calc_autoswap_to_loki(
    loki_amount: LokiAmount,
    other_amount: GenericCoinAmount,
    liquidity: Liquidity,
) -> Result<(LokiAmount, GenericCoinAmount), &'static str> {
    // Input fee is 0 because we are swapping
    // some other coin for Loki

    // This only checks that the amount of the other coin is there in principle, i.e.
    // to make *some* kind of swap
    if small_other_stake(other_amount, liquidity) {
        warn!("Fee exceeds staked amount");
        let stake = calc_symmetric_from_loki(loki_amount, other_amount.coin_type(), liquidity);
        return Ok((stake.loki, stake.other));
    }

    let x = match search::find_other_x(loki_amount, other_amount, liquidity, LOKI_SWAP_PROCESS_FEE)
    {
        Some(x) => x,
        None => {
            // It is possible pass the test above, but still have only a marginal amount
            // extra of the other coin (not enough to pay for the fee)
            info!(
                "No amount of other coin can be autoswapped, falling back to staking symmetrically"
            );
            let stake = calc_symmetric_from_loki(loki_amount, other_amount.coin_type(), liquidity);
            return Ok((stake.loki, stake.other));
        }
    };

    let y = utils::price::calculate_output_amount(
        other_amount.coin_type(),
        x,
        liquidity.depth,
        0,
        Coin::LOKI,
        liquidity.loki_depth,
        LOKI_SWAP_PROCESS_FEE,
    )
    .unwrap_or(0);

    let loki_effective = LokiAmount::from_atomic(loki_amount.to_atomic().saturating_add(y));
    let other_effective = GenericCoinAmount::from_atomic(
        other_amount.coin_type(),
        other_amount.to_atomic().saturating_sub(x),
    );

    if y == 0 {
        debug!("Auto-swapped amount is negligible");
        let stake = calc_symmetric_from_loki(loki_amount, other_amount.coin_type(), liquidity);
        Ok((stake.loki, stake.other))
    } else {
        // Liquidity "changed" due to autoswap
        let loki_depth = liquidity.loki_depth - y - LOKI_SWAP_PROCESS_FEE;
        let depth = liquidity.depth + x;

        validate_autoswap(
            loki_effective,
            other_effective,
            Liquidity { loki_depth, depth },
        )
        .map_err(|_| "Autoswap didn't pass validity check")?;
        Ok((loki_effective, other_effective))
    }
}

fn validate_autoswap(
    loki_effective_amount: LokiAmount,
    other_effective_amount: GenericCoinAmount,
    liquidity: Liquidity,
) -> Result<(), ()> {
    let l: BigInt = loki_effective_amount.to_atomic().into();
    let e: BigInt = other_effective_amount.to_atomic().into();

    let de: BigInt = liquidity.depth.into();
    let dl: BigInt = liquidity.loki_depth.into();

    // Error in atomic loki (easier to calculate in whole numbers)
    let error = (dl * e) / de - &l;

    // We multiply the nominator by this amount because we work
    // with whole number, which can't represent fractions
    const ACCURACY: u32 = 1_000_000;

    // Normalize error by the input amount:
    let error = (BigInt::from(ACCURACY) * error) / &l;

    let error: i128 = error.try_into().map_err(|_| ())?;

    if error.abs() > 1 {
        return Err(());
    }

    Ok(())
}

/// Determines which way the swap should go. Note that it doesn't take fees into account:
/// for now the user always pays fees even if the autoswapped amount (y) would be smaller than fee
/// payed from that amount (o_fee).
fn calc_swap_direction(
    loki_amount: LokiAmount,
    other_amount: GenericCoinAmount,
    liquidity: Liquidity,
) -> SwapDirection {
    let l: BigInt = loki_amount.to_atomic().into();
    let e: BigInt = other_amount.to_atomic().into();

    let dl: BigInt = liquidity.loki_depth.into();
    let de: BigInt = liquidity.depth.into();

    let gamma = &l * &de - &e * &dl;

    if gamma >= BigInt::from(0) {
        SwapDirection::FromLoki
    } else {
        SwapDirection::ToLoki
    }
}

#[derive(Debug, PartialEq, Eq)]
/// In which direction to perform autoswap
enum SwapDirection {
    /// Other coin to Loki
    ToLoki,
    /// Loki to other coin
    FromLoki,
}

struct EffectiveStakeAmounts {
    loki: LokiAmount,
    other: GenericCoinAmount,
}

/// Calculate the ideal amount of loki to be staked
/// together with `other` amount (to make the stake symmetrical)
fn calc_symmetric_from_other(
    other_amount: GenericCoinAmount,
    liquidity: Liquidity,
) -> EffectiveStakeAmounts {
    let e: BigInt = other_amount.to_atomic().into();
    let de: BigInt = liquidity.depth.into();
    let dl: BigInt = liquidity.loki_depth.into();

    let loki = (e * dl) / de;

    let loki: u128 = loki.try_into().expect("unexpected overflow");

    EffectiveStakeAmounts {
        loki: LokiAmount::from_atomic(loki),
        other: other_amount,
    }
}

/// Calculate the ideal amount of loki to be staked
/// together with `other` amount (to make the stake symmetrical)
fn calc_symmetric_from_loki(
    loki_amount: LokiAmount,
    other_coin: Coin,
    liquidity: Liquidity,
) -> EffectiveStakeAmounts {
    let l: BigInt = loki_amount.to_atomic().into();
    let de: BigInt = liquidity.depth.into();
    let dl: BigInt = liquidity.loki_depth.into();

    let other = (l * de) / dl;

    let other: u128 = other.try_into().expect("unexpected overflow");

    EffectiveStakeAmounts {
        loki: loki_amount,
        other: GenericCoinAmount::from_atomic(other_coin, other),
    }
}

/// Calculate effective contribution
pub(crate) fn calc_autoswap_amount(
    loki_amount: LokiAmount,
    other_amount: GenericCoinAmount,
    liquidity: Liquidity,
) -> Result<(LokiAmount, GenericCoinAmount), &'static str> {
    // Need to determine which way to swap:

    match calc_swap_direction(loki_amount, other_amount, liquidity) {
        SwapDirection::FromLoki => calc_autoswap_from_loki(loki_amount, other_amount, liquidity),
        SwapDirection::ToLoki => calc_autoswap_to_loki(loki_amount, other_amount, liquidity),
    }
}

#[cfg(test)]
mod tests;
