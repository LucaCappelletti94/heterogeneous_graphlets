#![no_main]

use arbitrary::Arbitrary;
use heterogeneous_graphlets::perfect_graphlet_hash::PerfectGraphletHash;
use heterogeneous_graphlets::prelude::ExtendedGraphletType;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    kind: u8,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    n: u8,
}

fuzz_target!(|input: Input| {
    // 1..=32 labels, with every node label strictly below the label count.
    let n = (input.n % 32) + 1;
    let kind = ExtendedGraphletType::from(input.kind % 12);
    let quad = (input.a % n, input.b % n, input.c % n, input.d % n);

    let encoded: u128 = quad.encode_with_graphlet::<ExtendedGraphletType>(kind, n);
    let decoded = <(u8, u8, u8, u8)>::decode_graphlet_kind::<ExtendedGraphletType>(encoded, n);

    // The kind must survive an encode/decode round-trip.
    assert_eq!(decoded, kind);
});
