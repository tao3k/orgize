//! Private transport for the existing POO event strategy; never a Rust parser.
use super::batch;
use crate::c_ffi::{ContractInput, ContractOutput};
use crate::org_aot::{NativeExpressionValue, native_expression};
use gerbil_parser_runtime::{KindCategory, LanguageSpec, TreeEvent};
// Native ingress references this crate's C ABI without calling it from Rust.
// Retain its Rust ABI wrappers and native lifecycle archive in both lanes.
#[cfg(not(feature = "runtime-scheme"))]
use gerbil_scheme::{GerbilRuntime, LinkedGerbilProgram};
#[cfg(feature = "runtime-scheme")]
use gerbil_scheme_sys as _;
use std::ffi::{CString, c_char};
#[cfg(not(feature = "runtime-scheme"))]
use std::sync::{
    OnceLock,
    mpsc::{self, SyncSender},
};

unsafe extern "C" {
    #[cfg(not(feature = "runtime-scheme"))]
    fn ___LNK_orgize__gerbil__program(
        state: *mut gerbil_scheme_sys::GerbilGlobalState,
    ) -> *mut gerbil_scheme_sys::GerbilModuleOrLink;
    #[cfg(not(feature = "runtime-scheme"))]
    fn orgize_parse_events(input: *const u8, length: usize) -> i64;
    #[cfg_attr(feature = "runtime-scheme", link_name = "orgize_scheme_contract")]
    #[cfg_attr(
        not(feature = "runtime-scheme"),
        link_name = "orgize_contract_evaluate"
    )]
    fn execute_contract(
        rows: *const RawRow,
        count: u32,
        scope: i64,
        kind: *const c_char,
        field: *const c_char,
        value: *const c_char,
        expectation: u32,
        expected: u32,
        result: *mut RawResult,
    ) -> i32;
}

#[cfg(not(feature = "runtime-scheme"))]
enum Request {
    Parse {
        bytes: Vec<u8>,
        submitted: crate::runtime_profile::Stamp,
        response: SyncSender<crate::runtime_profile::OwnerReply<Result<Vec<u8>, String>>>,
    },
    Contract {
        input: ContractInput,
        response: SyncSender<Result<ContractOutput, String>>,
    },
}

#[repr(C)]
struct RawRow {
    id: i64,
    parent_id: i64,
    kind: *const c_char,
    field_name: *const c_char,
    field_value: *const c_char,
}

#[repr(C)]
struct RawResult {
    status: i32,
    matched_count: u32,
    passed: i32,
}

fn call_contract(input: ContractInput) -> Result<ContractOutput, String> {
    if input.rows.len() > 10_000
        || input.expected_count > 10_000
        || input.scope_id < 0
        || input.kind.is_empty()
        || input.expectation > 2
    {
        return Err("invalid Org contract ABI input".into());
    }
    let encoded = |value: &str| {
        CString::new(value).map_err(|_| "Contract strings cannot contain NUL bytes".to_owned())
    };
    let kind = encoded(&input.kind)?;
    let field = encoded(&input.field_name)?;
    let value = encoded(&input.field_value)?;
    let strings: Vec<_> = input
        .rows
        .iter()
        .map(|row| {
            if row.id < 0 || row.parent_id < -1 {
                return Err("invalid Org contract row".into());
            }
            Ok((
                encoded(&row.kind)?,
                encoded(&row.field_name)?,
                encoded(&row.field_value)?,
            ))
        })
        .collect::<Result<_, String>>()?;
    let rows: Vec<_> = input
        .rows
        .iter()
        .zip(&strings)
        .map(|(row, (kind, field, value))| RawRow {
            id: row.id,
            parent_id: row.parent_id,
            kind: kind.as_ptr(),
            field_name: field.as_ptr(),
            field_value: value.as_ptr(),
        })
        .collect();
    let mut result = RawResult {
        status: -1,
        matched_count: 0,
        passed: 0,
    };
    // SAFETY: all C storage stays live until this blocking call returns. The
    // Rust lane calls on its owner; the Scheme lane admits it to the native
    // owner without exposing Scheme handles. Both contain Scheme exceptions.
    let status = unsafe {
        execute_contract(
            rows.as_ptr(),
            rows.len() as u32,
            input.scope_id,
            kind.as_ptr(),
            field.as_ptr(),
            value.as_ptr(),
            input.expectation,
            input.expected_count,
            &mut result,
        )
    };
    if status != 0 || result.status != 0 {
        return Err("Scheme Contract rejected the admitted rows".into());
    }
    Ok(ContractOutput {
        matched_count: result.matched_count,
        passed: result.passed != 0,
    })
}

