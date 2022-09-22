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

//returns zero if v is zero.
pub fn round_up_power_of_two(mut v: u32) -> u32 {
    //from http://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2 (public domain)

    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;

    v
}
