extern crate pesty_php;

use pesty_php::*;

#[test]
fn parse_stmt_echo() {
    assert_eq!(process_stmt("echo 1;"), Expr::Echo(vec![Expr::Int(1)]));
}

#[test]
fn parse_stmt_return() {
    assert_eq!(process_stmt("return true;"), Expr::Return(Box::new(Expr::True)));
    assert_eq!(process_stmt("return;"), Expr::Return(Box::new(Expr::None)));
}

#[test]
fn parse_stmt_assignment() {
    assert_eq!(process_stmt(r#"$test=4;"#), Expr::Assign(Box::new(Expr::Variable("test".into())), Box::new(Expr::Int(4))));
    assert_eq!(process_stmt(r#"$test["a"]=4+$b;"#), Expr::Assign(Box::new(
        Expr::ArrayIdx(
            Box::new(Expr::Variable("test".into())),
            vec![Expr::String("a".into())]
        )),
        Box::new(Expr::BinaryOp(Op::Add, Box::new(Expr::Int(4)), Box::new(Expr::Variable("b".into()))))
    ));
}

#[test]
fn parse_stmt_if_while() {
    let expr_var = Expr::Variable("a".into());
    let block_stmt = Expr::Call(Box::new(Expr::Identifier("b".into())), vec![]);
    let block_expr = Expr::Block(vec![block_stmt.clone()]);
    let args = [
        ("if", Expr::If(Box::new(expr_var.clone()), Box::new(block_expr.clone()), Box::new(Expr::None))),
        ("while", Expr::While(Box::new(expr_var.clone()), Box::new(block_expr)))
    ];
    for arg in &args {
        assert_eq!(process_stmt(&(arg.0.to_owned() + r#" ($a) { b(); }"#)), arg.1);
    }
    let args = [
        ("if", Expr::If(Box::new(expr_var.clone()), Box::new(block_stmt.clone()), Box::new(Expr::None))),
        ("while", Expr::While(Box::new(expr_var.clone()), Box::new(block_stmt)))
    ];
    for arg in &args {
        assert_eq!(process_stmt(&(arg.0.to_owned() + r#"($a) b();"#)), arg.1);
    }
}

#[test]
fn parse_stmt_if_else() {
    let make_result = |block_expr, else_expr| Expr::If(Box::new(Expr::Variable("a".into())), Box::new(block_expr), Box::new(else_expr));
    let main_call_expr = Expr::Call(Box::new(Expr::Identifier("a".into())), vec![]);
    let block_expr = Expr::Block(vec![main_call_expr.clone()]);
    let call_expr = Expr::Call(Box::new(Expr::Identifier("b".into())), vec![]);
    let else_expr = Expr::Block(vec![call_expr.clone()]);

    assert_eq!(process_stmt("if ($a) { a(); } else { b(); }"), make_result(block_expr, else_expr));
    assert_eq!(process_stmt("if ($a) a(); else b();"), make_result(main_call_expr, call_expr));
    assert_eq!(process_stmt("if ($a) a(); else if ($b) b(); else c();"), Expr::If(
        Box::new(Expr::Variable("a".into())),
        Box::new(Expr::Call(Box::new(Expr::Identifier("a".into())), vec![])),
        Box::new(
            Expr::If(Box::new(Expr::Variable("b".into())),
                Box::new(Expr::Call(Box::new(Expr::Identifier("b".into())), vec![])),
                Box::new(Expr::Call(Box::new(Expr::Identifier("c".into())), vec![]))
            )
        )
    ));
}

#[test]
fn parse_stmt_do_while() {
    assert_eq!(process_stmt("do { test(); } while(count($a));"), Expr::DoWhile(
        Box::new(Expr::Block(vec![Expr::Call(Box::new(Expr::Identifier("test".into())), vec![])])),
        Box::new(Expr::Call(Box::new(Expr::Identifier("count".into())), vec![Expr::Variable("a".into())]))
    ));
}

#[test]
fn parse_stmt_foreach() {
    assert_eq!(process_stmt("foreach ($test as $v) { ok(); }"), Expr::ForEach(
        Box::new(Expr::Variable("test".into())),
        Box::new(Expr::None), // key
        Box::new(Expr::Variable("v".into())), // value
        Box::new(Expr::Block(vec![Expr::Call(Box::new(Expr::Identifier("ok".into())), vec![])])) //body
    ));
    assert_eq!(process_stmt("foreach ($test as $k => $v) { ok(); }"), Expr::ForEach(
        Box::new(Expr::Variable("test".into())),
        Box::new(Expr::Variable("k".into())), // key
        Box::new(Expr::Variable("v".into())), // value
        Box::new(Expr::Block(vec![Expr::Call(Box::new(Expr::Identifier("ok".into())), vec![])])) //body
    ));
}

#[test]
fn parse_func_decl() {
    assert_eq!(process_stmt("function test() { ok(); }"), Expr::Decl(Decl::GlobalFunction("test".into(), FunctionDecl { params: vec![],
        body: vec![Expr::Call(Box::new(Expr::Identifier("ok".into())), vec![])] })
    ));
    assert_eq!(process_stmt("function test($a) { ok(); }"), Expr::Decl(Decl::GlobalFunction("test".into(), FunctionDecl {
        params: vec![ParamDefinition { name: "a".into(), as_ref: false }],
        body: vec![Expr::Call(Box::new(Expr::Identifier("ok".into())), vec![])] })
    ));
    assert_eq!(process_stmt("function test($a, $b) { ok(); }"), Expr::Decl(Decl::GlobalFunction("test".into(), FunctionDecl {
        params: vec![ParamDefinition { name: "a".into(), as_ref: false }, ParamDefinition { name: "b".into(), as_ref: false }],
        body: vec![Expr::Call(Box::new(Expr::Identifier("ok".into())), vec![])] })
    ));
}

#[test]
fn parse_class_decl() {
    assert_eq!(process_stmt("class Test {}"), Expr::Decl(Decl::Class(ClassDecl {
        name: "Test".into(), base_class: None, members: vec![]
    })));
    assert_eq!(process_stmt("class Test extends Abc\\Test2 {}"), Expr::Decl(Decl::Class(ClassDecl {
        name: "Test".into(), base_class: Some(Path::NamespacedClass("Abc".into(), "Test2".into())), members: vec![]
    })));
}

#[test]
fn parse_class_properties() {
    assert_eq!(process_stmt("class Test { public $test; }"), Expr::Decl(Decl::Class(ClassDecl {
        name: "Test".into(), base_class: None,
        members: vec![ClassMember::Property(Modifiers(false, Visibility::Public, ClassModifier::None), "test".into(), Expr::None)]
    })));
}

#[test]
fn parse_class_methods() {
    assert_eq!(process_stmt("class Test { public function a() { run(); } }"), Expr::Decl(Decl::Class(ClassDecl {
        name: "Test".into(), base_class: None,
        members: vec![ClassMember::Method(Modifiers(false, Visibility::Public, ClassModifier::None), "a".into(), FunctionDecl {
            params: vec![], body: vec![Expr::Call(Box::new(Expr::Identifier("run".into())), vec![])]
        })]
    })));
}

#[test]
fn parse_namespace_decl() {
    assert_eq!(process_stmt("namespace Foo\\Bar;"), Expr::Decl(Decl::Namespace(vec!["Foo".into(), "Bar".into()])));
}

#[test]
fn parse_use_statement() {
    assert_eq!(process_stmt("use Test;"), Expr::Use(vec![UseClause::QualifiedName(vec!["Test".into()]) ]));
}

#[test]
fn parse_continue_statement() {
    assert_eq!(process_stmt("continue;"), Expr::Continue(1));
    assert_eq!(process_stmt("continue 2;"), Expr::Continue(2));
}

#[test]
fn parse_break_statement() {
    assert_eq!(process_stmt("break;"), Expr::Break(1));
    assert_eq!(process_stmt("break 2;"), Expr::Break(2));
}

#[test]
fn parse_switch_statement() {
    assert_eq!(process_stmt(r#"switch ($test) { case 1: echo "1"; break; default: echo "2"; }"#), Expr::Switch(Box::new(Expr::Variable("test".into())),
        vec![ (vec![Expr::Int(1)], Expr::Block(vec![Expr::Echo(vec![Expr::String("1".into())]), Expr::Break(1)])), (vec![Expr::None], Expr::Echo(vec![Expr::String("2".into())])) ]));
    assert_eq!(process_stmt(r#"switch ($test) { case 1: echo "1"; default: echo "2"; }"#), Expr::Switch(Box::new(Expr::Variable("test".into())),
        vec![ (vec![Expr::Int(1)], Expr::Echo(vec![Expr::String("1".into())])), (vec![Expr::None], Expr::Echo(vec![Expr::String("2".into())])) ]));
    assert_eq!(process_stmt("switch ($test) { case 1: case 2: echo 1; }"), Expr::Switch(Box::new(Expr::Variable("test".into())),
        vec![(vec![Expr::Int(1), Expr::Int(2)], Expr::Echo(vec![Expr::Int(1)])) ]));
    assert_eq!(process_stmt("switch ($test) { case 1: case 2: case 3: case 4: echo 1; }"), Expr::Switch(Box::new(Expr::Variable("test".into())),
        vec![(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3), Expr::Int(4)], Expr::Echo(vec![Expr::Int(1)])) ]));
}
