mod harness;
use fuel_vm::prelude::*;

pub fn run(filter_regex: Option<regex::Regex>) {
    let filter = |name| {
        filter_regex
            .as_ref()
            .map(|regex| regex.is_match(name))
            .unwrap_or(true)
    };

    // programs that should successfully compile and terminate
    // with some known state
    let project_names = vec![
        ("asm_expr_basic", ProgramState::Return(6)),
        ("basic_func_decl", ProgramState::Return(1)), // 1 == true
        ("contract_abi_impl", ProgramState::Return(0)),
        // TEMPORARILY DISABLED DUE TO OOM ("dependencies", ProgramState::Return(0)), // 0 == false
        ("if_elseif_enum", ProgramState::Return(10)),
        ("tuple_types", ProgramState::Return(123)),
        ("out_of_order_decl", ProgramState::Return(1)),
        ("struct_field_reassignment", ProgramState::Return(0)),
        ("enum_in_fn_decl", ProgramState::Return(255)),
        ("empty_impl", ProgramState::Return(0)),
        ("main_returns_unit", ProgramState::Return(0)),
        ("unary_not_basic", ProgramState::Return(1)), // 1 == true
        ("unary_not_basic_2", ProgramState::Return(1)), // 1 == true
        ("fix_opcode_bug", ProgramState::Return(30)),
        (
            "retd_b256",
            ProgramState::ReturnData(Bytes32::from([
                102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32, 8,
                151, 20, 133, 110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41, 37,
            ])),
        ),
        (
            "retd_struct",
            ProgramState::ReturnData(Bytes32::from([
                251, 57, 24, 241, 63, 94, 17, 102, 252, 182, 8, 110, 140, 105, 102, 105, 138, 202,
                155, 39, 97, 32, 94, 129, 141, 144, 190, 142, 33, 32, 33, 75,
            ])),
        ),
        ("op_precedence", ProgramState::Return(0)),
        ("asm_without_return", ProgramState::Return(0)),
        ("op_precedence", ProgramState::Return(0)), // false
        ("b256_bad_jumps", ProgramState::Return(1)),
        ("b256_ops", ProgramState::Return(100)),
        ("struct_field_access", ProgramState::Return(43)),
        ("bool_and_or", ProgramState::Return(42)),
        ("neq_4_test", ProgramState::Return(0)),
        ("eq_4_test", ProgramState::Return(1)),
        ("local_impl_for_ord", ProgramState::Return(1)), // true
        ("const_decl", ProgramState::Return(100)),
        // TEMPORARILY DISABLED DUE TO OOM ("const_decl_in_library", ProgramState::Return(1)), // true
        ("aliased_imports", ProgramState::Return(42)),
        ("empty_method_initializer", ProgramState::Return(1)), // true
        ("b512_struct_alignment", ProgramState::Return(1)),    // true
        ("generic_structs", ProgramState::Return(1)),          // true
        ("generic_functions", ProgramState::Return(1)),        // true
        ("generic_enum", ProgramState::Return(1)),             // true
        ("import_method_from_other_file", ProgramState::Return(10)), // true
        ("address_test", ProgramState::Return(1)),             // true
        ("generic_struct", ProgramState::Return(1)),           // true
        ("zero_field_types", ProgramState::Return(10)),        // true
        ("assert_test", ProgramState::Return(1)),              // true
        ("match_expressions", ProgramState::Return(42)),
        ("assert_test", ProgramState::Return(1)),  // true
        ("array_basics", ProgramState::Return(1)), // true
        ("array_dynamic_oob", ProgramState::Revert(1)),
        ("array_generics", ProgramState::Return(1)), // true
        ("match_expressions_structs", ProgramState::Return(4)),
        ("block_height", ProgramState::Return(1)),   // true
        ("b512_test", ProgramState::Return(1)),      // true
        ("block_height", ProgramState::Return(1)),   // true
        ("valid_impurity", ProgramState::Return(0)), // false
        ("trait_override_bug", ProgramState::Return(7)),
        ("if_implicit_unit", ProgramState::Return(0)),
        ("modulo_uint_test", ProgramState::Return(1)), // true
        ("trait_import_with_star", ProgramState::Return(0)),
        ("tuple_desugaring", ProgramState::Return(9)),
    ];

    project_names.into_iter().for_each(|(name, res)| {
        if filter(name) {
            assert_eq!(crate::e2e_vm_tests::harness::runs_in_vm(name), res);
            assert_eq!(crate::e2e_vm_tests::harness::test_json_abi(name), Ok(()));
        }
    });

    // source code that should _not_ compile
    let project_names = vec![
        "recursive_calls",
        "asm_missing_return",
        "asm_should_not_have_return",
        "missing_fn_arguments",
        "excess_fn_arguments",
        // the feature for the below test, detecting inf deps, was reverted
        // when that is re-implemented we should reenable this test
        //"infinite_dependencies",
        "top_level_vars",
        "dependencies_parsing_error",
        "disallowed_gm",
        "bad_generic_annotation",
        "bad_generic_var_annotation",
        "unify_identical_unknowns",
        "array_oob",
        "array_bad_index",
        "name_shadowing",
        "match_expressions_wrong_struct",
        "match_expressions_enums",
        "pure_calls_impure",
        "nested_impure",
        "predicate_calls_impure",
        "script_calls_impure",
        "contract_pure_calls_impure",
    ];
    project_names.into_iter().for_each(|name| {
        if filter(name) {
            crate::e2e_vm_tests::harness::does_not_compile(name)
        }
    });

    // ---- Tests paired with contracts upon which they depend which must be pre-deployed.
    // TODO validate that call output is correct
    let contract_and_project_names = &[
        ("basic_storage", "call_basic_storage"),
        ("increment_contract", "call_increment_contract"),
        ("auth_testing_contract", "caller_auth_test"),
        ("context_testing_contract", "caller_context_test"),
        ("contract_abi_impl", "contract_call"),
        ("balance_test_contract", "bal_opcode"),
    ];

    // Filter them first.
    let (contracts, projects): (Vec<_>, Vec<_>) = contract_and_project_names
        .iter()
        .filter(|names| filter(names.1))
        .cloned()
        .unzip();

    // Deploy and then test.
    for name in contracts {
        harness::deploy_contract(name)
    }
    for name in projects {
        harness::runs_on_node(name);
    }

    println!("_________________________________\nTests passed.");
}