#[cfg(not(feature = "runtime-scheme"))]
static OWNER: OnceLock<Result<SyncSender<Request>, String>> = OnceLock::new();

#[cfg(not(feature = "runtime-scheme"))]
pub(crate) fn initialize_owner() -> Result<(), String> {
    OWNER
        .get_or_init(|| {
            // Match the Scheme lane's 64 queued requests plus one consumer.
            let (requests, incoming) = mpsc::sync_channel::<Request>(64);
            let (ready, startup) = mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("org-gerbil-aot".into())
                .stack_size(2 * 1024 * 1024)
                .spawn(move || {
                    // SAFETY: both symbols belong to this build's single static program.
                    // The descriptor is registered only here, on the runtime owner.
                    let program =
                        unsafe { LinkedGerbilProgram::from_linker(___LNK_orgize__gerbil__program) };
                    let runtime = match GerbilRuntime::initialize_program(program) {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            let _ = ready.send(Err(error.to_string()));
                            return;
                        }
                    };
                    // SAFETY: the Scheme export contains exceptions, borrows input only
                    // during the call, and transfers one fresh bytevector root or zero.
                    let export = match unsafe { runtime.bind_bytes_export(orgize_parse_events) } {
                        Ok(export) => export,
                        Err(error) => {
                            let _ = ready.send(Err(error.to_string()));
                            return;
                        }
                    };
                    let _ = ready.send(Ok(()));
                    for request in incoming {
                        match request {
                            Request::Parse {
                                bytes,
                                submitted,
                                response,
                            } => {
                                let admission = submitted.elapsed();
                                let service = submitted.child();
                                #[cfg(all(feature = "runtime-profile", unix))]
                                let cpu_begin = service.thread_cpu();
                                let mut copy_ns = 0;
                                let result = export
                                    .call(&bytes)
                                    .into_result()
                                    .and_then(|root| {
                                        let copy = submitted.child();
                                        let result = root.to_vec().into_result();
                                        copy_ns = copy.elapsed();
                                        drop(root);
                                        result
                                    })
                                    .map_err(|error| error.to_string());
                                // The root is dropped on this thread before its copied bytes
                                // reach callers. No runtime handle or Scheme value is Send.
                                #[cfg(all(feature = "runtime-profile", unix))]
                                let cpu_ns = cpu_begin.map_or(0, |begin| {
                                    service
                                        .thread_cpu()
                                        .expect("active owner CPU")
                                        .checked_sub(begin)
                                        .expect("native owner CPU regressed")
                                });
                                #[cfg(not(all(feature = "runtime-profile", unix)))]
                                let cpu_ns = 0;
                                let reply = crate::runtime_profile::OwnerReply::new(
                                    result,
                                    [admission, service.elapsed(), copy_ns, cpu_ns],
                                    submitted.child(),
                                );
                                let _ = response.send(reply);
                            }
                            Request::Contract { input, response } => {
                                let _ = response.send(call_contract(input));
                            }
                        }
                    }
                })
                .map_err(|error| error.to_string())?;
            startup
                .recv()
                .map_err(|_| "Gerbil owner failed during startup".to_owned())??;
            Ok(requests)
        })
        .as_ref()
        .map(|_| ())
        .map_err(Clone::clone)
}

