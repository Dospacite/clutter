use crate::diagnostic::{ClutterError, Result};
use crate::model::SnapshotInfo;

#[derive(Clone, Copy)]
pub struct Profile {
    pub cids: Cids,
    pub compressed_pointers: bool,
    pub instance_header_words: usize,
    pub unboxed_word_u32_chunks: usize,
}

#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cids {
    pub class: i32,
    pub patch_class: i32,
    pub function: i32,
    pub type_parameters: i32,
    pub closure_data: i32,
    pub ffi_trampoline_data: i32,
    pub field: i32,
    pub script: i32,
    pub library: i32,
    pub namespace: i32,
    pub kernel_program_info: i32,
    pub weak_serialization_reference: i32,
    pub weak_array: i32,
    pub code: i32,
    pub object_pool: i32,
    pub pc_descriptors: i32,
    pub code_source_map: i32,
    pub compressed_stack_maps: i32,
    pub exception_handlers: i32,
    pub context: i32,
    pub context_scope: i32,
    pub sentinel: i32,
    pub single_target_cache: i32,
    pub unlinked_call: i32,
    pub monomorphic_smiable_call: i32,
    pub call_site_data: i32,
    pub ic_data: i32,
    pub megamorphic_cache: i32,
    pub subtype_test_cache: i32,
    pub loading_unit: i32,
    pub language_error: i32,
    pub unhandled_exception: i32,
    pub instance: i32,
    pub library_prefix: i32,
    pub type_arguments: i32,
    pub type_: i32,
    pub function_type: i32,
    pub record_type: i32,
    pub type_parameter: i32,
    pub closure: i32,
    pub mint: i32,
    pub double: i32,
    pub float32x4: i32,
    pub int32x4: i32,
    pub float64x2: i32,
    pub record: i32,
    pub typed_data: i32,
    pub external_typed_data: i32,
    pub typed_data_view: i32,
    pub capability: i32,
    pub receive_port: i32,
    pub send_port: i32,
    pub stack_trace: i32,
    pub suspend_state: i32,
    pub regexp: i32,
    pub weak_property: i32,
    pub weak_reference: i32,
    pub future_or: i32,
    pub user_tag: i32,
    pub transferable_typed_data: i32,
    pub map: i32,
    pub const_map: i32,
    pub set: i32,
    pub const_set: i32,
    pub array: i32,
    pub immutable_array: i32,
    pub growable_array: i32,
    pub string: i32,
    pub one_byte_string: i32,
    pub two_byte_string: i32,
    pub typed_data_first: i32,
    pub byte_data_view: i32,
    pub predefined_count: i32,
}

pub fn profile_for(info: &SnapshotInfo, pointer_width: usize) -> Result<Profile> {
    let version = info
        .dart_version
        .as_deref()
        .or_else(|| info.profile_id.strip_prefix("dart-"))
        .unwrap_or_default();
    let minor = version
        .split('.')
        .nth(1)
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| {
            ClutterError::Unsupported(format!(
                "cannot select a clustered-snapshot profile for {}",
                info.profile_id
            ))
        })?;
    let compressed_pointers = info
        .isolate_header
        .features
        .iter()
        .any(|feature| feature == "compressed-pointers");
    let cids = match minor {
        4 | 5 => CIDS_343,
        6..=8 => CIDS_362,
        9..=12 => CIDS_392,
        _ => {
            return Err(ClutterError::Unsupported(format!(
                "Dart {version} clustered objects are outside the supported 3.4–3.12 range"
            )));
        }
    };
    let (instance_header_words, unboxed_word_u32_chunks) =
        serialized_instance_layout(pointer_width, compressed_pointers)?;
    Ok(Profile {
        cids,
        compressed_pointers,
        instance_header_words,
        unboxed_word_u32_chunks,
    })
}

fn serialized_instance_layout(
    pointer_width: usize,
    compressed_pointers: bool,
) -> Result<(usize, usize)> {
    if !matches!(pointer_width, 4 | 8) {
        return Err(ClutterError::Unsupported(format!(
            "unsupported target pointer width {pointer_width}"
        )));
    }
    // `Instance::NextFieldOffset()` is one target machine word. It occupies
    // two 32-bit compressed words on 64-bit compressed-pointer targets, but
    // only one word on native 32-bit and uncompressed 64-bit targets.
    let instance_header_words = pointer_width
        / if compressed_pointers {
            std::mem::size_of::<u32>()
        } else {
            pointer_width
        };
    // `WriteWordWith32BitWrites` emits one 32-bit chunk per target machine
    // word half, even when heap references themselves are compressed.
    let unboxed_word_u32_chunks = pointer_width / std::mem::size_of::<u32>();
    Ok((instance_header_words, unboxed_word_u32_chunks))
}

