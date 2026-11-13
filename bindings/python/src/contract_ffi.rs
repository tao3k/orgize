//! C SDK dispatch to the same Scheme owner as parsing.
use orgize::c_ffi::{ContractInput, ContractRow};
use std::ffi::{CStr, c_char};

#[repr(C)]
pub struct OrgizeElementRow {
    id: i64,
    parent_id: i64,
    kind: *const c_char,
    field_name: *const c_char,
    field_value: *const c_char,
}
#[repr(C)]
pub struct OrgizeContractResult {
    status: i32,
    matched_count: u32,
    passed: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn orgize_shared_abi_revision() -> u32 {
    2
}

/// # Safety
/// First call requires the same exclusive startup contract as the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn orgize_shared_runtime_initialize() -> i32 {
    match unsafe { orgize::initialize_native_runtime() } {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

unsafe fn text(pointer: *const c_char) -> Result<String, String> {
    if pointer.is_null() {
        return Err("null Contract string".into());
    }
    // SAFETY: the ABI caller owns a readable, terminated string for this call.
    unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map(str::to_owned)
        .map_err(|e| e.to_string())
}

/// All pointers are call-scoped; result is writable and rows/strings readable.
/// Rust copies the request before sending it to the single Gerbil owner.
///
/// # Safety
/// Non-null output must be aligned and writable. When count is nonzero, rows
/// must reference that many aligned, readable rows. Every string pointer must
/// reference a readable NUL-terminated string for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn orgize_shared_contract_evaluate(
    rows: *const OrgizeElementRow,
    count: u32,
    scope: i64,
    kind: *const c_char,
    field: *const c_char,
    value: *const c_char,
    expectation: u32,
    expected: u32,
    output: *mut OrgizeContractResult,
) -> i32 {
    if output.is_null() {
        return -1;
    }
    unsafe {
        *output = OrgizeContractResult {
            status: -1,
            matched_count: 0,
            passed: 0,
        };
    }
    let answer = (|| {
        if count > 10_000 || (count != 0 && rows.is_null()) {
            return Err("invalid Contract row span".into());
        }
        let slice = if count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(rows, count as usize) }
        };
        let copied = slice
            .iter()
            .map(|row| {
                Ok(ContractRow {
                    id: row.id,
                    parent_id: row.parent_id,
                    kind: unsafe { text(row.kind) }?,
                    field_name: unsafe { text(row.field_name) }?,
                    field_value: unsafe { text(row.field_value) }?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        orgize::c_ffi::evaluate_contract(ContractInput {
            rows: copied,
            scope_id: scope,
            kind: unsafe { text(kind) }?,
            field_name: unsafe { text(field) }?,
            field_value: unsafe { text(value) }?,
            expectation,
            expected_count: expected,
        })
    })();
    match answer {
        Ok(result) => {
            unsafe {
                *output = OrgizeContractResult {
                    status: 0,
                    matched_count: result.matched_count,
                    passed: i32::from(result.passed),
                };
            }
            0
        }
        Err(_) => -1,
    }
}
