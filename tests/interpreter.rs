#[cfg(test)]
mod tests {

    use v8::Interpreter;

    // Helper function to run source and capture output
    fn run_and_capture(source: &str) -> (String, String) {
        let mut interpreter = Interpreter::new().setup();

        // This is pseudocode - you'd need to inject capture into your console.log implementation
        println!("source: {source}");
        let (out, err) = interpreter.interpret(source).unwrap();

        (out, err)
    }

    // Simpler helper that just runs and returns stdout
    fn run(source: &str) -> String {
        run_and_capture(source).0
    }

    // ==========================================================================
    // LITERALS AND BASIC EXPRESSIONS
    // ==========================================================================

    #[test]
    fn test_number_literals() {
        assert_eq!(run("console.log(42);"), "42\n");
        assert_eq!(run("console.log(3.14);"), "3.14\n");
        assert_eq!(run("console.log(0);"), "0\n");
    }

    #[test]
    fn test_string_literals() {
        assert_eq!(run("console.log('hello');"), "hello\n");
        assert_eq!(run("console.log(\"world\");"), "world\n");
        assert_eq!(run("console.log('');"), "\n");
    }

    #[test]
    fn test_boolean_literals() {
        assert_eq!(run("console.log(true);"), "true\n");
        assert_eq!(run("console.log(false);"), "false\n");
    }

    #[test]
    fn test_null_undefined() {
        assert_eq!(run("console.log(null);"), "null\n");
        assert_eq!(run("console.log(undefined);"), "undefined\n");
    }

    // // ==========================================================================
    // // ARITHMETIC OPERATORS
    // // ==========================================================================

    #[test]
    fn test_addition() {
        assert_eq!(run("console.log(2 + 3);"), "5\n");
        assert_eq!(run("console.log(10 + 20 + 30);"), "60\n");
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(run("console.log(10 - 3);"), "7\n");
        assert_eq!(run("console.log(5 - 10);"), "-5\n");
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(run("console.log(4 * 5);"), "20\n");
        assert_eq!(run("console.log(3 * 2 * 2);"), "12\n");
    }

    #[test]
    fn test_division() {
        assert_eq!(run("console.log(20 / 4);"), "5\n");
        assert_eq!(run("console.log(7 / 2);"), "3.5\n");
    }

    #[test]
    fn test_modulo() {
        assert_eq!(run("console.log(10 % 3);"), "1\n");
        assert_eq!(run("console.log(20 % 7);"), "6\n");
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(run("console.log(2 + 3 * 4);"), "14\n");
        assert_eq!(run("console.log((2 + 3) * 4);"), "20\n");
        assert_eq!(run("console.log(10 - 2 * 3);"), "4\n");
    }

    #[test]
    fn test_unary_operators() {
        assert_eq!(run("console.log(-5);"), "-5\n");
        assert_eq!(run("console.log(!true);"), "false\n");
        assert_eq!(run("console.log(!false);"), "true\n");
    }

    // // ==========================================================================
    // // STRING CONCATENATION
    // // ==========================================================================

    #[test]
    fn test_string_concatenation() {
        assert_eq!(
            run("console.log('hello' + ' ' + 'world');"),
            "hello world\n"
        );
        assert_eq!(run("console.log('num: ' + 42);"), "num: 42\n");
        assert_eq!(run("console.log(5 + '3');"), "53\n");
    }

    // // ==========================================================================
    // // COMPARISON OPERATORS
    // // ==========================================================================

    #[test]
    fn test_equality() {
        assert_eq!(run("console.log(5 == 5);"), "true\n");
        assert_eq!(run("console.log(5 == 3);"), "false\n");
        assert_eq!(run("console.log(5 != 3);"), "true\n");
        assert_eq!(run("console.log(5 != 5);"), "false\n");
    }

    #[test]
    fn test_strict_equality() {
        assert_eq!(run("console.log(5 === 5);"), "true\n");
        assert_eq!(run("console.log(5 === 3);"), "false\n");
        assert_eq!(run("console.log(5 === '5');"), "false\n");
        assert_eq!(run("console.log(5 !== '5');"), "true\n");
        assert_eq!(run("console.log(5 !== '3');"), "true\n");
        assert_eq!(run("console.log(5 !== 3);"), "true\n");
    }

    #[test]
    fn test_comparison() {
        assert_eq!(run("console.log(5 > 3);"), "true\n");
        assert_eq!(run("console.log(5 < 3);"), "false\n");
        assert_eq!(run("console.log(5 >= 5);"), "true\n");
        assert_eq!(run("console.log(5 <= 3);"), "false\n");
    }

    // // ==========================================================================
    // // LOGICAL OPERATORS
    // // ==========================================================================