macro_rules! common_cids {
    (
        object_pool: $object_pool:expr,
        pc: $pc:expr,
        code_map: $code_map:expr,
        stack_maps: $stack_maps:expr,
        handlers: $handlers:expr,
        context: $context:expr,
        context_scope: $context_scope:expr,
        sentinel: $sentinel:expr,
        single_target: $single_target:expr,
        unlinked: $unlinked:expr,
        mono: $mono:expr,
        call_site: $call_site:expr,
        ic: $ic:expr,
        mega: $mega:expr,
        subtype: $subtype:expr,
        loading: $loading:expr,
        language_error: $language_error:expr,
        unhandled: $unhandled:expr,
        instance: $instance:expr,
        library_prefix: $library_prefix:expr,
        type_args: $type_args:expr,
        type: $type:expr,
        function_type: $function_type:expr,
        record_type: $record_type:expr,
        type_parameter: $type_parameter:expr,
        closure: $closure:expr,
        mint: $mint:expr,
        double: $double:expr,
        f32x4: $f32x4:expr,
        i32x4: $i32x4:expr,
        f64x2: $f64x2:expr,
        record: $record:expr,
        typed_data: $typed_data:expr,
        external_typed: $external_typed:expr,
        typed_view: $typed_view:expr,
        capability: $capability:expr,
        receive: $receive:expr,
        send: $send:expr,
        stack: $stack:expr,
        suspend: $suspend:expr,
        regexp: $regexp:expr,
        weak_property: $weak_property:expr,
        weak_reference: $weak_reference:expr,
        future_or: $future_or:expr,
        user_tag: $user_tag:expr,
        transferable: $transferable:expr,
        map: $map:expr,
        const_map: $const_map:expr,
        set: $set:expr,
        const_set: $const_set:expr,
        array: $array:expr,
        immutable_array: $immutable_array:expr,
        growable: $growable:expr,
        string: $string:expr,
        one_string: $one_string:expr,
        two_string: $two_string:expr,
        typed_first: $typed_first:expr,
        byte_view: $byte_view:expr,
        predefined: $predefined:expr
    ) => {
        Cids {
            class: 5,
            patch_class: 6,
            function: 7,
            type_parameters: 8,
            closure_data: 9,
            ffi_trampoline_data: 10,
            field: 11,
            script: 12,
            library: 13,
            namespace: 14,
            kernel_program_info: 15,
            weak_serialization_reference: 16,
            weak_array: 17,
            code: 18,
            object_pool: $object_pool,
            pc_descriptors: $pc,
            code_source_map: $code_map,
            compressed_stack_maps: $stack_maps,
            exception_handlers: $handlers,
            context: $context,
            context_scope: $context_scope,
            sentinel: $sentinel,
            single_target_cache: $single_target,
            unlinked_call: $unlinked,
            monomorphic_smiable_call: $mono,
            call_site_data: $call_site,
            ic_data: $ic,
            megamorphic_cache: $mega,
            subtype_test_cache: $subtype,
            loading_unit: $loading,
            language_error: $language_error,
            unhandled_exception: $unhandled,
            instance: $instance,
            library_prefix: $library_prefix,
            type_arguments: $type_args,
            type_: $type,
            function_type: $function_type,
            record_type: $record_type,
            type_parameter: $type_parameter,
            closure: $closure,
            mint: $mint,
            double: $double,
            float32x4: $f32x4,
            int32x4: $i32x4,
            float64x2: $f64x2,
            record: $record,
            typed_data: $typed_data,
            external_typed_data: $external_typed,
            typed_data_view: $typed_view,
            capability: $capability,
            receive_port: $receive,
            send_port: $send,
            stack_trace: $stack,
            suspend_state: $suspend,
            regexp: $regexp,
            weak_property: $weak_property,
            weak_reference: $weak_reference,
            future_or: $future_or,
            user_tag: $user_tag,
            transferable_typed_data: $transferable,
            map: $map,
            const_map: $const_map,
            set: $set,
            const_set: $const_set,
            array: $array,
            immutable_array: $immutable_array,
            growable_array: $growable,
            string: $string,
            one_byte_string: $one_string,
            two_byte_string: $two_string,
            typed_data_first: $typed_first,
            byte_data_view: $byte_view,
            predefined_count: $predefined,
        }
    };
}

