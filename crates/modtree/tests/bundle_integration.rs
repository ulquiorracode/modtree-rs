#![cfg(feature = "macros")]

use modtree::prelude::*;

#[derive(Default)]
pub struct AppContext {
    pub auth: bool,
    pub audited: bool,
}

pub fn require_auth(ctx: &mut AppContext, intent: u32) -> FlowControl<u32, u32, &'static str> {
    if ctx.auth {
        FlowControl::Proceed(intent)
    } else {
        FlowControl::Halt("unauthorized")
    }
}

pub fn audit_exit(
    ctx: &mut AppContext,
    _intent: &u32,
    _res: &mut Result<u32, RoutingError<&'static str>>,
) {
    ctx.audited = true;
}

#[derive(Clone, Default)]
pub struct AuthLibrary;

#[modtree::library(
    name = "auth",
    context(AppContext),
    intent(u32),
    outcome(u32),
    error(&'static str)
)]
impl AuthLibrary {
    #[modtree::command]
    pub fn login(_ctx: &mut AppContext, token: u32) -> Result<u32, &'static str> {
        Ok(token * 2)
    }
}

#[derive(Clone, Default)]
pub struct PaymentLibrary;

#[modtree::library(
    name = "billing",
    context(AppContext),
    intent(u32),
    outcome(u32),
    error(&'static str)
)]
impl PaymentLibrary {
    #[modtree::command]
    pub fn pay(_ctx: &mut AppContext, amount: u32) -> Result<u32, &'static str> {
        Ok(amount + 10)
    }
}

#[derive(Clone, Default)]
#[modtree::bundle(
    name = "security_suite",
    version = "1.0.0",
    description = "Security & Billing bundle suite",
    author = "GoldSrc Architecture Team",
    context(AppContext),
    intent(u32),
    outcome(u32),
    error(&'static str)
)]
pub struct SecurityBundle {
    #[modtree::mount("system/security")]
    pub auth_lib: AuthLibrary,
    #[modtree::mount("services/finance")]
    pub billing: PaymentLibrary,
}

#[test]
fn test_macro_bundle_installation_and_dispatch() {
    let mut root = ModuleNode::<AppContext, u32, u32, &'static str>::new("root")
        .with_enter(require_auth)
        .with_exit(audit_exit);

    let bundle = SecurityBundle {
        auth_lib: AuthLibrary,
        billing: PaymentLibrary,
    };

    let desc = bundle.descriptor();
    assert_eq!(desc.name, "security_suite");
    assert_eq!(desc.version, "1.0.0");
    assert_eq!(desc.description, "Security & Billing bundle suite");
    assert_eq!(desc.author, "GoldSrc Architecture Team");

    // Install bundle into root
    bundle
        .install(&mut root)
        .expect("bundle installation must succeed");

    let root_node = TreeNode::Module(root);

    let mut ctx = AppContext {
        auth: true,
        audited: false,
    };

    // Dispatch through the mounted auth node
    let res = dispatch(
        &root_node,
        &["root", "system", "security", "auth", "login"],
        &mut ctx,
        21,
    );
    assert_eq!(res, Ok(42));
    assert!(ctx.audited);

    // Dispatch through the mounted billing node
    ctx.audited = false;
    let res_billing = dispatch(
        &root_node,
        &["root", "services", "finance", "billing", "pay"],
        &mut ctx,
        100,
    );
    assert_eq!(res_billing, Ok(110));
    assert!(ctx.audited);
}

#[test]
fn test_bundle_conflict_rejection() {
    let mut root = ModuleNode::<AppContext, u32, u32, &'static str>::new("root");

    let bundle = SecurityBundle {
        auth_lib: AuthLibrary,
        billing: PaymentLibrary,
    };

    bundle
        .install(&mut root)
        .expect("initial installation succeeds");

    // Attempting to install again with identical paths must fail with MountConflict
    let err = bundle.install(&mut root);
    match err {
        Err(BundleError::MountConflict { target, .. }) => {
            assert_eq!(target, "auth");
        }
        other => panic!("expected MountConflict error, got: {:?}", other),
    }
}