    #[test]
    fn test_logical_and() {
        assert_eq!(run("console.log(true && true);"), "true\n");
        assert_eq!(run("console.log(true && false);"), "false\n");
        assert_eq!(run("console.log(false && true);"), "false\n");
    }

    #[test]
    fn test_logical_or() {
        assert_eq!(run("console.log(true || false);"), "true\n");
        assert_eq!(run("console.log(false || false);"), "false\n");
        assert_eq!(run("console.log(false || true);"), "true\n");
    }

    #[test]
    fn test_logical_short_circuit() {
        assert_eq!(run("console.log(5 || 10);"), "5\n");
        assert_eq!(run("console.log(0 || 10);"), "10\n");
        assert_eq!(run("console.log(5 && 10);"), "10\n");
        assert_eq!(run("console.log(0 && 10);"), "0\n");
    }

    // // ==========================================================================
    // // VARIABLES
    // // ==========================================================================

    #[test]
    fn test_let_declaration() {
        assert_eq!(run("let x = 5; console.log(x);"), "5\n");
        assert_eq!(run("let x; console.log(x);"), "undefined\n");
    }

    #[test]
    fn test_const_declaration() {
        assert_eq!(run("const x = 10; console.log(x);"), "10\n");
    }

    #[test]
    fn test_var_declaration() {
        assert_eq!(run("var x = 7; console.log(x);"), "7\n");
    }

    #[test]
    fn test_variable_assignment() {
        assert_eq!(run("let x = 5; x = 10; console.log(x);"), "10\n");
        assert_eq!(run("let x = 1; x = x + 1; console.log(x);"), "2\n");
    }

    #[test]
    fn test_multiple_variables() {
        let source = r#"
            let a = 5;
            let b = 10;
            let c = a + b;
            console.log(c);
        "#;
        assert_eq!(run(source), "15\n");
    }

    // // ==========================================================================
    // // IF STATEMENTS
    // // ==========================================================================

    #[test]
    fn test_if_true() {
        let source = r#"
            if (true) {
                console.log('yes');
            }
        "#;
        assert_eq!(run(source), "yes\n");
    }

    #[test]
    fn test_if_false() {
        let source = r#"
            if (false) {
                console.log('yes\n');
            }
        "#;
        assert_eq!(run(source), "");
    }

    #[test]
    fn test_if_else() {
        let source = r#"
            if (5 > 10) {
                console.log('bigger');
            } else {
                console.log('smaller');
            }
        "#;
        assert_eq!(run(source), "smaller\n");
    }

    #[test]
    fn test_if_else_chain() {
        let source = r#"
            let x = 5;
            if (x < 0) {
                console.log('negative');
            } else if (x == 0) {
                console.log('zero');
            } else {
                console.log('positive');
            }
        "#;
        assert_eq!(run(source), "positive\n");
    }

    // // ==========================================================================
    // // WHILE LOOPS
    // // ==========================================================================

    #[test]
    fn test_while_loop() {
        let source = r#"
            let i = 0;
            while (i < 3) {
                console.log(i);
                i = i + 1;
            }
        "#;
        assert_eq!(run(source), "0\n1\n2\n");
    }

    #[test]
    fn test_while_break() {
        let source = r#"
            let i = 0;
            while (true) {
                if (i >= 2) {
                    break;
                }
                console.log(i);
                i = i + 1;
            }
        "#;
        assert_eq!(run(source), "0\n1\n");
    }

    #[test]
    fn test_while_continue() {
        let source = r#"
            let i = 0;
            while (i < 5) {
                i = i + 1;
                if (i == 3) {
                    continue;
                }
                console.log(i);
            }
        "#;
        assert_eq!(run(source), "1\n2\n4\n5\n");
    }

    // // ==========================================================================
    // // FOR LOOPS
    // // ==========================================================================

    #[test]
    fn test_for_loop() {
        let source = r#"
            for (let i = 0; i < 3; i = i + 1) {
                console.log(i);
            }
        "#;
        assert_eq!(run(source), "0\n1\n2\n");
    }

    #[test]
    fn test_for_loop_no_init() {
        let source = r#"
            let i = 0;
            for (; i < 3; i = i + 1) {
                console.log(i);
            }
        "#;
        assert_eq!(run(source), "0\n1\n2\n");
    }

    #[test]
    fn test_for_loop_break() {
        let source = r#"
            for (let i = 0; i < 10; i = i + 1) {
                if (i == 3) {
                    break;
                }
                console.log(i);
            }
        "#;
        assert_eq!(run(source), "0\n1\n2\n");
    }

    #[test]
    fn test_for_loop_continue() {
        let source = r#"
            for (let i = 0; i < 5; i = i + 1) {
                if (i == 2) {
                    continue;
                }
                console.log(i);
            }
        "#;
        assert_eq!(run(source), "0\n1\n3\n4\n");
    }

