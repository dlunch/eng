use num_traits::{identities::Zero, int::PrimInt};

pub fn round_up<T>(num_to_round: T, multiple: T) -> T
where
    T: PrimInt + Zero,
{
    if multiple == T::zero() {
        return num_to_round;
    }

    let remainder = num_to_round % multiple;
    if remainder == T::zero() {
        num_to_round
    } else {
        num_to_round + multiple - remainder
    }
}
