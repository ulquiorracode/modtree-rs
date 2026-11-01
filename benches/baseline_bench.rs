use criterion::{Criterion, black_box, criterion_group, criterion_main};
use modtree::prelude::*;

struct BenchContext {
    auth_level: u32,
    audit_count: u64,
}

fn benchmark_u_cycle_dispatch(c: &mut Criterion) {
    let ban_leaf = LeafNode::new(
        "ban",
        |_ctx: &mut BenchContext, (user_id, reason_code): (u64, u32)| {
            Ok(user_id ^ (reason_code as u64))
        },
    );

    let users_lib = LibraryNode::new("users").with_leaf(ban_leaf);

    let admin_mod = ModuleNode::new("admin")
        .with_child(TreeNode::Library(users_lib))
        .with_enter(|ctx, intent| {
            if ctx.auth_level >= 10 {
                FlowControl::Proceed(intent)
            } else {
                FlowControl::Halt("unauthorized")
            }
        })
        .with_exit(|ctx, _intent, _outcome| {
            ctx.audit_count += 1;
        });

    let root = TreeNode::Module(ModuleNode::new("root").with_child(TreeNode::Module(admin_mod)));

    let mut ctx = BenchContext {
        auth_level: 10,
        audit_count: 0,
    };

    let path = ["root", "admin", "users", "ban"];

    c.bench_function("u_cycle_4_tier_traversal", |b| {
        b.iter(|| {
            let res = dispatch(
                &root,
                black_box(&path),
                &mut ctx,
                black_box((1001u64, 42u32)),
            );
            black_box(res)
        });
    });
}

criterion_group!(benches, benchmark_u_cycle_dispatch);
criterion_main!(benches);
