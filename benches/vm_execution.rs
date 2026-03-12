// Copyright 2020 Trezoa Maintainers <maintainers@trezoa.com>
//
// Licensed under the Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> or
// the MIT license <http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs::File, io::Read, sync::Arc};
use test_utils::create_vm;
#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
use trezoa_rbpf::{
    ebpf,
    memory_region::MemoryRegion,
    program::{FunctionRegistry, SBPFVersion},
    vm::Config,
};
use trezoa_rbpf::{
    elf::Executable, program::BuiltinProgram, verifier::RequisiteVerifier, vm::TestContextObject,
};

fn bench_init_interpreter_start(c: &mut Criterion) {
    let mut file = File::open("tests/elfs/rodata_section_sbpfv0.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();
    executable.verify::<RequisiteVerifier>().unwrap();
    let mut context_object = TestContextObject::default();
    create_vm!(
        vm,
        &executable,
        &mut context_object,
        stack,
        heap,
        Vec::new(),
        None
    );
    c.bench_function("init_interpreter_start", |b| {
        b.iter(|| {
            vm.context_object_pointer.remaining = 37;
            vm.execute_program(&executable, true).1.unwrap()
        })
    });
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_init_jit_start(c: &mut Criterion) {
    let mut file = File::open("tests/elfs/rodata_section_sbpfv0.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let mut executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();
    executable.verify::<RequisiteVerifier>().unwrap();
    executable.jit_compile().unwrap();
    let mut context_object = TestContextObject::default();
    create_vm!(
        vm,
        &executable,
        &mut context_object,
        stack,
        heap,
        Vec::new(),
        None
    );
    c.bench_function("init_jit_start", |b| {
        b.iter(|| {
            vm.context_object_pointer.remaining = 37;
            vm.execute_program(&executable, false).1.unwrap()
        })
    });
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter(
    c: &mut Criterion,
    group_name: &str,
    assembly: &str,
    config: Config,
    instruction_meter: u64,
    mem_size: usize,
) {
    let mut executable = trezoa_rbpf::assembler::assemble::<TestContextObject>(
        assembly,
        Arc::new(BuiltinProgram::new_loader(
            config,
            FunctionRegistry::default(),
        )),
    )
    .unwrap();
    executable.verify::<RequisiteVerifier>().unwrap();
    executable.jit_compile().unwrap();

    let mut group = c.benchmark_group(group_name);

    {
        let mut context_object = TestContextObject::default();
        let mut mem = vec![0u8; mem_size];
        let mem_region = MemoryRegion::new_writable(&mut mem, ebpf::MM_INPUT_START);
        create_vm!(
            vm,
            &executable,
            &mut context_object,
            stack,
            heap,
            vec![mem_region],
            None
        );
        group.bench_function("interpreter", |b| {
            b.iter(|| {
                vm.context_object_pointer.remaining = instruction_meter;
                let (count, result) = vm.execute_program(&executable, true);
                assert!(result.is_ok(), "{:?}", result);
                assert_eq!(count, instruction_meter);
            })
        });
    }

    {
        let mut context_object = TestContextObject::default();
        let mut mem = vec![0u8; mem_size];
        let mem_region = MemoryRegion::new_writable(&mut mem, ebpf::MM_INPUT_START);
        create_vm!(
            vm,
            &executable,
            &mut context_object,
            stack,
            heap,
            vec![mem_region],
            None
        );
        group.bench_function("jit", |b| {
            b.iter(|| {
                vm.context_object_pointer.remaining = instruction_meter;
                let (count, result) = vm.execute_program(&executable, false);
                assert!(result.is_ok(), "{:?}", result);
                assert_eq!(count, instruction_meter);
            })
        });
    }

    group.finish();
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter_address_translation(c: &mut Criterion) {
    bench_jit_vs_interpreter(
        c,
        "jit_vs_interpreter_address_translation",
        "
    ldxb r0, [r1]
    add r1, 1
    mov r0, r1
    and r0, 0xFFFFFF
    jlt r0, 0x20000, -5
    exit",
        Config::default(),
        655361,
        0x20000,
    );
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
static ADDRESS_TRANSLATION_STACK_CODE: &str = "
    mov r1, r2
    and r1, 4095
    mov r3, r10
    sub r3, r1
    add r3, -1
    ldxb r4, [r3]
    add r2, 1
    jlt r2, 0x10000, -8
    exit";

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter_address_translation_stack_fixed(c: &mut Criterion) {
    bench_jit_vs_interpreter(
        c,
        "jit_vs_interpreter_address_translation_stack_fixed",
        ADDRESS_TRANSLATION_STACK_CODE,
        Config {
            enabled_sbpf_versions: SBPFVersion::V0..=SBPFVersion::V0,
            ..Config::default()
        },
        524289,
        0,
    );
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter_address_translation_stack_dynamic(c: &mut Criterion) {
    bench_jit_vs_interpreter(
        c,
        "jit_vs_interpreter_address_translation_stack_dynamic",
        ADDRESS_TRANSLATION_STACK_CODE,
        Config::default(),
        524289,
        0,
    );
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter_empty_for_loop(c: &mut Criterion) {
    bench_jit_vs_interpreter(
        c,
        "jit_vs_interpreter_empty_for_loop",
        "
    mov r1, r2
    and r1, 1023
    add r2, 1
    jlt r2, 0x10000, -4
    exit",
        Config::default(),
        262145,
        0,
    );
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter_call_depth_fixed(c: &mut Criterion) {
    bench_jit_vs_interpreter(
        c,
        "jit_vs_interpreter_call_depth_fixed",
        "
    mov r6, 0
    add r6, 1
    mov r1, 18
    call function_foo
    jlt r6, 1024, -4
    exit
    function_foo:
    stw [r10-4], 0x11223344
    mov r6, r1
    jgt r6, 0, +1
    exit
    mov r1, r6
    add r1, -1
    call function_foo
    exit",
        Config {
            enabled_sbpf_versions: SBPFVersion::V0..=SBPFVersion::V0,
            ..Config::default()
        },
        137218,
        0,
    );
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_vs_interpreter_call_depth_dynamic(c: &mut Criterion) {
    bench_jit_vs_interpreter(
        c,
        "jit_vs_interpreter_call_depth_dynamic",
        "
    mov r6, 0
    add r6, 1
    mov r1, 18
    call function_foo
    jlt r6, 1024, -4
    exit
    function_foo:
    add r10, -64
    stw [r10+4], 0x11223344
    mov r6, r1
    jeq r6, 0, +3
    mov r1, r6
    add r1, -1
    call function_foo
    exit",
        Config::default(),
        156674,
        0,
    );
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
criterion_group!(
    benches,
    bench_init_interpreter_start,
    bench_init_jit_start,
    bench_jit_vs_interpreter_address_translation,
    bench_jit_vs_interpreter_address_translation_stack_fixed,
    bench_jit_vs_interpreter_address_translation_stack_dynamic,
    bench_jit_vs_interpreter_empty_for_loop,
    bench_jit_vs_interpreter_call_depth_fixed,
    bench_jit_vs_interpreter_call_depth_dynamic,
);
#[cfg(not(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64")))]
criterion_group!(benches, bench_init_interpreter_start);
criterion_main!(benches);
