#![no_main]

libfuzzer_sys::fuzz_target!(
    init: {
        // SAFETY: libFuzzer runs this hook once, before fuzz workers start.
        unsafe { orgize::initialize_native_runtime() }.expect("native fuzz startup");
    },
    |data: &[u8]| {
        if let Ok(utf8) = std::str::from_utf8(data) {
            if let Ok(org) = orgize::Org::try_parse(utf8) {
                assert_eq!(org.to_org(), utf8);
            }
        }
    }
);