#[cfg(not(feature = "runtime-scheme"))]
fn owner() -> Result<&'static SyncSender<Request>, String> {
    crate::startup::require_initialized()?;
    OWNER
        .get()
        .ok_or("native owner has not been started")?
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(feature = "runtime-scheme")]
pub(crate) fn initialize_owner() -> Result<(), String> {
    unsafe extern "C" {
        fn orgize_scheme_runtime_initialize() -> i32;
    }
    // SAFETY: caller holds exclusive startup admission for the linked program.
    if unsafe { orgize_scheme_runtime_initialize() } == 0 {
        Ok(())
    } else {
        Err("native Scheme owner failed during explicit startup".into())
    }
}

pub(crate) fn parse_events(
    source: &str,
    level: usize,
    policy: usize,
    grammar: &LanguageSpec,
) -> Result<Vec<TreeEvent>, String> {
    parse_request(source, 0, level, policy, grammar)
}

pub(crate) fn parse_events_batch(
    sources: &[&str],
    level: usize,
    policy: usize,
    grammar: &LanguageSpec,
) -> Result<Vec<Result<Vec<TreeEvent>, String>>, String> {
    if sources.is_empty() {
        return Ok(Vec::new());
    }
    crate::startup::require_initialized()?;
    let profile = crate::runtime_profile::is_active();
    let request = crate::runtime_profile::stage("native.request_encode", || {
        batch::encode(sources, level, policy, profile)
    })?;
    let tape =
        crate::runtime_profile::stage("native.transport_inclusive", || transport_parse(request))?;
    crate::runtime_profile::stage("native.event_decode", || {
        batch::decode_batch(tape.as_ref(), sources.len(), grammar, profile)
    })
}

#[cfg(test)]
pub(crate) fn parse_expression_events(
    source: &str,
    grammar: &LanguageSpec,
) -> Result<Vec<TreeEvent>, String> {
    parse_request(source, 1, 0, 0, grammar)
}

fn parse_request(
    source: &str,
    operation: u8,
    level: usize,
    policy: usize,
    grammar: &LanguageSpec,
) -> Result<Vec<TreeEvent>, String> {
    let tape = request_tape(source, operation, level, policy)?;
    #[cfg(feature = "runtime-profile")]
    let bytes = if operation == 0 && crate::runtime_profile::is_active() {
        crate::runtime_profile::native_tape(tape.as_ref())?
    } else {
        tape.as_ref()
    };
    #[cfg(not(feature = "runtime-profile"))]
    let bytes = tape.as_ref();
    crate::runtime_profile::stage("native.event_decode", || decode(bytes, grammar))
}

pub(crate) fn parse_expression_values(
    source: &str,
    grammar: &LanguageSpec,
) -> Result<Vec<NativeExpressionValue>, String> {
    parse_values(source, 2, grammar)
}

fn parse_values(
    source: &str,
    operation: u8,
    grammar: &LanguageSpec,
) -> Result<Vec<NativeExpressionValue>, String> {
    let tape = request_tape(source, operation, 0, 0)?;
    native_expression::decode(tape.as_ref(), grammar.grammar_digest)
}

fn request_tape(
    source: &str,
    operation: u8,
    level: usize,
    policy: usize,
) -> Result<NativeTape, String> {
    request_bytes(source.as_bytes(), operation, level, policy)
}

fn request_bytes(
    source: &[u8],
    operation: u8,
    level: usize,
    policy: usize,
) -> Result<NativeTape, String> {
    crate::startup::require_initialized()?;
    let bytes =
        crate::runtime_profile::stage("native.request_encode", || -> Result<Vec<u8>, String> {
            let mut bytes =
                Vec::with_capacity(source.len().checked_add(14).ok_or("Org input too large")?);
            bytes.extend_from_slice(b"ONR1");
            bytes.push(if operation == 0 && crate::runtime_profile::is_active() {
                5
            } else {
                operation
            });
            bytes.extend_from_slice(
                &u64::try_from(level)
                    .map_err(|_| "Org level overflow")?
                    .to_le_bytes(),
            );
            bytes.push(u8::try_from(policy).map_err(|_| "Org policy overflow")?);
            bytes.extend_from_slice(source);
            Ok(bytes)
        })?;
    crate::runtime_profile::stage("native.transport_inclusive", || {
        crate::runtime_profile::operation(bytes[4], || transport_parse(bytes))
    })
}

