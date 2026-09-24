//! Direct Rust consumer of the standalone Scheme-owned Orgize C ABI.
//! This is deliberately outside Cargo's normal build: ordinary Rust users
//! never need Gerbil merely to parse Org documents.

use std::ffi::{c_char, CString};

#[repr(C)]
struct OrgizeElementRow {
    id: i64,
    parent_id: i64,
    kind: *const c_char,
    field_name: *const c_char,
    field_value: *const c_char,
}

#[repr(C)]
#[derive(Default)]
struct OrgizeContractResult {
    status: i32,
    matched_count: u32,
    passed: i32,
}

#[link(name = "orgize")]
unsafe extern "C" {
    fn orgize_runtime_init() -> i32;
    fn orgize_runtime_shutdown();
    fn orgize_abi_revision() -> u32;
    fn orgize_contract_evaluate(
        rows: *const OrgizeElementRow,
        row_count: u32,
        scope_id: i64,
        query_kind: *const c_char,
        field_name: *const c_char,
        field_value: *const c_char,
        expectation: u32,
        expected_count: u32,
        result: *mut OrgizeContractResult,
    ) -> i32;
}

fn main() {
    let root = CString::new("org-data").unwrap();
    let headline = CString::new("headline").unwrap();
    let title = CString::new("title").unwrap();
    let evidence = CString::new("Evidence").unwrap();
    let empty = CString::new("").unwrap();
    let rows = [
        OrgizeElementRow {
            id: 0,
            parent_id: -1,
            kind: root.as_ptr(),
            field_name: empty.as_ptr(),
            field_value: empty.as_ptr(),
        },
        OrgizeElementRow {
            id: 1,
            parent_id: 0,
            kind: headline.as_ptr(),
            field_name: title.as_ptr(),
            field_value: evidence.as_ptr(),
        },
    ];
    let mut result = OrgizeContractResult::default();
    unsafe {
        assert_eq!(orgize_runtime_init(), 0);
        assert_eq!(orgize_abi_revision(), 1);
        assert_eq!(
            orgize_contract_evaluate(
                rows.as_ptr(), 2, 0, headline.as_ptr(), title.as_ptr(),
                evidence.as_ptr(), 0, 1, &mut result,
            ),
            0
        );
        assert_eq!((result.status, result.matched_count, result.passed), (0, 1, 1));
        orgize_runtime_shutdown();
        assert_eq!(orgize_runtime_init(), -1);
    }
}
