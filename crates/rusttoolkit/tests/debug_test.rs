use rusttoolkit::for_each;

// Generate functions at module level - use the new %{x} syntax
for_each!([hello], |x| {
    fn %{x}() -> &'static str {
        "working"
    }
});

#[test]
fn debug_simple_replacement() {
    // This should call the generated function
    assert_eq!(hello(), "working");
}