/// Encode values only. Scheme owns template syntax and secondary-value calls.
pub(crate) fn expand_macro_fields(operation: u8, fields: &[&str]) -> Result<String, String> {
    if !matches!(operation, 8 | 9 | 15) || fields.is_empty() {
        return Err("invalid native macro operation".into());
    }
    let mut values = semantic_fields(operation, fields)?;
    if values.len() != 1 {
        return Err("invalid native macro response count".into());
    }
    match values.pop().unwrap() {
        NativeExpressionValue::String(value) => Ok(value),
        _ => Err("invalid native macro response value".into()),
    }
}

/// Counted wire values only; source syntax is never interpreted in Rust.
pub(crate) fn semantic_fields(
    operation: u8,
    fields: &[&str],
) -> Result<Vec<NativeExpressionValue>, String> {
    if !(8..=24).contains(&operation) || fields.is_empty() {
        return Err("invalid native semantic operation".into());
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| "too many macro fields")?
            .to_le_bytes(),
    );
    for field in fields {
        bytes.extend_from_slice(
            &u32::try_from(field.len())
                .map_err(|_| "macro field too large")?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(field.as_bytes());
    }
    let tape = request_bytes(&bytes, operation, 0, 0)?;
    native_expression::decode(
        tape.as_ref(),
        crate::org_aot::org_language_spec().grammar_digest,
    )
}

#[cfg(not(feature = "runtime-scheme"))]
fn transport_parse(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let (response, result) = mpsc::sync_channel(1);
    let submitted = crate::runtime_profile::Stamp::start();
    owner()?
        .send(Request::Parse {
            bytes,
            submitted,
            response,
        })
        .map_err(|_| "Gerbil owner terminated")?;
    Ok(result
        .recv()
        .map_err(|_| "Gerbil call terminated")?
        .receive()?)
}

#[cfg(not(feature = "runtime-scheme"))]
type NativeTape = Vec<u8>;

#[cfg(not(feature = "runtime-scheme"))]
pub(crate) fn evaluate_contract(input: ContractInput) -> Result<ContractOutput, String> {
    let (response, result) = mpsc::sync_channel(1);
    owner()?
        .send(Request::Contract { input, response })
        .map_err(|_| "Gerbil owner terminated")?;
    result
        .recv()
        .map_err(|_| "Gerbil contract call terminated")?
}

#[cfg(feature = "runtime-scheme")]
pub(crate) fn evaluate_contract(input: ContractInput) -> Result<ContractOutput, String> {
    crate::startup::require_initialized()?;
    call_contract(input)
}

#[cfg(feature = "runtime-scheme")]
unsafe extern "C" {
    fn orgize_scheme_parse(
        input: *const u8,
        length: usize,
        output: *mut *mut u8,
        output_length: *mut usize,
    ) -> i32;
    fn orgize_scheme_free(output: *mut u8);
    #[cfg(feature = "runtime-profile")]
    fn orgize_scheme_parse_profiled(
        input: *const u8,
        length: usize,
        output: *mut *mut u8,
        output_length: *mut usize,
        profile: *mut u64,
    ) -> i32;
}

// Native malloc storage is decoded in place: no extra C-buffer-to-Vec copy.
// The caller owns only these bytes, never a Scheme object or root token.
#[cfg(feature = "runtime-scheme")]
struct NativeTape {
    pointer: std::ptr::NonNull<u8>,
    length: usize,
}

#[cfg(feature = "runtime-scheme")]
impl AsRef<[u8]> for NativeTape {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: native submit transfers this initialized allocation solely to
        // this caller; length is checked below and Drop frees it exactly once.
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), self.length) }
    }
}