const CIDS_343: Cids = common_cids!(
    object_pool: 22, pc: 23, code_map: 24, stack_maps: 25,
    handlers: 27, context: 28, context_scope: 29, sentinel: 30,
    single_target: 31, unlinked: 32, mono: 33, call_site: 34,
    ic: 35, mega: 36, subtype: 37, loading: 38,
    language_error: 41, unhandled: 42, instance: 44,
    library_prefix: 45, type_args: 46, type: 48, function_type: 49,
    record_type: 50, type_parameter: 51, closure: 56, mint: 60,
    double: 61, f32x4: 63, i32x4: 64, f64x2: 65, record: 66,
    typed_data: 68, external_typed: 69, typed_view: 70,
    capability: 73, receive: 74, send: 75, stack: 76, suspend: 77,
    regexp: 78, weak_property: 79, weak_reference: 80, future_or: 82,
    user_tag: 83, transferable: 84, map: 85, const_map: 86,
    set: 87, const_set: 88, array: 89, immutable_array: 90,
    growable: 91, string: 92, one_string: 93, two_string: 94,
    typed_first: 111, byte_view: 167, predefined: 174
);

const CIDS_362: Cids = common_cids!(
    object_pool: 23, pc: 24, code_map: 25, stack_maps: 26,
    handlers: 28, context: 29, context_scope: 30, sentinel: 31,
    single_target: 32, unlinked: 33, mono: 34, call_site: 35,
    ic: 36, mega: 37, subtype: 38, loading: 39,
    language_error: 42, unhandled: 43, instance: 45,
    library_prefix: 46, type_args: 47, type: 49, function_type: 50,
    record_type: 51, type_parameter: 52, closure: 57, mint: 61,
    double: 62, f32x4: 64, i32x4: 65, f64x2: 66, record: 67,
    typed_data: 69, external_typed: 70, typed_view: 71,
    capability: 74, receive: 75, send: 76, stack: 77, suspend: 78,
    regexp: 79, weak_property: 80, weak_reference: 81, future_or: 83,
    user_tag: 84, transferable: 85, map: 86, const_map: 87,
    set: 88, const_set: 89, array: 90, immutable_array: 91,
    growable: 92, string: 93, one_string: 94, two_string: 95,
    typed_first: 112, byte_view: 168, predefined: 175
);

const CIDS_392: Cids = common_cids!(
    object_pool: 23, pc: 24, code_map: 25, stack_maps: 26,
    handlers: 28, context: 29, context_scope: 30, sentinel: 31,
    single_target: 32, unlinked: 35, mono: 33, call_site: 34,
    ic: 36, mega: 37, subtype: 38, loading: 39,
    language_error: 42, unhandled: 43, instance: 45,
    library_prefix: 46, type_args: 47, type: 49, function_type: 50,
    record_type: 51, type_parameter: 52, closure: 57, mint: 61,
    double: 62, f32x4: 64, i32x4: 65, f64x2: 66, record: 67,
    typed_data: 69, external_typed: 70, typed_view: 71,
    capability: 74, receive: 75, send: 76, stack: 77, suspend: 78,
    regexp: 79, weak_property: 80, weak_reference: 81, future_or: 83,
    user_tag: 84, transferable: 85, map: 86, const_map: 87,
    set: 88, const_set: 89, array: 90, immutable_array: 91,
    growable: 92, string: 93, one_string: 94, two_string: 95,
    typed_first: 112, byte_view: 168, predefined: 175
);

#[cfg(test)]
pub(crate) fn test_cids() -> Cids {
    CIDS_392
}

#[cfg(test)]
mod tests {
    use super::serialized_instance_layout;

    #[test]
    fn uses_target_word_width_for_instance_headers_and_unboxed_values() {
        assert_eq!(serialized_instance_layout(4, false).unwrap(), (1, 1));
        assert_eq!(serialized_instance_layout(8, true).unwrap(), (2, 2));
        assert_eq!(serialized_instance_layout(8, false).unwrap(), (1, 2));
        assert!(serialized_instance_layout(16, false).is_err());
    }
}
