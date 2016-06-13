extern crate pesty_php;

use pesty_php::*;

fn assert_comment(input: &str, is_end: bool) {
    let mut parser = Rdp::new(StringInput::new(input));
    assert!(parser.comment());
    assert_eq!(parser.end(), is_end);
}

#[test]
fn parse_expr_op() {
    assert_eq!(process_expr(r#"1+2"#), Expr::BinaryOp(Op::Add, Box::new(Expr::Int(1)), Box::new(Expr::Int(2))));
    assert_eq!(process_expr(r#"1+2*3"#), Expr::BinaryOp(Op::Add, Box::new(Expr::Int(1)), Box::new(Expr::BinaryOp(Op::Mul, Box::new(Expr::Int(2)), Box::new(Expr::Int(3))))));
    assert_eq!(process_expr(r#"2+$d**$c**$d"#), Expr::BinaryOp(Op::Add, Box::new(Expr::Int(2)),
        Box::new(Expr::BinaryOp(
            Op::Pow,
            Box::new(Expr::Variable("d".into())),
            Box::new(Expr::BinaryOp(Op::Pow, Box::new(Expr::Variable("c".into())), Box::new(Expr::Variable("d".into()))))
        ))
    ));
    assert_eq!(process_expr(r#"$g["a"]-$g["b"]/3"#), Expr::BinaryOp(
        Op::Sub,
        Box::new(Expr::ArrayIdx(Box::new(Expr::Variable("g".into())), vec![Expr::String("a".into())])),
        Box::new(Expr::BinaryOp(Op::Div, Box::new(Expr::ArrayIdx(Box::new(Expr::Variable("g".into())), vec![Expr::String("b".into())])), Box::new(Expr::Int(3))))
    ));
}

#[test]
fn parse_expr_logical() {
    assert_eq!(process_expr(r#"$a||$b"#), Expr::BinaryOp(Op::Or, Box::new(Expr::Variable("a".into())), Box::new(Expr::Variable("b".into()))));
    assert_eq!(process_expr(r#"$a&&true"#), Expr::BinaryOp(Op::And, Box::new(Expr::Variable("a".into())), Box::new(Expr::True)));
    assert_eq!(process_expr(r#"!$a"#), Expr::UnaryOp(Op::Not, Box::new(Expr::Variable("a".into()))));
}

#[test]
fn parse_expr_parens() {
    assert_eq!(process_expr(r#"(1+2)*3"#), Expr::BinaryOp(Op::Mul, Box::new(Expr::BinaryOp(Op::Add, Box::new(Expr::Int(1)), Box::new(Expr::Int(2)))), Box::new(Expr::Int(3))));
    assert_eq!(process_expr(r#"(true||false)&&true"#), Expr::BinaryOp(Op::And, Box::new(Expr::BinaryOp(Op::Or, Box::new(Expr::True), Box::new(Expr::False))), Box::new(Expr::True)));
}

#[test]
fn parse_expr_string() {
    assert_eq!(process_expr(r#""t\nest\tsss\"os\"haha""#), Expr::String("t\nest\tsss\"os\"haha".into()));
    assert_eq!(process_expr(r#""\xe7\x9a\x84""#), Expr::String("的".into()));
    assert_eq!(process_expr(r#""a\142\143d""#), Expr::String("abcd".into()));
    assert_eq!(process_expr(r#""a\"b\\\"c\\\"d\"e""#), Expr::String(r#"a"b\"c\"d"e"#.into()));
}

#[test]
fn parse_expr_char_string() {
    assert_eq!(process_expr(r#"'\ntest\142'"#), Expr::String("\\ntest\\142".into()));
    assert_eq!(process_expr(r#"'a\'b\'c'"#), Expr::String("a'b'c".into()));
    assert_eq!(process_expr(r#"'d\'e\\\'f\\\'\'g'"#), Expr::String("d\'e\\\'f\\\'\'g".into()));
}

#[test]
fn parse_expr_array_idx() {
    assert_eq!(process_expr(r#"$test["a"]"#), Expr::ArrayIdx(Box::new(Expr::Variable("test".into())), vec![Expr::String("a".into())]));
    assert_eq!(process_expr(r#"$test["a"]['b\n']"#), Expr::ArrayIdx(Box::new(Expr::Variable("test".into())), vec![
        Expr::String("a".into()), Expr::String("b\\n".into())
    ]));
    assert_eq!(process_expr(r#"$test[$g["a"]]["b"]["c"]"#), Expr::ArrayIdx(Box::new(Expr::Variable("test".into())), vec![
        Expr::ArrayIdx(Box::new(Expr::Variable("g".into())), vec![Expr::String("a".into())]),
        Expr::String("b".into()),
        Expr::String("c".into())
    ]));
}

#[test]
fn parse_expr_func_call() {
    assert_eq!(process_expr(r#"func_x(1, 2)"#), Expr::Call(Box::new(Expr::Identifier("func_x".into())), vec![Expr::Int(1), Expr::Int(2)]));
    assert_eq!(process_expr(r#"func_x(abc(1), 2)"#), Expr::Call(Box::new(Expr::Identifier("func_x".into())), vec![
        Expr::Call(Box::new(Expr::Identifier("abc".into())), vec![Expr::Int(1)]),
        Expr::Int(2)
    ]));
    assert_eq!(process_expr(r#"$g[0]()"#), Expr::Call(Box::new(Expr::ArrayIdx(Box::new(Expr::Variable("g".into())), vec![Expr::Int(0)])), vec![]));
    assert_eq!(process_expr(r#"$g[0]()[1](true)"#), Expr::Call(
        Box::new(Expr::ArrayIdx(
            Box::new(Expr::Call(
                Box::new(Expr::ArrayIdx(Box::new(Expr::Variable("g".into())), vec![Expr::Int(0)])),
                vec![]
            )), vec![Expr::Int(1)]
        )), vec![Expr::True]
    ));
}

#[test]
fn parse_expr_object_property() {
    assert_eq!(process_expr(r#"$obj->prop"#), Expr::ObjProperty(Box::new(Expr::Variable("obj".into())), vec![Expr::Identifier("prop".into())]));
    assert_eq!(process_expr(r#"$obj->$a->b"#), Expr::ObjProperty(Box::new(Expr::Variable("obj".into())), vec![Expr::Variable("a".into()), Expr::Identifier("b".into())]));
    assert_eq!(process_expr(r#"$obj->$a->b()"#), Expr::Call(Box::new(Expr::ObjProperty(
        Box::new(Expr::Variable("obj".into())),
        vec![Expr::Variable("a".into()), Expr::Identifier("b".into())]
    )), vec![]));
}

#[test]
fn parse_expr_static_property() {
    assert_eq!(process_expr(r#"Obj::$test"#), Expr::StaticProperty(Box::new(Expr::Identifier("Obj".into())), vec![Expr::Variable("test".into())]));
}

#[test]
fn parse_expr_comment() {
    assert_comment("//test", true);
    assert_comment("/*test*/", true);
    assert_comment("//test\ns", false);
    assert_comment("/*test*/s", false);
}

#[test]
fn parse_post_pre_dec_inc() {
    assert_eq!(process_expr("$c++"), Expr::UnaryOp(Op::PostInc, Box::new(Expr::Variable("c".into()))));
    assert_eq!(process_expr("$c--"), Expr::UnaryOp(Op::PostDec, Box::new(Expr::Variable("c".into()))));
    assert_eq!(process_expr("++$c"), Expr::UnaryOp(Op::PreInc, Box::new(Expr::Variable("c".into()))));
    assert_eq!(process_expr("--$c"), Expr::UnaryOp(Op::PreDec, Box::new(Expr::Variable("c".into()))));
}

#[test]
fn parse_closure() {
    assert_eq!(process_expr("function () { c(); }"), Expr::Function(FunctionDecl {
        params: vec![],
        body: vec![Expr::Call(Box::new(Expr::Identifier("c".into())), vec![])]
    }));
}