#[cfg(feature = "runtime-scheme")]
impl Drop for NativeTape {
    fn drop(&mut self) {
        // SAFETY: the allocator and matching release belong to the same ABI.
        unsafe { orgize_scheme_free(self.pointer.as_ptr()) };
    }
}

#[cfg(feature = "runtime-scheme")]
fn transport_parse(bytes: Vec<u8>) -> Result<NativeTape, String> {
    let mut pointer = std::ptr::null_mut();
    let mut length = 0;
    // SAFETY: input remains borrowed/live through native admission and job
    // completion. The native actor copies its Scheme tape before waking us;
    // the result never contains a Scheme pointer or GC root.
    #[cfg(feature = "runtime-profile")]
    let mut timings = [0_u64; 5];
    let status = {
        #[cfg(feature = "runtime-profile")]
        if crate::runtime_profile::is_active() {
            // SAFETY: all five timing words stay live until synchronous completion.
            unsafe {
                orgize_scheme_parse_profiled(
                    bytes.as_ptr(),
                    bytes.len(),
                    &mut pointer,
                    &mut length,
                    timings.as_mut_ptr(),
                )
            }
        } else {
            unsafe { orgize_scheme_parse(bytes.as_ptr(), bytes.len(), &mut pointer, &mut length) }
        }
        #[cfg(not(feature = "runtime-profile"))]
        unsafe {
            orgize_scheme_parse(bytes.as_ptr(), bytes.len(), &mut pointer, &mut length)
        }
    };
    if status != 0 || pointer.is_null() || length == 0 || length > isize::MAX as usize {
        if !pointer.is_null() {
            // SAFETY: any non-null output was allocated by this native ABI.
            unsafe { orgize_scheme_free(pointer) };
        }
        return Err("Scheme runtime failed native parse admission".into());
    }
    #[cfg(feature = "runtime-profile")]
    if crate::runtime_profile::is_active() {
        crate::runtime_profile::record_transport(
            [timings[0], timings[1], timings[2], timings[4]],
            timings[3],
        );
    }
    Ok(NativeTape {
        pointer: std::ptr::NonNull::new(pointer).unwrap(),
        length,
    })
}

pub(super) fn decode(mut bytes: &[u8], grammar: &LanguageSpec) -> Result<Vec<TreeEvent>, String> {
    bytes = bytes
        .strip_prefix(b"OEV1")
        .and_then(|body| body.strip_prefix(grammar.grammar_digest.as_bytes()))
        .ok_or("Org native grammar identity mismatch")?;
    let mut events = Vec::new();
    while let Some((&tag, tail)) = bytes.split_first() {
        bytes = tail;
        if tag == 0 {
            events.push(TreeEvent::FinishNode);
            continue;
        }
        if !matches!(tag, 1 | 2) {
            return Err("invalid Org event tag".into());
        }
        let width = if tag == 1 { 2 } else { 18 };
        let record = bytes.get(..width).ok_or("truncated Org native event")?;
        bytes = &bytes[width..];
        let kind = u16::from_le_bytes(record[..2].try_into().unwrap());
        let spec = grammar
            .kinds
            .get(usize::from(kind))
            .ok_or("unknown Org native kind")?;
        if tag == 1 {
            if spec.category != KindCategory::Node {
                return Err("Org native start is not a node".into());
            }
            events.push(TreeEvent::StartNode(kind));
        } else {
            if spec.category != KindCategory::Token {
                return Err("Org native token is not a token".into());
            }
            let start = usize::try_from(u64::from_le_bytes(record[2..10].try_into().unwrap()))
                .map_err(|_| "Org native offset overflow")?;
            let end = usize::try_from(u64::from_le_bytes(record[10..18].try_into().unwrap()))
                .map_err(|_| "Org native offset overflow")?;
            events.push(TreeEvent::Token { kind, start, end });
        }
    }
    Ok(events)
}

#[cfg(test)]
#[path = "../../tests/unit/org_events.rs"]
mod tests;
