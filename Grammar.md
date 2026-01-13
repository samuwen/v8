# JavaScript Grammar Reference (Attenuated)

## Program Structure

```
Program         → Statement* EOF

Statement       → VariableDecl
                | FunctionDecl
                | ExpressionStmt
                | BlockStmt
                | IfStmt
                | WhileStmt
                | ForStmt
                | ReturnStmt
                | BreakStmt
                | ContinueStmt
```

## Declarations

```
VariableDecl    → ("let" | "const" | "var") IDENTIFIER ("=" Expression)? ";"

FunctionDecl    → "function" IDENTIFIER "(" Parameters? ")" BlockStmt

Parameters      → IDENTIFIER ("," IDENTIFIER)*
```

## Statements

```
ExpressionStmt  → Expression ";"

BlockStmt       → "{" Statement* "}"

IfStmt          → "if" "(" Expression ")" Statement ("else" Statement)?

WhileStmt       → "while" "(" Expression ")" Statement

ForStmt         → "for" "(" (VariableDecl | ExpressionStmt | ";")
                           Expression? ";"
                           Expression? ")" Statement

ReturnStmt      → "return" Expression? ";"

BreakStmt       → "break" ";"

ContinueStmt    → "continue" ";"
```

## Expressions

Ordered by precedence (lowest to highest):

```
Expression      → Assignment

Assignment      → (Call ".")? IDENTIFIER "=" Assignment
                | LogicalOr

LogicalOr       → LogicalAnd ("||" LogicalAnd)*

LogicalAnd      → Equality ("&&" Equality)*

Equality        → Comparison (("==" | "!=" | "===" | "!==") Comparison)*

Comparison      → Term ((">" | ">=" | "<" | "<=") Term)*

Term            → Factor (("+" | "-") Factor)*

Factor          → Unary (("*" | "/" | "%") Unary)*

Unary           → ("!" | "-" | "++" | "--") Unary
                | Postfix

Postfix         → Call ("++" | "--")?

Call            → Primary ( "(" Arguments? ")" 
                          | "[" Expression "]" 
                          | "." IDENTIFIER )*

Arguments       → Expression ("," Expression)*

Primary         → NUMBER
                | STRING
                | "true" | "false" | "null" | "undefined"
                | IDENTIFIER
                | "(" Expression ")"
                | ArrayLiteral
                | ObjectLiteral
                | FunctionExpr
                | ArrowFunction
```

## Literals

```
ArrayLiteral    → "[" (Expression ("," Expression)*)? "]"

ObjectLiteral   → "{" (Property ("," Property)*)? "}"

Property        → (IDENTIFIER | STRING) ":" Expression

FunctionExpr    → "function" IDENTIFIER? "(" Parameters? ")" BlockStmt

ArrowFunction   → (IDENTIFIER | "(" Parameters? ")") "=>" (Expression | BlockStmt)
```

## Lexical Elements

```
NUMBER          → [0-9]+ ("." [0-9]+)?

STRING          → '"' [^"]* '"' | "'" [^']* "'"

IDENTIFIER      → [a-zA-Z_$][a-zA-Z0-9_$]*
```

## Operator Precedence Summary

From lowest to highest:

1. Assignment: `=`
2. Logical OR: `||`
3. Logical AND: `&&`
4. Equality: `==`, `!=`, `===`, `!==`
5. Comparison: `>`, `>=`, `<`, `<=`
6. Addition/Subtraction: `+`, `-`
7. Multiplication/Division: `*`, `/`, `%`
8. Unary: `!`, `-`, `++`, `--`
9. Postfix: `++`, `--`
10. Call/Member: `()`, `[]`, `.`

## Features Included

**Variables:**

- `let`, `const`, `var` declarations with optional initialization
- Assignment expressions

**Control Flow:**

- `if`/`else` conditionals
- `while` loops
- `for` loops (C-style: init; condition; increment)
- `break` and `continue` statements

**Functions:**

- Function declarations: `function foo(x, y) { ... }`
- Function expressions: `let f = function(x) { ... }`
- Arrow functions: `x => x * 2` and `(x, y) => { ... }`
- Return statements
- Function calls with arguments

**Operators:**

- Logical: `||`, `&&`
- Equality: `==`, `!=`, `===`, `!==`
- Comparison: `>`, `>=`, `<`, `<=`
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Unary: `!`, `-`
- Increment/Decrement: `++`, `--` (prefix and postfix)

**Data Structures:**

- Arrays: `[1, 2, 3]`
- Objects: `{x: 10, y: 20}`
- Property access: `obj.prop`
- Computed property access: `obj[expr]`
- Method calls: `obj.method(args)`
- Call chaining: `obj.method().property`

**Literals:**

- Numbers: `42`, `3.14`
- Strings: `"hello"`, `'world'`
- Booleans: `true`, `false`
- Special values: `null`, `undefined`

## Deliberately Omitted

Features intentionally left out for simplicity (can be added later):

- `switch` statements
- `do-while` loops
- `for-in` / `for-of` loops
- Try-catch-finally
- Classes and constructors
- `this` keyword
- Async/await and Promises
- Destructuring assignment
- Spread/rest operators (`...`)
- Template literals
- Regular expressions
- Operators: `typeof`, `instanceof`, `in`, `delete`, `void`
- Ternary operator (`? :`)
- Comma operator
- Bitwise operators
- Computed property names in object literals
- Shorthand property syntax
- Method definitions in objects

## Suggested Implementation Order

**Phase 1 - Basic Expressions (Week 1):**

- Literals (numbers, strings, booleans)
- Binary operators (arithmetic, comparison)
- Variables and assignment
- Expression statements

**Phase 2 - Functions (Week 2):**

- Function declarations
- Function calls
- Return statements
- Parameters and arguments

**Phase 3 - Control Flow (Week 3):**

- If/else statements
- While loops
- Logical operators (`&&`, `||`)
- Block statements

**Phase 4 - Data Structures (Week 4):**

- Arrays and array literals
- Objects and object literals
- Property access (dot and bracket notation)
- Method calls

**Phase 5 - Advanced Features (Week 5):**

- For loops
- Function expressions
- Arrow functions
- Increment/decrement operators

## Example Programs This Grammar Can Parse

**Variables and arithmetic:**

```javascript
let x = 10;
let y = 20;
let sum = x + y;
```

**Functions:**

```javascript
function add(a, b) {
    return a + b;
}

let result = add(5, 3);
```

**Control flow:**

```javascript
if (x > 10) {
    console.log("big");
} else {
    console.log("small");
}

while (x > 0) {
    x = x - 1;
}
```

**Arrays and objects:**

```javascript
let arr = [1, 2, 3];
let obj = { x: 10, y: 20 };
let value = obj.x + arr[0];
```

**Arrow functions:**

```javascript
let double = (x) => x * 2;
let sum = (a, b) => a + b;
```

## Notes on Ambiguities

**Arrow functions vs comparison:** The sequence `x => ...` could be confused
with comparison. Resolve by checking for `=>` token after identifier/parameter
list.

**Function expressions vs declarations:** Start of statement with `function`
keyword is a declaration. `function` in expression position is an expression.

**Object literals vs blocks:** `{` at start of statement is a block. `{` in
expression position is an object literal.

**Increment/decrement:** `++` and `--` can be prefix or postfix. Parser must
track position relative to operand.

**Property assignment:** Left side of `=` must be validated: either simple
identifier or member expression (`obj.prop` or `obj[expr]`).
