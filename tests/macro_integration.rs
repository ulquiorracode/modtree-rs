#![cfg(feature = "macros")]

use modtree::prelude::*;

#[derive(Default)]
pub struct AdminContext {
    pub role: &'static str,
    pub audited: bool,
}

pub fn check_admin(
    ctx: &mut AdminContext,
    intent: (u64, u32),
) -> FlowControl<(u64, u32), u64, &'static str> {
    if ctx.role == "admin" {
        FlowControl::Proceed(intent)
    } else {
        FlowControl::Halt("forbidden")
    }
}

pub fn audit_admin(
    ctx: &mut AdminContext,
    _intent: &(u64, u32),
    _res: &mut Result<u64, RoutingError<&'static str>>,
) {
    ctx.audited = true;
}

#[derive(Clone, Default)]
pub struct UsersLibrary;

#[modtree::library(
    name = "users",
    context(AdminContext),
    intent((u64, u32)),
    outcome(u64),
    error(&'static str)
)]
impl UsersLibrary {
    #[modtree::command]
    pub fn ban(_ctx: &mut AdminContext, (user_id, code): (u64, u32)) -> Result<u64, &'static str> {
        Ok(user_id + (code as u64))
    }
}

#[modtree::module(
    name = "root",
    context(AdminContext),
    intent((u64, u32)),
    outcome(u64),
    error(&'static str),
    enter = check_admin,
    exit = audit_admin
)]
pub struct RootModule {
    pub users: UsersLibrary,
}

#[modtree::module]
#[modtree::name("granular_root")]
#[modtree::context(AdminContext)]
#[modtree::intent((u64, u32))]
#[modtree::outcome(u64)]
#[modtree::error(&'static str)]
#[modtree::enter(check_admin)]
#[modtree::exit(audit_admin)]
pub struct GranularRootModule {
    pub users: UsersLibrary,
}

#[test]
fn test_macro_attribute_tree_generation() {
    let mut ctx = AdminContext {
        role: "admin",
        audited: false,
    };

    let root_mod = RootModule {
        users: UsersLibrary,
    };

    let root_node = root_mod.into_tree_node();
    assert_eq!(root_node.name(), "root");

    let res = dispatch(&root_node, &["root", "users", "ban"], &mut ctx, (1000, 42));
    assert_eq!(res, Ok(1042));
    assert!(ctx.audited);

    // Unauthorized scenario
    ctx.role = "guest";
    ctx.audited = false;
    let res2 = dispatch(&root_node, &["root", "users", "ban"], &mut ctx, (1000, 42));
    assert_eq!(res2, Err(RoutingError::Halted("forbidden")));
    assert!(ctx.audited);
}

#[test]
fn test_granular_attribute_tree_generation() {
    let mut ctx = AdminContext {
        role: "admin",
        audited: false,
    };

    let granular_mod = GranularRootModule {
        users: UsersLibrary,
    };

    let root_node = granular_mod.into_tree_node();
    assert_eq!(root_node.name(), "granular_root");

    let res = dispatch(
        &root_node,
        &["granular_root", "users", "ban"],
        &mut ctx,
        (2000, 5),
    );
    assert_eq!(res, Ok(2005));
    assert!(ctx.audited);
}
