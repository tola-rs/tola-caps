use tola_caps::AutoCaps;
use tola_caps::std_caps::AutoCapSet;

#[derive(Clone, Copy, Debug, Default, AutoCaps)]
struct ComplexFoo;

fn check<T: AutoCapSet>() {}

#[test]
fn test_repro_complex() {
    check::<ComplexFoo>();
}