    // // ==========================================================================
    // // FUNCTIONS
    // // ==========================================================================

    #[test]
    fn test_function_declaration() {
        let source = r#"
            function greet() {
                console.log('hello');
            }
            greet();
        "#;
        assert_eq!(run(source), "hello\n");
    }

    #[test]
    fn test_function_parameters() {
        let source = r#"
            function add(a, b) {
                return a + b;
            }
            console.log(add(5, 3));
        "#;
        assert_eq!(run(source), "8\n");
    }

    #[test]
    fn test_function_return() {
        let source = r#"
            function double(x) {
                return x * 2;
            }
            let result = double(7);
            console.log(result);
        "#;
        assert_eq!(run(source), "14\n");
    }

    #[test]
    fn test_function_no_return() {
        let source = r#"
            function doNothing() {
            }
            console.log(doNothing());
        "#;
        assert_eq!(run(source), "undefined\n");
    }

    #[test]
    fn test_function_expression() {
        let source = r#"
            let add = function(a, b) {
                return a + b;
            };
            console.log(add(10, 20));
        "#;
        assert_eq!(run(source), "30\n");
    }

    #[test]
    fn test_function_scope() {
        let source = r#"
            let x = 10;
            function test() {
                let x = 20;
                console.log(x);
            }
            test();
            console.log(x);
        "#;
        assert_eq!(run(source), "20\n10\n");
    }

    #[test]
    fn test_closure() {
        let source = r#"
            function outer(x) {
                function inner(y) {
                    return x + y;
                }
                return inner;
            }
            let add5 = outer(5);
            console.log(add5(3));
        "#;
        assert_eq!(run(source), "8\n");
    }

    #[test]
    fn test_nested_functions() {
        let source = r#"
            function outer() {
                function inner() {
                    console.log('inner');
                }
                inner();
            }
            outer();
        "#;
        assert_eq!(run(source), "inner\n");
    }

    #[test]
    fn test_recursive_function() {
        let source = r#"
            function factorial(n) {
                if (n <= 1) {
                    return 1;
                }
                return n * factorial(n - 1);
            }
            console.log(factorial(5));
        "#;
        assert_eq!(run(source), "120\n");
    }

    // // ==========================================================================
    // // ARROW FUNCTIONS
    // // ==========================================================================

    #[test]
    fn test_arrow_function_expression() {
        let source = r#"
            let double = x => x * 2;
            console.log(double(5));
        "#;
        assert_eq!(run(source), "10\n");
    }

    #[test]
    fn test_arrow_function_multiple_params() {
        let source = r#"
            let add = (a, b) => a + b;
            console.log(add(3, 7));
        "#;
        assert_eq!(run(source), "10\n");
    }

    #[test]
    fn test_arrow_function_no_params() {
        let source = r#"
            let greet = () => console.log('hi');
            greet();
        "#;
        assert_eq!(run(source), "hi\n");
    }

    #[test]
    fn test_arrow_function_block_body() {
        let source = r#"
            let add = (a, b) => {
                let result = a + b;
                return result;
            };
            console.log(add(4, 6));
        "#;
        assert_eq!(run(source), "10\n");
    }

    // // ==========================================================================
    // // ARRAYS
    // // ==========================================================================

    // #[test]
    // fn test_array_literal() {
    //     let source = r#"
    //         let arr = [1, 2, 3];
    //         console.log(arr[0]);
    //         console.log(arr[1]);
    //         console.log(arr[2]);
    //     "#;
    //     assert_eq!(run(source), "1\n2\n3\n");
    // }

    // #[test]
    // fn test_empty_array() {
    //     let source = r#"
    //         let arr = [];
    //         console.log(arr);
    //     "#;
    //     assert_eq!(run(source), "[]\n");
    // }

    // #[test]
    // fn test_array_assignment() {
    //     let source = r#"
    //         let arr = [1, 2, 3];
    //         arr[1] = 10;
    //         console.log(arr[1]);
    //     "#;
    //     assert_eq!(run(source), "10\n");
    // }

    // #[test]
    // fn test_nested_arrays() {
    //     let source = r#"
    //         let arr = [[1, 2], [3, 4]];
    //         console.log(arr[0][1]);
    //         console.log(arr[1][0]);
    //     "#;
    //     assert_eq!(run(source), "2\n3\n");
    // }

    // // ==========================================================================
    // // OBJECTS
    // // ==========================================================================

    // #[test]
    // fn test_object_literal() {
    //     let source = r#"
    //         let obj = {x: 10, y: 20};
    //         console.log(obj.x);
    //         console.log(obj.y);
    //     "#;
    //     assert_eq!(run(source), "10\n20\n");
    // }

