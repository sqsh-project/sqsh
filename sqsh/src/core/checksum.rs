pub trait Checksum {
    type Output;

    fn checksum(&self) -> Self::Output;
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod tests {
    use super::Checksum;
    use crate::core::Process;
    use std::fmt::{Debug, Display};

    pub(crate) fn assert_checksum<
        T: PartialEq + Debug,
        C: Default + Process + Checksum<Output = T>,
    >(
        source: &[u8],
        expected: <C as Checksum>::Output,
    ) {
        let mut model: C = Default::default();
        let mut sink = Vec::<u8>::new();
        model.process(source, &mut sink).expect("Error");
        assert_eq!(model.checksum(), expected);
    }

    pub(crate) fn check_debug_format<C: Default + Debug>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m:?}"), expected)
    }

    pub(crate) fn check_display_format<C: Default + Display>(expected: &str) {
        let m: C = Default::default();
        assert_eq!(format!("{m}"), expected)
    }
}
