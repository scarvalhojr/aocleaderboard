use conv::ConvUtil;

pub fn number_width(num: usize) -> usize {
    1 + num.value_as::<f64>().unwrap_or(0_f64).log10().floor() as usize
}
