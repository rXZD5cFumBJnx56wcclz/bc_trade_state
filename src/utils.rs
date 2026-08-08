fn price_type(src: &[f64], type_: &str) -> f64 {
    match type_ {
        "index" => src[7],
        "mark" => src[8],
        "last" | _ => src[4],
    }
}

pub fn price_is_crossed(price: f64, src: &[f64], src_l: &[f64], type_price_crossed: &str) -> bool {
    price <= price_type(src, type_price_crossed) && price >= price_type(src_l, type_price_crossed)
}

pub fn price_is_crossed_direction(
    price: f64,
    direction: usize,
    src: &[f64],
    src_l: &[f64],
    type_price_crossed: &str,
) -> bool {
    let p = price_type(src, type_price_crossed);
    let p_l = price_type(src_l, type_price_crossed);
    match direction {
        1 => price <= p && price >= p_l,
        2 | _ => price >= p && price <= p_l,
    }
}

pub fn pnl(qty: f64, avg_price: f64, last_price: f64, leverage: f64, side: &str) -> (f64, f64) {
    let percent =
        (last_price - avg_price) / avg_price * leverage * if side == "buy" { 1. } else { -1. };
    (percent, percent * qty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude_tests::prelude::*;

    #[test]
    fn price_type_res_1() {
        assert_eq_pr!(
            price_type(
                &[
                    1., 1.9, 1.92, 1.89, 1.91, 11111., 111111., 1.910001, 1.910002,
                ],
                "last"
            ),
            1.91
        )
    }

    #[test]
    fn price_is_crossed_res_1() {
        assert_eq_pr!(price_is_crossed(1.9, &[1.91; 5], &[1.89; 5], "last"), true)
    }

    #[test]
    fn price_is_crossed_direction_res_1() {
        assert_eq_pr!(
            price_is_crossed_direction(1.9, 1, &[1.91; 5], &[1.89; 5], "last"),
            true
        )
    }

    #[test]
    fn price_is_crossed_direction_res_2() {
        assert_eq_pr!(
            price_is_crossed_direction(1.9, 2, &[1.91; 5], &[1.89; 5], "last"),
            false
        )
    }

    #[test]
    fn price_is_crossed_direction_res_3() {
        assert_eq_pr!(
            price_is_crossed_direction(1.9, 2, &[1.89; 5], &[1.91; 5], "last"),
            true
        )
    }

    #[test]
    fn pnl_res_1() {
        let percent = (1.95 - 1.9) / 1.9 * 2. * -1.;
        assert_eq_pr!(pnl(10., 1.9, 1.95, 2., "sell"), (percent, percent * 10.))
    }
}
