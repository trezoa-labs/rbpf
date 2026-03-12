// Copyright 2020 Trezoa Maintainers <maintainers@trezoa.com>
//
// Licensed under the Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> or
// the MIT license <http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs::File, io::Read, sync::Arc};
use test_utils::create_vm;
use trezoa_rbpf::{
    elf::Executable, program::BuiltinProgram, verifier::RequisiteVerifier, vm::TestContextObject,
};

fn bench_init_vm(c: &mut Criterion) {
    let mut file = File::open("tests/elfs/relative_call_sbpfv0.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();
    executable.verify::<RequisiteVerifier>().unwrap();
    c.bench_function("init_vm", |b| {
        b.iter(|| {
            let mut context_object = TestContextObject::default();
            create_vm!(
                _vm,
                &executable,
                &mut context_object,
                stack,
                heap,
                Vec::new(),
                None
            );
        })
    });
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
fn bench_jit_compile(c: &mut Criterion) {
    let mut file = File::open("tests/elfs/relative_call_sbpfv0.so").unwrap();
    let mut elf = Vec::new();
    file.read_to_end(&mut elf).unwrap();
    let mut executable =
        Executable::<TestContextObject>::from_elf(&elf, Arc::new(BuiltinProgram::new_mock()))
            .unwrap();
    executable.verify::<RequisiteVerifier>().unwrap();
    c.bench_function("jit_compile", |b| {
        b.iter(|| executable.jit_compile().unwrap())
    });
}

#[cfg(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64"))]
criterion_group!(benches, bench_init_vm, bench_jit_compile);
#[cfg(not(all(feature = "jit", not(target_os = "windows"), target_arch = "x86_64")))]
criterion_group!(benches, bench_init_vm);
criterion_main!(benches);