    // #[test]
    // fn test_empty_object() {
    //     let source = r#"
    //         let obj = {};
    //         console.log(obj);
    //     "#;
    //     assert_eq!(run(source), "{}\n");
    // }

    // #[test]
    // fn test_object_property_assignment() {
    //     let source = r#"
    //         let obj = {x: 5};
    //         obj.x = 10;
    //         console.log(obj.x);
    //     "#;
    //     assert_eq!(run(source), "10\n");
    // }

    // #[test]
    // fn test_object_new_property() {
    //     let source = r#"
    //         let obj = {};
    //         obj.name = 'test';
    //         console.log(obj.name);
    //     "#;
    //     assert_eq!(run(source), "test\n");
    // }

    // #[test]
    // fn test_object_bracket_notation() {
    //     let source = r#"
    //         let obj = {x: 100};
    //         console.log(obj['x']);
    //     "#;
    //     assert_eq!(run(source), "100\n");
    // }

    // #[test]
    // fn test_nested_objects() {
    //     let source = r#"
    //         let obj = {
    //             inner: {
    //                 value: 42
    //             }
    //         };
    //         console.log(obj.inner.value);
    //     "#;
    //     assert_eq!(run(source), "42\n");
    // }

    // #[test]
    // fn test_object_methods() {
    //     let source = r#"
    //         let obj = {
    //             greet: function() {
    //                 return 'hello';
    //             }
    //         };
    //         console.log(obj.greet());
    //     "#;
    //     assert_eq!(run(source), "hello\n");
    // }

    // // ==========================================================================
    // // COMPLEX SCENARIOS
    // // ==========================================================================

    // #[test]
    // fn test_fibonacci() {
    //     let source = r#"
    //         function fib(n) {
    //             if (n <= 1) {
    //                 return n;
    //             }
    //             return fib(n - 1) + fib(n - 2);
    //         }
    //         console.log(fib(7));
    //     "#;
    //     assert_eq!(run(source), "13\n");
    // }

    // #[test]
    // fn test_counter_closure() {
    //     let source = r#"
    //         function makeCounter() {
    //             let count = 0;
    //             return function() {
    //                 count = count + 1;
    //                 return count;
    //             };
    //         }
    //         let counter = makeCounter();
    //         console.log(counter());
    //         console.log(counter());
    //         console.log(counter());
    //     "#;
    //     assert_eq!(run(source), "1\n2\n3\n");
    // }

    // #[test]
    // fn test_array_sum() {
    //     let source = r#"
    //         function sum(arr) {
    //             let total = 0;
    //             for (let i = 0; i < 5; i = i + 1) {
    //                 total = total + arr[i];
    //             }
    //             return total;
    //         }
    //         let nums = [1, 2, 3, 4, 5];
    //         console.log(sum(nums));
    //     "#;
    //     assert_eq!(run(source), "15\n");
    // }

    // #[test]
    // fn test_object_with_methods() {
    //     let source = r#"
    //         let calculator = {
    //             add: function(a, b) {
    //                 return a + b;
    //             },
    //             multiply: function(a, b) {
    //                 return a * b;
    //             }
    //         };
    //         console.log(calculator.add(5, 3));
    //         console.log(calculator.multiply(4, 7));
    //     "#;
    //     assert_eq!(run(source), "8\n28\n");
    // }

    // #[test]
    // fn test_increment_operators() {
    //     let source = r#"
    //         let x = 5;
    //         console.log(++x);
    //         console.log(x);
    //         let y = 10;
    //         console.log(y++);
    //         console.log(y);
    //     "#;
    //     assert_eq!(run(source), "6\n6\n10\n11\n");
    // }

    // #[test]
    // fn test_decrement_operators() {
    //     let source = r#"
    //         let x = 5;
    //         console.log(--x);
    //         console.log(x);
    //         let y = 10;
    //         console.log(y--);
    //         console.log(y);
    //     "#;
    //     assert_eq!(run(source), "4\n4\n10\n9\n");
    // }

    // // ==========================================================================
    // // ERROR CASES (should handle gracefully)
    // // ==========================================================================

    // #[test]
    // fn test_undefined_variable() {
    //     let (_stdout, stderr) = run_and_capture("console.log(unknownVar);");
    //     assert!(stderr.contains("ReferenceError") || stderr.contains("undefined"));
    // }

    // #[test]
    // fn test_const_reassignment() {
    //     let (_stdout, stderr) = run_and_capture("const x = 5; x = 10;");
    //     assert!(stderr.contains("TypeError") || stderr.contains("const"));
    // }

    // #[test]
    // fn test_call_non_function() {
    //     let (_stdout, stderr) = run_and_capture("let x = 5; x();");
    //     assert!(stderr.contains("TypeError") || stderr.contains("not a function"));
    // }
}
